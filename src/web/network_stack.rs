use core::str::FromStr;

use embassy_net::{DhcpConfig, Stack, StackResources, StaticConfigV4};
use embassy_stm32::{peripherals::RNG, rng::Rng};
use embassy_usb::class::cdc_ncm::embassy_net::Device;
use heapless::Vec;
use static_cell::StaticCell;

use crate::{
    usb::MTU,
    web::{GATEWAY, HOSTNAME, IP_ADDRESS, USE_DHCP},
};

/// Initializes a network stack.
pub async fn init_network_stack(
    eth_device: Device<'static, MTU>,
    rng: &mut Rng<'static, RNG>,
) -> (
    Stack<'static>,
    embassy_net::Runner<'static, Device<'static, MTU>>,
) {
    // Configure dhcp
    let network_config;
    if USE_DHCP {
        let mut dhcp_config = DhcpConfig::default();
        dhcp_config.hostname = Some(heapless::String::from_str(HOSTNAME).unwrap());
        network_config = embassy_net::Config::dhcpv4(dhcp_config);
    } else {
        let ip_config = StaticConfigV4 {
            address: IP_ADDRESS,
            gateway: GATEWAY,
            dns_servers: Vec::new(),
        };
        network_config = embassy_net::Config::ipv4_static(ip_config);
    }

    // Generate random seed
    let mut seed = [0u8; 8];
    rng.async_fill_bytes(&mut seed).await.unwrap();
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static NETWORK_STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, stack_runner) = embassy_net::new(
        eth_device,
        network_config,
        NETWORK_STACK_RESOURCES.init(StackResources::new()),
        seed,
    );

    (stack, stack_runner)
}

/// Runs a network stack.
#[embassy_executor::task]
pub async fn network_stack_task(
    mut runner: embassy_net::Runner<'static, Device<'static, MTU>>,
) -> ! {
    runner.run().await
}
