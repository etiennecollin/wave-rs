use core::str::FromStr;

use embassy_stm32::{peripherals::USB_OTG_HS, usb::Driver};
use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    Builder,
};
use static_cell::StaticCell;

/// Initializes a serial logger class.
pub async fn init_serial_logger(
    builder: &mut Builder<'static, Driver<'static, USB_OTG_HS>>,
) -> CdcAcmClass<'static, Driver<'static, USB_OTG_HS>> {
    static STATE_SERIAL_LOGGER: StaticCell<cdc_acm::State> = StaticCell::new();
    let class_serial_logger =
        CdcAcmClass::new(builder, STATE_SERIAL_LOGGER.init(cdc_acm::State::new()), 64);
    class_serial_logger
}

/// Starts a logger task that logs messages to the serial logger class via USB.
#[embassy_executor::task]
pub async fn usb_serial_logger_task(class: CdcAcmClass<'static, Driver<'static, USB_OTG_HS>>) {
    embassy_usb_logger::with_custom_style!(
        1024,
        log::LevelFilter::from_str(env!("DEFMT_LOG")).unwrap(),
        class,
        |record, writer| {
            use core::fmt::Write;
            let level = record.level().as_str();
            let content = record.args();
            let file = record.file().unwrap_or("");
            let line = record.line().unwrap_or(0);
            write!(writer, "╭[{level}] {content}\r\n╰{file}:{line}\r\n").unwrap();
        }
    )
    .await;
}
