use defmt::*;
use embassy_stm32::{exti::ExtiInput, peripherals::USB_OTG_HS, usb::Driver};
use embassy_usb::class::hid::HidWriter;
use packed_struct::PackedStruct;
use usbd_human_interface_device::{device::keyboard::NKROBootKeyboardReport, page::Keyboard};

use crate::usb::HID_KEYBOARD_WRITER_N;

/// Runs a HID writer task.
#[embassy_executor::task]
pub async fn keyboard_scan_task(
    mut writer: HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_WRITER_N>,
    mut button: ExtiInput<'static>,
) {
    loop {
        button.wait_for_high().await;
        info!("Button pressed!");
        // Create a report with the A key pressed. (no shift modifier)
        let report = NKROBootKeyboardReport::new([Keyboard::A]).pack().unwrap();
        match writer.write(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };

        button.wait_for_low().await;
        info!("Button released!");
        let report = NKROBootKeyboardReport::default().pack().unwrap();
        match writer.write(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };
    }
}
