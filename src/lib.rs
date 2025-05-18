#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod config;
pub mod keyboard;
pub mod usb;
pub mod utils;
pub mod web;

use embassy_stm32::{
    bind_interrupts,
    peripherals::{RNG, USB_OTG_HS},
};

// Bind interrupts for the required peripherals.
bind_interrupts!(pub struct Irqs {
    OTG_HS => embassy_stm32::usb::InterruptHandler<USB_OTG_HS>;
    RNG => embassy_stm32::rng::InterruptHandler<RNG>;
});
