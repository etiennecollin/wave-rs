use defmt::{info, warn};
use embassy_stm32::{
    dma::{ReadableRingBuffer, WritableRingBuffer},
    peripherals::USB_OTG_HS,
    usb::Driver,
};
use embassy_usb::class::hid::HidWriter;
use heapless::Vec;
use packed_struct::PackedStruct;
use usbd_human_interface_device::{device::keyboard::NKROBootKeyboardReport, page::Keyboard};

use crate::{
    config::{NKRO_MAX_KEYS, NUMBER_COLUMNS, NUMBER_ROWS},
    usb::HID_KEYBOARD_WRITER_N,
};

/// Runs a HID writer task.
#[embassy_executor::task]
pub async fn keyboard_scan_task(
    mut writer: HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_WRITER_N>,
    _write_ring_buffer: WritableRingBuffer<'static, u16>,
    mut read_ring_buffer: ReadableRingBuffer<'static, u16>,
) {
    // Pre‐allocate once
    let mut pressed: Vec<(u8, u8), NKRO_MAX_KEYS> = Vec::new();
    let mut row_buf = [0u16; NUMBER_ROWS];

    loop {
        let _ = read_ring_buffer
            .read_exact(&mut row_buf)
            .await
            .expect("Failed to read from DMA");

        // Get the pressed keys
        for (row, &bits) in row_buf.iter().enumerate() {
            for col in 0..NUMBER_COLUMNS {
                if (bits & (1 << col)) != 0 {
                    // Record the pressed key
                    if pressed.push((row as u8, col as u8)).is_err() {
                        warn!("More than {} keys pressed", NKRO_MAX_KEYS);
                    };
                }
            }
        }
        info!("Pressed keys (row, col): {:?}", pressed);

        // TODO: Convert the key positions to mapped keys
        // pressed
        //     .iter_mut()
        //     .map(|key| key.0 = key.0 + 1)
        //     .collect::<Vec<(u8, u8), NKRO_MAX_KEYS>>();

        // TODO: Send the report to the host
        let report = NKROBootKeyboardReport::new([Keyboard::A]).pack().unwrap();
        match writer.write(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };

        // Clear the pressed keys for next scan
        pressed.clear();
    }
}
