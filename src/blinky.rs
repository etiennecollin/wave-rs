use core::cell::RefCell;

use embassy_stm32::{exti::ExtiInput, gpio::Output};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::Timer;

static STATUS: CriticalSectionMutex<RefCell<bool>> = CriticalSectionMutex::new(RefCell::new(false));

/// Flashes 3 LEDs or counts in binary on them based on a status.
#[embassy_executor::task]
pub async fn blinky(mut leds: [Output<'static>; 3]) {
    let mut counter: u8 = 0;
    loop {
        Timer::after_millis(1).await;
        if is_status_high() {
            // Count in binary on 3 LEDs
            if counter > 7 {
                counter = 0;
            }

            // Reverse the order of the LEDs because binary has biggest bit
            // on the left and LEDs are left-to-right in the array.
            leds.iter_mut().rev().enumerate().for_each(|(i, led)| {
                // Get the ith bit of the counter for ith LED
                // Set the LED accordingly
                let current_bit = (counter >> i) & 1;
                if current_bit == 1 {
                    led.set_high();
                } else {
                    led.set_low();
                }
            });
            Timer::after_millis(500).await;

            counter += 1;
        } else {
            counter = 0;
            leds.iter_mut().for_each(|led| {
                led.set_low();
            });

            if is_status_high() {
                continue;
            }

            Timer::after_millis(200).await;
            leds.iter_mut().for_each(|led| {
                led.set_high();
            });
            Timer::after_millis(200).await;
        }
    }
}

/// Listens for a button press and toggles the status.
#[embassy_executor::task]
pub async fn button_listen(mut button: ExtiInput<'static>) {
    loop {
        button.wait_for_rising_edge().await;
        toggle_status();
    }
}

/// Checks if the status is high.
fn is_status_high() -> bool {
    STATUS.lock(|status| *status.borrow())
}

/// Toggles the status.
fn toggle_status() {
    STATUS.lock(|status| {
        let mut status = status.borrow_mut();
        *status = !*status;
    });
}
