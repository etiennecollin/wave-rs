use defmt::*;
use embassy_stm32::{peripherals::USB_OTG_HS, usb::Driver};
use embassy_usb::{
    class::hid::{self, HidReader, HidReaderWriter, HidWriter, ReportId, RequestHandler, State},
    control::OutResponse,
    Builder,
};
use static_cell::StaticCell;
use usbd_human_interface_device::device::{
    keyboard::NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR, mouse::BOOT_MOUSE_REPORT_DESCRIPTOR,
};

use crate::usb::{
    HID_KEYBOARD_MAX_PACKET_SIZE, HID_KEYBOARD_POLL_MS, HID_KEYBOARD_READER_N,
    HID_KEYBOARD_WRITER_N, HID_MOUSE_MAX_PACKET_SIZE, HID_MOUSE_POLL_MS, HID_MOUSE_WRITER_N,
};

/// Initializes an HID keyboard device.
pub async fn init_hid_keyboard(
    builder: &mut Builder<'static, Driver<'static, USB_OTG_HS>>,
) -> (
    hid::HidReader<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_READER_N>,
    hid::HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_WRITER_N>,
) {
    // Create classes on the builder
    let config = hid::Config {
        report_descriptor: NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
        request_handler: None,
        poll_ms: HID_KEYBOARD_POLL_MS,
        max_packet_size: HID_KEYBOARD_MAX_PACKET_SIZE,
    };

    // Create the hid reader/writer
    static HID_KEYBOARD_STATE: StaticCell<State> = StaticCell::new();
    let hid = HidReaderWriter::<_, HID_KEYBOARD_READER_N, HID_KEYBOARD_WRITER_N>::new(
        builder,
        HID_KEYBOARD_STATE.init(State::new()),
        config,
    );

    // Split the reader and writer
    let (reader, writer) = hid.split();
    (reader, writer)
}

/// Initializes an HID mouse device.
pub async fn init_hid_mouse(
    builder: &mut Builder<'static, Driver<'static, USB_OTG_HS>>,
) -> hid::HidWriter<'static, Driver<'static, USB_OTG_HS>, HID_MOUSE_WRITER_N> {
    // Create classes on the builder
    static HID_MOUSE_HANDLER: StaticCell<HIDRequestHandler> = StaticCell::new();
    let config = hid::Config {
        report_descriptor: BOOT_MOUSE_REPORT_DESCRIPTOR,
        request_handler: Some(HID_MOUSE_HANDLER.init(HIDRequestHandler {})),
        poll_ms: HID_MOUSE_POLL_MS,
        max_packet_size: HID_MOUSE_MAX_PACKET_SIZE,
    };

    // Create the writer
    static HID_MOUSE_STATE: StaticCell<State> = StaticCell::new();
    let writer = HidWriter::<_, HID_MOUSE_WRITER_N>::new(
        builder,
        HID_MOUSE_STATE.init(State::new()),
        config,
    );
    writer
}

/// Runs a HID reader task.
#[embassy_executor::task]
pub async fn hid_keyboard_reader_task(
    reader: HidReader<'static, Driver<'static, USB_OTG_HS>, HID_KEYBOARD_READER_N>,
) -> ! {
    let mut request_handler = HIDRequestHandler {};
    reader.run(false, &mut request_handler).await;
}

struct HIDRequestHandler {}

impl RequestHandler for HIDRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}
