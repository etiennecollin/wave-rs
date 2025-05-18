use defmt::*;
use embassy_stm32::{
    dma::{AnyChannel, Request, Transfer, TransferOptions},
    exti::ExtiInput,
    peripherals::USB_OTG_HS,
    time::Hertz,
    usb::Driver,
    Peri,
};
use embassy_usb::class::hid::HidWriter;

use crate::{
    config::{
        GPIO_PORT_COLUMNS, GPIO_PORT_ROWS, LAYOUT, NKRO_MAX_KEYS, NUMBER_COLUMNS, NUMBER_ROWS,
    },
    usb::HID_KEYBOARD_WRITER_N,
    utils::{dma::DmaTimer, DmaTimerPeripheral},
};

/// Runs a HID writer task.
#[embassy_executor::task]
pub async fn keyboard_scan_task(
    mut writer: HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_WRITER_N>,
    mut button: ExtiInput<'static>,
    dma_timer: Peri<'static, DmaTimerPeripheral>,
    dma_write_channel: Peri<'static, AnyChannel>,
    dma_read_channel: Peri<'static, AnyChannel>,
) {
    let mut timer = DmaTimer::new(dma_timer);
    timer.configure(Hertz(1000), 1, 1000);

    // Initialize the buffers
    // The read buffer is used to read from the GPIO registers.
    let mut read_buffer = [0u16; NUMBER_COLUMNS];

    // The write buffer is used to write to the GPIO registers
    // This is will enable each GPIO column pin one by one using a "bit mask"
    let mut write_buffer = [0u16; NUMBER_COLUMNS];
    write_buffer.iter_mut().enumerate().for_each(|(i, x)| {
        *x = 1 << i as u16;
    });

    // TODO: Find a way to bind the DMA transfers to the timer channels.
    //
    // Using timer, schedule write/read operations from the DMA:
    // - When timer == 1: Trigger GPDMA write
    // - When timer == 1000: Trigger GPDMA read
    //
    // Enable HTIE and CTIE flags on GPDMA channel
    // On GPDMA read Half/Full transfer interrupt:
    // - Read from DMA ringbuffer
    //

    // Configure the DMA channel to write to the GPIO registers
    let transfer_write = unsafe {
        Transfer::new_write(
            dma_write_channel,
            Request::default(),
            &write_buffer,
            GPIO_PORT_COLUMNS.idr().as_ptr() as *mut u16,
            TransferOptions::default(),
        )
    };

    // Configure the DMA channel to read from the GPIO registers
    let transfer_read = unsafe {
        Transfer::new_read(
            dma_read_channel,
            Request::default(),
            GPIO_PORT_ROWS.odr().as_ptr() as *mut u16,
            &mut read_buffer,
            TransferOptions::default(),
        )
    };

    // let mut read_buffer_tmp = [0u16; NUMBER_COLUMNS];
    // loop {
    //     transfer_read.await;
    //     read_buffer.copy_from_slice(&read_buffer);
    // }

    // loop {
    //     button.wait_for_high().await;
    //     info!("Button pressed!");
    //     // Create a report with the A key pressed. (no shift modifier)
    //     let report = NKROBootKeyboardReport::new([Keyboard::A]).pack().unwrap();
    //     match writer.write(&report).await {
    //         Ok(()) => {}
    //         Err(e) => warn!("Failed to send report: {:?}", e),
    //     };
    //
    //     button.wait_for_low().await;
    //     info!("Button released!");
    //     let report = NKROBootKeyboardReport::default().pack().unwrap();
    //     match writer.write(&report).await {
    //         Ok(()) => {}
    //         Err(e) => warn!("Failed to send report: {:?}", e),
    //     };
    // }
}

// use heapless::Vec;
// use packed_struct::PackedStruct; // To pack the keyboard reports.
// use usbd_human_interface_device::{device::keyboard::NKROBootKeyboardReport, page::Keyboard};
// use super::keys::Key;
//
// fn get_matrix(read_buffer: [u16; NUMBER_COLUMNS]) -> [Key; NKRO_MAX_KEYS] {
//     let mut pressed_keys = [Key::None; NKRO_MAX_KEYS];
//
//     for (col, &row_bits) in read_buffer.iter().enumerate() {
//         for row in 0..NUMBER_ROWS {
//             if (row_bits & (1 << row)) != 0 {
//                 let key = LAYOUT.get_key(row, col);
//
//                 // Check if this key is a layer switch key
//                 if LAYOUT.is_layer_switch_key(key) {
//                     LAYOUT.set_layer(key);
//                     layer_changed = true;
//                 }
//             }
//         }
//     }
//
//     // If the layer was changed, reprocess the matrix with the updated layer
//     for (col, &row_bits) in read_buffer.iter().enumerate() {
//         for row in 0..NUMBER_ROWS {
//             if (row_bits & (1 << row)) != 0 {
//                 let key = LAYOUT.get_key(row, col);
//
//                 if !LAYOUT.is_layer_switch_key(key) {
//                     pressed_keys[i] = key;
//                     i += 1;
//                     if i >= NKRO_MAX_KEYS {
//                         return pressed_keys;
//                     }
//                 }
//             }
//         }
//     }
//
//     pressed_keys
// }
