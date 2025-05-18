use core::sync::atomic::{AtomicBool, Ordering};

use defmt::info;
use embassy_stm32::{
    peripherals::USB_OTG_HS,
    usb::{DmPin, DpPin, Driver},
    Peri,
};
use embassy_usb::{Builder, Handler, UsbDevice, UsbVersion};
use static_cell::StaticCell;

use crate::{
    usb::{
        USB_BOS_DESC_SIZE, USB_CONFIG_DESC_SIZE, USB_CONTROL_BUF_SIZE, USB_MANUFACTURER,
        USB_MSOS_DESC_SIZE, USB_OUTPUT_BUFFER_SIZE, USB_PID, USB_PRODUCT, USB_RELEASE_VERSION,
        USB_SN, USB_VID,
    },
    Irqs,
};

/// Initializes a USB peripheral builder.
///
/// The USB device is configured as a composite device. Its maximum current draw is 100 mA and it
/// supports remote wakeup.
///
/// # Arguments
///
/// - `usb`: The USB peripheral.
/// - `dp`: The USB D+ pin.
/// - `dm`: The USB D- pin.
pub async fn init_usb(
    usb: Peri<'static, USB_OTG_HS>,
    dp: Peri<'static, impl DpPin<USB_OTG_HS>>,
    dm: Peri<'static, impl DmPin<USB_OTG_HS>>,
) -> Builder<'static, Driver<'static, USB_OTG_HS>> {
    // Create a buffer for the output endpoint.
    static USB_OUTPUT_BUFFER: StaticCell<[u8; USB_OUTPUT_BUFFER_SIZE]> = StaticCell::new();
    let ep_out_buffer = &mut USB_OUTPUT_BUFFER.init([0; USB_OUTPUT_BUFFER_SIZE])[..];

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
    static USB_CONFIG_DESC: StaticCell<[u8; USB_CONFIG_DESC_SIZE]> = StaticCell::new();
    static USB_BOS_DESC: StaticCell<[u8; USB_BOS_DESC_SIZE]> = StaticCell::new();
    static USB_MSOS_DESC: StaticCell<[u8; USB_MSOS_DESC_SIZE]> = StaticCell::new();
    static USB_CONTROL_BUF: StaticCell<[u8; USB_CONTROL_BUF_SIZE]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        USB_CONFIG_DESC.init([0; USB_CONFIG_DESC_SIZE]),
        USB_BOS_DESC.init([0; USB_BOS_DESC_SIZE]),
        USB_MSOS_DESC.init([0; USB_MSOS_DESC_SIZE]),
        USB_CONTROL_BUF.init([0; USB_CONTROL_BUF_SIZE]),
    );

    static USB_DEVICE_HANDLER: StaticCell<USBDeviceHandler> = StaticCell::new();
    builder.handler(USB_DEVICE_HANDLER.init(USBDeviceHandler::new()));

    builder
}

/// Runs a USB device.
#[embassy_executor::task]
pub async fn usb_task(mut device: UsbDevice<'static, Driver<'static, USB_OTG_HS>>) -> ! {
    device.run().await
}

struct USBDeviceHandler {
    configured: AtomicBool,
}

impl USBDeviceHandler {
    fn new() -> Self {
        USBDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for USBDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
