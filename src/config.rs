use embassy_stm32::{
    gpio::{Input, Output},
    pac::{gpio::Gpio, GPIOA, GPIOB},
};
use embassy_sync::once_lock::OnceLock;
use usbd_human_interface_device::page::Keyboard;

use crate::keyboard::{
    action::{k, KeyAction::Single},
    layers::{Layer, Layers},
};

/// Matrix scanning configuration
pub mod scan {
    use embassy_stm32::time::Hertz;

    /// The frequency at which the full matrix is scanned.
    ///
    /// The effective timer frequency is `FREQUENCY` * [`MATRIX_COLUMNS_NUMBER`](super::MATRIX_COLUMNS_NUMBER).
    pub const FREQUENCY: Hertz = Hertz(4000);
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

pub const LAYER_1: Layer<MATRIX_ROWS_NUMBER, MATRIX_COLUMNS_NUMBER> = Layer::new([
    [
        Single(k(Keyboard::A)),
        Single(k(Keyboard::B)),
        Single(k(Keyboard::C)),
        Single(k(Keyboard::D)),
        Single(k(Keyboard::E)),
    ],
    [
        Single(k(Keyboard::F)),
        Single(k(Keyboard::G)),
        Single(k(Keyboard::H)),
        Single(k(Keyboard::I)),
        Single(k(Keyboard::J)),
    ],
    [
        Single(k(Keyboard::K)),
        Single(k(Keyboard::L)),
        Single(k(Keyboard::M)),
        Single(k(Keyboard::N)),
        Single(k(Keyboard::O)),
    ],
    [
        Single(k(Keyboard::P)),
        Single(k(Keyboard::Q)),
        Single(k(Keyboard::R)),
        Single(k(Keyboard::S)),
        Single(k(Keyboard::T)),
    ],
]);

pub const LAYOUT: Layers<NUMBER_LAYERS, MATRIX_ROWS_NUMBER, MATRIX_COLUMNS_NUMBER> =
    Layers::new([LAYER_1]);
