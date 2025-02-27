use core::str::FromStr;

use embassy_futures::select::{select, Either};
use embassy_net::{tcp::TcpSocket, DhcpConfig, Stack, StackResources};
use embassy_stm32::{peripherals::RNG, rng::Rng};
use embassy_time::Timer;
use embassy_usb::class::cdc_ncm::embassy_net::Device;
use static_cell::StaticCell;

use crate::{
    logger::*,
    usb::{HOSTNAME, MTU, SERVER_PORT},
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
    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some(heapless::String::from_str(HOSTNAME).unwrap());
    let dhcp_config = embassy_net::Config::dhcpv4(dhcp_config);

    // Generate random seed
    let mut seed = [0u8; 8];
    rng.async_fill_bytes(&mut seed).await.unwrap();
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, stack_runner) = embassy_net::new(
        eth_device,
        dhcp_config,
        RESOURCES.init(StackResources::new()),
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

/// Runs a web server that listens on a TCP port and logs incoming data.
#[embassy_executor::task]
pub async fn web_server_task(stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("HTTP | Listening on TCP:{}...", SERVER_PORT);
        if let Err(e) = socket.accept(SERVER_PORT).await {
            warn!("Accept error: {:?}", e);
            continue;
        }
        info!(
            "HTTP | Received connection from {:?}",
            socket.remote_endpoint()
        );

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("HTTP | Read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("HTTP | Read error: {:?}", e);
                    break;
                }
            };

            info!("HTTP | Received: {:?}", &buf[..n]);

            if (write_tcp_buf(&mut socket, &buf[..n]).await).is_err() {
                error!("HTTP | Error writing response");
                abort_connection(&mut socket).await;
                break;
            }
        }
    }
}

/// Writes a buffer to a TCP socket.
async fn write_tcp_buf(socket: &mut TcpSocket<'_>, mut buf: &[u8]) -> Result<(), ()> {
    while !buf.is_empty() {
        match socket.write(buf).await {
            Ok(0) => warn!("HTTP | Wrote 0 bytes to the buffer"),
            Ok(n) => buf = &buf[n..],
            Err(_) => return Err(()),
        }
    }

    flush_wrapper(socket, 500).await?;

    Ok(())
}

/// Abort a connection and flush the socket.
async fn abort_connection(socket: &mut TcpSocket<'_>) {
    socket.abort();
    let _ = flush_wrapper(socket, 500).await;
}

/// Flush the socket and return an error if it takes too long
/// `max_time` is in milliseconds
async fn flush_wrapper(socket: &mut TcpSocket<'_>, max_time: u64) -> Result<(), ()> {
    match select(socket.flush(), Timer::after_millis(max_time)).await {
        Either::First(v) => {
            if v.is_err() {
                error!("HTTP | Error flushing socket: {:?}", v);
                return Err(());
            }
        }
        Either::Second(_) => {
            error!("HTTP | Socket took too long to flush");
            return Err(());
        }
    }
    Ok(())
}
