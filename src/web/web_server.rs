use defmt::*;
use embassy_net::{tcp::TcpSocket, Stack};

use crate::{
    usb::SERVER_PORT,
    web::utils::{abort_connection, write_tcp_buf},
};

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

        match stack.config_v4() {
            None => {
                defmt::warn!("HTTP | Stack has no IP");
                continue;
            }
            Some(test) => {
                defmt::info!("HTTP | Stack has IP {:?}", test.address);
            }
        };

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
