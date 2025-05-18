use embassy_stm32::gpio::AnyPin;
use embassy_time::Instant;

pub struct Debouncer {
    pin: AnyPin,
    is_debounced: bool,
    timestamp: Instant,
}
