pub mod ethernet;
pub mod hid;
pub mod serial;
pub mod usb_device;

// =============================================================================
// USB
// =============================================================================
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

/// USB output buffer size.
pub const USB_OUTPUT_BUFFER_SIZE: usize = 256;
/// USB configuration descriptor size.
pub const USB_CONFIG_DESC_SIZE: usize = 256;
/// USB BOS descriptor size.
pub const USB_BOS_DESC_SIZE: usize = 64;
/// USB MSOS descriptor size.
pub const USB_MSOS_DESC_SIZE: usize = 64;
/// USB control buffer size.
pub const USB_CONTROL_BUF_SIZE: usize = 64;

// =============================================================================
// HID
// =============================================================================
/// Polling interval of the HID device in milliseconds.
pub const HID_KEYBOARD_POLL_MS: u8 = 1;
/// Maximum size in bytes of a HID packet.
pub const HID_KEYBOARD_MAX_PACKET_SIZE: u16 = 32;
/// Size in bytes of the reports received by the HID reader.
pub const HID_KEYBOARD_READER_N: usize = 1;
/// Size in bytes of the keyboard report sent to the HID writer.
pub const HID_KEYBOARD_WRITER_N: usize = 25;

/// Polling interval of the HID device in milliseconds.
pub const HID_MOUSE_POLL_MS: u8 = 60;
/// Maximum size in bytes of a HID packet.
pub const HID_MOUSE_MAX_PACKET_SIZE: u16 = 8;
/// Size in bytes of the mouse report sent to the HID writer.
pub const HID_MOUSE_WRITER_N: usize = 5;

// =============================================================================
// Ethernet
// =============================================================================
/// Maximum transmission unit of the ethernet frames.
pub const MTU: usize = 1500;
/// Maximum size of an ethernet packet.
pub const ETH_MAX_PACKET_SIZE: u16 = 64;
/// Port hosting the server.
pub const SERVER_PORT: u16 = 8080;
/// MAC address of the STM32.
pub const OUR_MAC_ADDR: [u8; 6] = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
/// MAC address the host "thinks" the USB-to-ethernet adapter has.
pub const HOST_MAC_ADDR: [u8; 6] = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];
