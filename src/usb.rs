pub mod ethernet;
pub mod network_stack;
pub mod serial;
pub mod serial_logger;
pub mod usb_device;

/// Maximum transmission unit of the ethernet frames.
pub const MTU: usize = 1514;
/// Port hosting the server.
pub const SERVER_PORT: u16 = 1234;
/// MAC address of the STM32.
pub const OUR_MAC_ADDR: [u8; 6] = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
/// MAC address the host "thinks" the USB-to-ethernet adapter has.
pub const HOST_MAC_ADDR: [u8; 6] = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];
/// Hostname of the device.
pub const HOSTNAME: &str = "wave-rs";

/// USB vendor ID.
pub const USB_VID: u16 = 0xc0de;
/// USB product ID.
pub const USB_PID: u16 = 0xcafe;
/// USB manufacturer string.
pub const USB_MANUFACTURER: &str = "etiennecollin";
/// USB product string.
pub const USB_PRODUCT: &str = "wave-rs";
/// USB serial number.
pub const USB_SN: &str = "wave-rs-0001";
/// USB release version in BCD.
pub const USB_RELEASE_VERSION: u16 = 0x0010;
