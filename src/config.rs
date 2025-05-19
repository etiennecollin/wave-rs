use embassy_stm32::{
    gpio::{Input, Output},
    pac::{gpio::Gpio, GPIOA, GPIOB},
};
use embassy_sync::once_lock::OnceLock;
use usbd_human_interface_device::page::Keyboard;

use crate::keyboard::{
    keys::Key,
    layers::{Layer, Layers},
};

/// Matrix scanning configuration
pub mod scan {
    use embassy_stm32::time::Hertz;

    /// The frequency at which the matrix is scanned.
    pub const FREQUENCY: Hertz = Hertz(2);
    /// The maximum value for the counter compare register.
    pub const CC_MAX: u32 = 1000;
    /// The counter compare value at which the DMA set the column GPIO pins.
    pub const CC_1: u32 = 500;
    /// The counter compare value at which the DMA reads the row GPIO pins.
    pub const CC_2: u32 = 1000;
}

pub const NKRO_MAX_KEYS: usize = 10;
pub const NUMBER_LAYERS: usize = 1;

pub const MATRIX_COLUMNS_NUMBER: usize = 5;
pub const MATRIX_ROWS_NUMBER: usize = 4;

pub const MATRIX_COLUMNS_GPIO_PORT: Gpio = GPIOA;
pub const MATRIX_ROWS_GPIO_PORT: Gpio = GPIOB;

pub static MATRIX_COLUMNS: OnceLock<[Output<'static>; MATRIX_COLUMNS_NUMBER]> = OnceLock::new();
pub static MATRIX_ROWS: OnceLock<[Input<'static>; MATRIX_ROWS_NUMBER]> = OnceLock::new();

#[rustfmt::skip]
pub const LAYER_1: Layer<MATRIX_ROWS_NUMBER, MATRIX_COLUMNS_NUMBER> = Layer::new([
    [Key::Base(Keyboard::A), Key::Base(Keyboard::B), Key::Base(Keyboard::C), Key::Base(Keyboard::D), Key::Base(Keyboard::E)],
    [Key::Base(Keyboard::F), Key::Base(Keyboard::G), Key::Base(Keyboard::H), Key::Base(Keyboard::I), Key::Base(Keyboard::J)],
    [Key::Base(Keyboard::K), Key::Base(Keyboard::L), Key::Base(Keyboard::M), Key::Base(Keyboard::N), Key::Base(Keyboard::O)],
    [Key::Base(Keyboard::P), Key::Base(Keyboard::Q), Key::Base(Keyboard::R), Key::Base(Keyboard::S), Key::Base(Keyboard::T)],
]);

pub const LAYOUT: Layers<NUMBER_LAYERS, MATRIX_ROWS_NUMBER, MATRIX_COLUMNS_NUMBER> =
    Layers::new([LAYER_1]);
