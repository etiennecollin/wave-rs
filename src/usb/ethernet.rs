use embassy_stm32::{peripherals::USB_OTG_HS, usb::Driver};
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner};
use embassy_usb::{
    class::cdc_ncm::{self, CdcNcmClass},
    Builder,
};
use static_cell::StaticCell;

use crate::usb::{HOST_MAC_ADDR, MTU, OUR_MAC_ADDR};

/// Initializes an ethernet device.
pub async fn init_ethernet(
    builder: &mut Builder<'static, Driver<'static, USB_OTG_HS>>,
) -> (
    Runner<'static, Driver<'static, USB_OTG_HS>, MTU>,
    Device<'static, MTU>,
) {
    // Create the usb-ethernet class
    static STATE_ETH: StaticCell<cdc_ncm::State> = StaticCell::new();
    let class_eth = CdcNcmClass::new(
        builder,
        STATE_ETH.init(cdc_ncm::State::new()),
        HOST_MAC_ADDR,
        64,
    );

    // Create the network runner
    static NET_STATE: StaticCell<cdc_ncm::embassy_net::State<MTU, 4, 4>> = StaticCell::new();
    let (eth_runner, eth_device) = class_eth.into_embassy_net_device::<MTU, 4, 4>(
        NET_STATE.init(cdc_ncm::embassy_net::State::new()),
        OUR_MAC_ADDR,
    );

    (eth_runner, eth_device)
}

/// Runs a USB ethernet stack.
#[embassy_executor::task]
pub async fn usb_ethernet_task(runner: Runner<'static, Driver<'static, USB_OTG_HS>, MTU>) -> ! {
    runner.run().await
}
