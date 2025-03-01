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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Key {
    Base(Keyboard),
    Mouse(Mouse),
    Layer(usize),
    Transparent,
    None,
}
