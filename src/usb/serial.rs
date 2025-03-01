use defmt::{info, panic};
use embassy_stm32::{
    peripherals::USB_OTG_HS,
    usb::{Driver, Instance},
};
use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    driver::EndpointError,
    Builder,
};
use static_cell::StaticCell;

/// Initializes a serial class.
pub async fn init_serial(
    builder: &mut Builder<'static, Driver<'static, USB_OTG_HS>>,
) -> CdcAcmClass<'static, Driver<'static, USB_OTG_HS>> {
    static STATE_SERIAL: StaticCell<cdc_acm::State> = StaticCell::new();
    let class_serial = CdcAcmClass::new(builder, STATE_SERIAL.init(cdc_acm::State::new()), 64);
    class_serial
}

/// Runs a USB serial stack.
///
/// It waits for a connection, reads incoming data, echos it back, and then waits for a new connection.
#[embassy_executor::task]
pub async fn usb_serial_task(mut class: CdcAcmClass<'static, Driver<'static, USB_OTG_HS>>) {
    loop {
        class.wait_connection().await;
        info!("SERIAL | Connected");
        match echo(&mut class).await {
            Ok(_) => {}
            Err(EndpointError::Disabled) => {}
            Err(EndpointError::BufferOverflow) => panic!("SERIAL | Buffer overflow"),
        };
        info!("SERIAL | Disconnected");
    }
}

/// Reads data from a serial connection and echos it back.
async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), EndpointError> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("SERIAL | Data: {:?}", data);
        class.write_packet(data).await?;
    }
}
