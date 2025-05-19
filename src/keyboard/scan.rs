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
    config::{MATRIX_COLUMNS_NUMBER, MATRIX_ROWS_NUMBER, NKRO_MAX_KEYS},
    usb::HID_KEYBOARD_WRITER_N,
};

/// Runs a HID writer task.
#[embassy_executor::task]
pub async fn keyboard_scan_task(
    mut _writer: HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_WRITER_N>,
    mut write_ring_buffer: WritableRingBuffer<'static, u32>,
    mut read_ring_buffer: ReadableRingBuffer<'static, u32>,
) {
    // Start the DMA
    write_ring_buffer.start();
    read_ring_buffer.start();

    // Pre‐allocate once
    let mut pressed: Vec<(u8, u8), NKRO_MAX_KEYS> = Vec::new();
    let mut row_buf = [0; MATRIX_COLUMNS_NUMBER];

    loop {
        let _ = read_ring_buffer
            .read_exact(&mut row_buf)
            .await
            .expect("Failed to read from DMA");

        // Get the pressed keys
        for (col, &bits) in row_buf.iter().enumerate() {
            for row in 0..MATRIX_ROWS_NUMBER {
                if (bits & (1 << row)) != 0 {
                    info!("PA{}, PB{}", col, row);
                    // Record the pressed key
                    if pressed.push((row as u8, col as u8)).is_err() {
                        warn!("More than {} keys pressed", NKRO_MAX_KEYS);
                    };
                }
            }
        }

        // TODO: Convert the key positions to mapped keys
        // pressed
        //     .iter_mut()
        //     .map(|key| key.0 = key.0 + 1)
        //     .collect::<Vec<(u8, u8), NKRO_MAX_KEYS>>();

        // TODO: Send the report to the host
        // let report = NKROBootKeyboardReport::new([Keyboard::A]).pack().unwrap();
        // match writer.write(&report).await {
        //     Ok(()) => {}
        //     Err(e) => warn!("Failed to send report: {:?}", e),
        // };

        // Clear the pressed keys for next scan
        pressed.clear();
    }
}
