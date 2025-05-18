use embassy_stm32::pac::{gpio::Gpio, GPIOA, GPIOB};
use usbd_human_interface_device::page::Keyboard;

use crate::keyboard::{
    keys::Key,
    layers::{Layer, Layers},
};

pub const NKRO_MAX_KEYS: usize = 10;
pub const NUMBER_COLUMNS: usize = 5;
pub const NUMBER_ROWS: usize = 4;
pub const NUMBER_LAYERS: usize = 1;

pub const GPIO_PORT_COLUMNS: Gpio = GPIOA;
pub const GPIO_PORT_ROWS: Gpio = GPIOB;

#[rustfmt::skip]
pub const LAYER_1: Layer<NUMBER_ROWS, NUMBER_COLUMNS> = Layer::new([
    [Key::Base(Keyboard::A), Key::Base(Keyboard::B), Key::Base(Keyboard::C), Key::Base(Keyboard::D), Key::Base(Keyboard::E)],
    [Key::Base(Keyboard::F), Key::Base(Keyboard::G), Key::Base(Keyboard::H), Key::Base(Keyboard::I), Key::Base(Keyboard::J)],
    [Key::Base(Keyboard::K), Key::Base(Keyboard::L), Key::Base(Keyboard::M), Key::Base(Keyboard::N), Key::Base(Keyboard::O)],
    [Key::Base(Keyboard::P), Key::Base(Keyboard::Q), Key::Base(Keyboard::R), Key::Base(Keyboard::S), Key::Base(Keyboard::T)],
]);

pub const LAYOUT: Layers<NUMBER_LAYERS, NUMBER_ROWS, NUMBER_COLUMNS> = Layers::new([LAYER_1]);
