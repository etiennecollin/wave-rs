use defmt::warn;
use embassy_stm32::{peripherals::USB_OTG_HS, usb::Driver};
use embassy_time::Timer;
use embassy_usb::class::hid::HidWriter;
use packed_struct::PackedStruct;
use usbd_human_interface_device::device::mouse::BootMouseReport;

use crate::usb::HID_MOUSE_WRITER_N;

/// Runs a HID writer task.
#[embassy_executor::task]
pub async fn mouse_writer_task(
    mut writer: HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_MOUSE_WRITER_N>,
) {
    let mut y: i8 = 25;
    loop {
        Timer::after_millis(500).await;
        y = -y;

        let report = BootMouseReport {
            buttons: 0,
            x: 0,
            y,
        }
        .pack()
        .unwrap();

        match writer.write(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        }
    }
}
