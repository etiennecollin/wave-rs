use embassy_stm32::{
    peripherals::{self, USB_OTG_HS},
    usb::Driver,
};
use embassy_usb::{Builder, UsbDevice, UsbVersion};
use static_cell::StaticCell;

use crate::{
    usb::{USB_MANUFACTURER, USB_PID, USB_PRODUCT, USB_RELEASE_VERSION, USB_SN, USB_VID},
    Irqs,
};

/// Initializes a USB peripheral builder.
///
/// The USB device is configured as a composite device. Its maximum current draw is 100 mA and it
/// supports remote wakeup.
///
/// # Arguments
/// - `usb`: The USB peripheral.
/// - `dp`: The USB D+ pin.
/// - `dm`: The USB D- pin.
pub async fn init_usb(
    usb: USB_OTG_HS,
    dp: peripherals::PA12,
    dm: peripherals::PA11,
) -> Builder<'static, Driver<'static, USB_OTG_HS>> {
    // Create a buffer for the output endpoint.
    static OUTPUT_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
    let ep_out_buffer = &mut OUTPUT_BUFFER.init([0; 512])[..];

    // Create the driver, from the HAL.
    let mut hal_config = embassy_stm32::usb::Config::default();
    hal_config.vbus_detection = false; // Only enable if board can stay powered on without USB.
    let driver = Driver::new_hs(usb, Irqs, dp, dm, ep_out_buffer, hal_config);

    // =========================================================================
    // Create embassy-usb Config
    // =========================================================================
    let mut config = embassy_usb::Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(USB_MANUFACTURER);
    config.product = Some(USB_PRODUCT);
    config.serial_number = Some(USB_SN);
    config.device_release = USB_RELEASE_VERSION;
    config.bcd_usb = UsbVersion::TwoOne;
    config.max_packet_size_0 = 64; // Full speed is 64
    config.max_power = 100; // in mA
    config.self_powered = false;
    config.supports_remote_wakeup = true;

    // Configures the device as a composite device with interface association descriptors
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    // =========================================================================

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut [], // no msos descriptors
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    builder
}

/// Runs a USB device.
#[embassy_executor::task]
pub async fn usb_task(mut device: UsbDevice<'static, Driver<'static, USB_OTG_HS>>) -> ! {
    device.run().await
}
