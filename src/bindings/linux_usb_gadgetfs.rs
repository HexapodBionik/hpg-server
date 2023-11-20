pub const usb_gadgetfs_event_type_GADGETFS_NOP: usb_gadgetfs_event_type = 0;
pub const usb_gadgetfs_event_type_GADGETFS_CONNECT: usb_gadgetfs_event_type = 1;
pub const usb_gadgetfs_event_type_GADGETFS_DISCONNECT: usb_gadgetfs_event_type = 2;
pub const usb_gadgetfs_event_type_GADGETFS_SETUP: usb_gadgetfs_event_type = 3;
pub const usb_gadgetfs_event_type_GADGETFS_SUSPEND: usb_gadgetfs_event_type = 4;
pub type usb_gadgetfs_event_type = ::std::os::raw::c_uint;

#[repr(C)]
#[derive(Copy, Clone)]
pub union usb_gadgetfs_event__bindgen_ty_1 {
    pub speed: usb_device_speed,
    pub setup: usb_ctrlrequest,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct usb_gadgetfs_event {
    pub u: usb_gadgetfs_event__bindgen_ty_1,
    pub type_: usb_gadgetfs_event_type,
}
