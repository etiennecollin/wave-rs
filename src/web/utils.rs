use defmt::*;
use embassy_futures::select::{select, Either};
use embassy_net::tcp::TcpSocket;
use embassy_time::Timer;

/// Writes a buffer to a TCP socket.
pub async fn write_tcp_buf(socket: &mut TcpSocket<'_>, mut buf: &[u8]) -> Result<(), ()> {
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
pub async fn abort_connection(socket: &mut TcpSocket<'_>) {
    socket.abort();
    let _ = flush_wrapper(socket, 500).await;
}

/// Flush the socket and return an error after `max_time` milliseconds if the
/// socket is not flushed.
pub async fn flush_wrapper(socket: &mut TcpSocket<'_>, max_time: u64) -> Result<(), ()> {
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
