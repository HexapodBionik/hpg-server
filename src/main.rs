use std::env;

use hpg_server::start_server;

fn main() {
    let mut args = env::args().skip(1);
    
    let usb_device_path = match args.next() {
        None => panic!("USB device path not provided."),
        Some(usb_device_path) => usb_device_path,
    };
    let files = args.collect::<Vec<String>>();
    if files.len() < 1 {
        panic!("No filenames provided.");
    }
    println!("{:?}, {:?}", usb_device_path, files);
    start_server(usb_device_path, files);
}
