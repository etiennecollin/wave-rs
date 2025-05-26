use cortex_m::singleton;
use usbd_human_interface_device::page::Keyboard;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Mouse {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    SpeedUp,
    SpeedDown,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    Mouse(Mouse),
    Keyboard(Keyboard),
}

/// Shortcut for creating a mouse action.
pub const fn m(key: Mouse) -> Action {
    Action::Mouse(key)
}

/// Shortcut for creating a keyboard action.
pub const fn k(key: Keyboard) -> Action {
    Action::Keyboard(key)
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KeyAction {
    NoOp,
    Transparent,
    Single(Action),
    Layer(usize),
    DefaultLayer(usize),
    HoldTap(HoldTapAction),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HoldTapConfig {
    Default,
    HoldOnOtherKeyPress,
    PermissiveHold,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HoldTapAction {
    pub hold: Action,
    pub tap: Action,
    pub config: HoldTapConfig,
}
