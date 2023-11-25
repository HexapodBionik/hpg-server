use libc::{self, c_void};
use nix::fcntl::{self, OFlag};
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::stat::{self, Mode};
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::mem;
use std::os::fd::{FromRawFd, RawFd};
use std::process;
use std::thread;

mod bindings;

const STRINGID_LANGID: u8 = 0;
const STRINGID_MANUFACTURER: u8 = 1;
const STRINGID_PRODUCT: u8 = 2;
const STRINGID_SERIAL: u8 = 3;
const STRINGID_CONFIG_HS: u8 = 4;
const STRINGID_CONFIG_LS: u8 = 5;
const STRINGID_INTERFACE: u8 = 6;

const DEVICE_DESCRIPTOR: bindings::usb_device_descriptor = bindings::usb_device_descriptor {
    bLength: bindings::USB_DT_DEVICE_SIZE as u8,
    bDescriptorType: bindings::USB_DT_DEVICE as u8,

    bcdUSB: 0x0200, /* USB 2.0 */
    bDeviceClass: bindings::USB_CLASS_COMM as u8,
    bDeviceSubClass: 0,
    bDeviceProtocol: 0,
    bMaxPacketSize0: 255, /* Set by driver */
    idVendor: 0x1209,     /* pid.codes */
    idProduct: 0x0001,    /* pid.codes Test Device */
    bcdDevice: 0x0100,    /* Version */
    iManufacturer: STRINGID_MANUFACTURER,
    iProduct: STRINGID_PRODUCT,
    iSerialNumber: STRINGID_SERIAL,
    bNumConfigurations: 1, /* Only one configuration */
};

const BASIC_EP_DESCRIPTOR: bindings::usb_endpoint_descriptor = bindings::usb_endpoint_descriptor {
    bLength: bindings::USB_DT_ENDPOINT_SIZE as u8,
    bDescriptorType: bindings::USB_DT_ENDPOINT as u8,

    bEndpointAddress: bindings::USB_DIR_OUT as u8, /* EP number has to be set individually */
    bmAttributes: bindings::USB_ENDPOINT_XFER_BULK as u8,
    wMaxPacketSize: 512,
    bInterval: 0,

    /* Audio Extension */
    bRefresh: 0,
    bSynchAddress: 0,
};

const IF_DESCRIPTOR: bindings::usb_interface_descriptor = bindings::usb_interface_descriptor {
    bLength: bindings::USB_DT_INTERFACE_SIZE as u8,
    bDescriptorType: bindings::USB_DT_INTERFACE as u8,

    bInterfaceNumber: 0,
    bAlternateSetting: 0,
    bNumEndpoints: 0, /* Number of endpoints has to be set individually */
    bInterfaceClass: bindings::USB_CLASS_COMM as u8,
    bInterfaceSubClass: 0,
    bInterfaceProtocol: 0,
    iInterface: STRINGID_INTERFACE,
};

/* Low speed configuration */
const CONFIG: bindings::usb_config_descriptor = bindings::usb_config_descriptor {
    bLength: bindings::USB_DT_CONFIG_SIZE as u8,
    bDescriptorType: bindings::USB_DT_CONFIG as u8,

    wTotalLength: 0, /* To be computed */
    bNumInterfaces: 1,
    bConfigurationValue: 2,
    iConfiguration: STRINGID_CONFIG_LS,
    bmAttributes: bindings::USB_CONFIG_ATT_ONE as u8 | bindings::USB_CONFIG_ATT_SELFPOWER as u8,
    bMaxPower: 1,
};

/* High speed configuration */
const CONFIG_HS: bindings::usb_config_descriptor = bindings::usb_config_descriptor {
    bLength: bindings::USB_DT_CONFIG_SIZE as u8,
    bDescriptorType: bindings::USB_DT_CONFIG as u8,

    wTotalLength: 0, /* To be computed */
    bNumInterfaces: 1,
    bConfigurationValue: 2,
    iConfiguration: STRINGID_CONFIG_HS,
    bmAttributes: bindings::USB_CONFIG_ATT_ONE as u8 | bindings::USB_CONFIG_ATT_SELFPOWER as u8,
    bMaxPower: 1,
};

fn format_ep_init_package(ep_num: u8) -> Vec<u8> {
    let mut package: Vec<u8> = Vec::with_capacity(4 + (BASIC_EP_DESCRIPTOR.bLength as usize) * 2);

    package.extend_from_slice(&[1u8, 0u8, 0u8, 0u8]);

    let mut ep_descriptor = BASIC_EP_DESCRIPTOR.clone();
    ep_descriptor.bEndpointAddress |= ep_num;

    /* Do not copy audio extension bytes */
    package.extend_from_slice(
        &unsafe {
            mem::transmute::<_, [u8; mem::size_of::<bindings::usb_endpoint_descriptor>()]>(
                ep_descriptor,
            )
        }[0..mem::size_of::<bindings::usb_endpoint_descriptor>() - 2],
    );
    package.extend_from_slice(
        &unsafe {
            mem::transmute::<_, [u8; mem::size_of::<bindings::usb_endpoint_descriptor>()]>(
                ep_descriptor,
            )
        }[0..mem::size_of::<bindings::usb_endpoint_descriptor>() - 2],
    );

    package
}

fn format_init_package(eps_cnt: u8) -> Vec<u8> {
    let mut if_descriptor = IF_DESCRIPTOR.clone();
    let mut config = CONFIG.clone();
    let mut config_hs = CONFIG_HS.clone();

    if_descriptor.bNumEndpoints = eps_cnt;
    config.wTotalLength = CONFIG.bLength as u16
        + IF_DESCRIPTOR.bLength as u16
        + BASIC_EP_DESCRIPTOR.bLength as u16 * eps_cnt as u16;
    config_hs.wTotalLength = CONFIG.bLength as u16
        + IF_DESCRIPTOR.bLength as u16
        + BASIC_EP_DESCRIPTOR.bLength as u16 * eps_cnt as u16;

    let mut package: Vec<u8> = Vec::with_capacity(
        4 + config.wTotalLength as usize
            + config_hs.wTotalLength as usize
            + DEVICE_DESCRIPTOR.bLength as usize,
    );

    package.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);
    package.extend_from_slice(&unsafe {
        mem::transmute::<_, [u8; mem::size_of::<bindings::usb_config_descriptor>()]>(config)
    });
    package.extend_from_slice(&unsafe {
        mem::transmute::<_, [u8; mem::size_of::<bindings::usb_interface_descriptor>()]>(
            if_descriptor,
        )
    });

    for i in 1..=eps_cnt {
        let mut ep_descriptor = BASIC_EP_DESCRIPTOR.clone();
        ep_descriptor.bEndpointAddress |= i;

        /* Do not copy audio extension bytes */
        package.extend_from_slice(
            &unsafe {
                mem::transmute::<_, [u8; mem::size_of::<bindings::usb_endpoint_descriptor>()]>(
                    ep_descriptor,
                )
            }[0..mem::size_of::<bindings::usb_endpoint_descriptor>() - 2],
        );
    }

    package.extend_from_slice(&unsafe {
        mem::transmute::<_, [u8; mem::size_of::<bindings::usb_config_descriptor>()]>(config_hs)
    });
    package.extend_from_slice(&unsafe {
        mem::transmute::<_, [u8; mem::size_of::<bindings::usb_interface_descriptor>()]>(
            if_descriptor,
        )
    });

    for i in 1..=eps_cnt {
        let mut ep_descriptor = BASIC_EP_DESCRIPTOR.clone();
        ep_descriptor.bEndpointAddress |= i;

        /* Do not copy audio extension bytes */
        package.extend_from_slice(
            &unsafe {
                mem::transmute::<_, [u8; mem::size_of::<bindings::usb_endpoint_descriptor>()]>(
                    ep_descriptor,
                )
            }[0..mem::size_of::<bindings::usb_endpoint_descriptor>() - 2],
        );
    }

    package.extend_from_slice(&unsafe {
        mem::transmute::<_, [u8; mem::size_of::<bindings::usb_device_descriptor>()]>(
            DEVICE_DESCRIPTOR,
        )
    });

    package
}

fn ep_io_thread(ep_fd: RawFd, ep_num: u8, filename: String) {
    let mut duty_cycle_fp =
    match OpenOptions::new().write(true).open(&filename) {
        Ok(fp) => fp,
        Err(_) => {
            eprintln!("Warning: Could not open \"{}\". Stopping thread.",
                      filename);
            return;
        },
    };

    let fp = unsafe { File::from_raw_fd(ep_fd) };
    let poll_fd = PollFd::new(&fp, PollFlags::POLLIN);

    let buf: &mut [u8] = &mut [0u8; 8];

    loop {
        let _ = poll::poll(&mut [poll_fd], 0).unwrap();

        let bytes_cnt =
            unsafe { libc::read(ep_fd, buf.as_ptr() as *mut u8 as *mut c_void, buf.len()) };
        if bytes_cnt < 0 {
            eprintln!(
                "Warning: Could not read from \"/dev/gadget/ep{}out\".",
                ep_num
            );
        } else {
            let result = duty_cycle_fp.write_all(&buf[0..bytes_cnt as usize]);
            if result.is_err() {
                eprintln!(
                    "Warning: Could not write to \"{}\".",
                    filename
                );
            }
        }
    }
}

fn init_ep(ep_num: u8) -> Option<RawFd> {
    let ep_filename = format!("/dev/gadget/ep{}out", ep_num);
    let ep_fd = fcntl::open(
        ep_filename.as_str(),
        OFlag::O_RDWR | OFlag::O_SYNC,
        Mode::S_IRWXU,
    )
    .unwrap();
    if ep_fd <= 0 {
        println!("Warning: Unable to open /dev/gadget/ep{}out", ep_num);
        return None;
    }

    let package = format_ep_init_package(ep_num);
    let package = package.as_slice();
    let bytes_cnt = unsafe {
        libc::write(
            ep_fd,
            package.as_ptr() as *const u8 as *const c_void,
            package.len(),
        )
    };
    if bytes_cnt < 0 {
        eprintln!(
            "Warning: Write to \"{}\" failed (error {}).",
            ep_filename, -bytes_cnt
        );
        return None;
    }

    println!("Info: EP{} configured.", ep_num);

    Some(ep_fd)
}

fn make_utf16le(string: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for chunk in string.encode_utf16() {
        result.push(chunk as u8);
        result.push((chunk >> 8) as u8);
    }

    result
}

fn make_usb_string(string: &str) -> Vec<u8> {
    let mut utf16le_string = make_utf16le(string);
    let mut usb_string: Vec<u8> = Vec::with_capacity(utf16le_string.len() + 2);

    usb_string.push(utf16le_string.len() as u8 + 2);
    usb_string.push(bindings::USB_DT_STRING as u8);
    usb_string.append(&mut utf16le_string);

    usb_string
}

fn usb_gadget_get_string(id: u8) -> Option<Vec<u8>> {
    match id {
        STRINGID_LANGID => Some(vec![4u8, bindings::USB_DT_STRING as u8, 0x09u8, 0x04u8]),
        STRINGID_MANUFACTURER => Some(make_usb_string("Antoni Przybylik")),
        STRINGID_PRODUCT => Some(make_usb_string("Bionik Hexapod PWM Gadget")),
        STRINGID_SERIAL => Some(make_usb_string("0001")),
        STRINGID_CONFIG_HS => Some(make_usb_string("High speed configuration")),
        STRINGID_CONFIG_LS => Some(make_usb_string("Low speed configuration")),
        STRINGID_INTERFACE => Some(make_usb_string("PWM control interface")),
        _ => None,
    }
}

fn handle_setup_request(fd: RawFd, setup: &bindings::usb_ctrlrequest, files: Vec<String>) {
    match setup.bRequest as u32 {
        bindings::USB_REQ_GET_DESCRIPTOR => {
            println!("* * GET_DESCRIPTOR");
            if setup.bRequestType == (bindings::USB_DIR_IN as u8) {
                match (setup.wValue >> 8) as u32 {
                    bindings::USB_DT_STRING => {
                        match usb_gadget_get_string(setup.wValue as u8) {
                            Some(string) => {
                                let string = string.as_slice();
                                unsafe {
                                    libc::write(
                                        fd,
                                        string.as_ptr() as *const u8 as *const c_void,
                                        string.len(),
                                    )
                                };
                            }
                            None => {
                                eprintln!("Warning: String not found.");
                            }
                        }

                        return;
                    }
                    num => {
                        eprintln!("Warning: Could not return descriptor {}", num);
                    }
                }
            }
        }
        bindings::USB_REQ_SET_CONFIGURATION => {
            println!("* * SET_CONFIGURATION");
            if setup.bRequestType == bindings::USB_DIR_OUT as u8 {
                match setup.wValue {
                    2 => {
                        /* Set configuration #2 */

                        for (i, filename) in files.into_iter().enumerate() {
                            let i = (i+1) as u8;
                            let ep_fd = init_ep(i);
                            let ep_fd = match ep_fd {
                                Some(ep_fd) => ep_fd,
                                None => {
                                    eprintln!("Warning: Could not configure endpoint {}", i);
                                    continue;
                                }
                            };
                            thread::spawn(move || {
                                ep_io_thread(ep_fd, i, filename);
                            });
                        }
                    }
                    0 => {
                        panic!();
                    }
                    _ => {
                        println!("Warning: Unhandled config value");
                    }
                }

                unsafe { libc::read(fd, &mut [] as *mut c_void, 0) };
                return;
            } else {
                eprintln!("Error: Bad direction.");
            }
        }
        bindings::USB_REQ_GET_INTERFACE => {
            println!("* * GET_INTERFACE");
            unsafe { libc::write(fd, &[0u8] as *const u8 as *const c_void, 1) };
            return;
        }
        bindings::USB_REQ_SET_INTERFACE => {
            println!("* * SET_INTERFACE");
            unimplemented!();
        }
        _ => {}
    }

    println!("Info: Stalled");
    if (setup.bRequestType & (bindings::USB_DIR_IN as u8)) != 0 {
        unsafe { libc::read(fd, &mut [] as *mut c_void, 0) };
    } else {
        unsafe { libc::write(fd, &[] as *const c_void, 0) };
    }
}

fn server_loop(fd: RawFd, usb_dev_path: String, files: Vec<String>) {
    let fp = unsafe { File::from_raw_fd(fd) };
    let poll_fd = PollFd::new(&fp, PollFlags::POLLIN);

    let events: &mut [bindings::usb_gadgetfs_event; 5] = &mut unsafe {
        mem::transmute::<_, [bindings::usb_gadgetfs_event; 5]>(
            [0u8; mem::size_of::<bindings::usb_gadgetfs_event>() * 5],
        )
    };

    loop {
        let _ = poll::poll(&mut [poll_fd], 0).unwrap();

        let bytes_cnt = unsafe {
            libc::read(
                fd,
                events as *mut bindings::usb_gadgetfs_event as *mut u8 as *mut c_void,
                mem::size_of::<bindings::usb_gadgetfs_event>() * events.len(),
            )
        };
        if bytes_cnt < 0 {
            eprintln!(
                "Warning: Could not read {} bytes from \"{}\".",
                mem::size_of::<bindings::usb_gadgetfs_event>() * events.len(),
                usb_dev_path
            );
            return;
        }

        let events_cnt: usize = <isize as TryInto<usize>>::try_into(bytes_cnt)
            .unwrap()
            .div_euclid(mem::size_of::<bindings::usb_gadgetfs_event>());

        for i in 0..events_cnt {
            match events[i].type_ {
                bindings::usb_gadgetfs_event_type_GADGETFS_CONNECT => {
                    println!("* EP0 CONNECT");
                }
                bindings::usb_gadgetfs_event_type_GADGETFS_DISCONNECT => {
                    println!("* EP0 DISCONNECT");
                }
                bindings::usb_gadgetfs_event_type_GADGETFS_SETUP => {
                    println!("* EP0 SETUP");
                    handle_setup_request(fd, &unsafe { events[i].u.setup }, files.clone());
                }
                _ => break,
            }
        }
    }
}

pub fn start_server(usb_dev_path: String, files: Vec<String>) {
    if stat::stat(usb_dev_path.as_str()).is_err() {
        eprintln!("Error: Could not stat \"{}\".", usb_dev_path);
        process::exit(-1);
    }

    let fd: RawFd =
        fcntl::open(usb_dev_path.as_str(), OFlag::O_RDWR | OFlag::O_SYNC, Mode::S_IRWXU).unwrap();

    let package = format_init_package(files.len() as u8);
    let package = package.as_slice();
    let bytes_cnt = unsafe {
        libc::write(
            fd,
            package.as_ptr() as *const u8 as *const c_void,
            package.len(),
        )
    };
    if bytes_cnt < 0 {
        eprintln!(
            "Error: Write to \"{}\" failed (error {}).",
            usb_dev_path, -bytes_cnt
        );
        process::exit(-1);
    }

    println!("Info: EP0 configured. Starting the server.");
    server_loop(fd, usb_dev_path, files);
}
