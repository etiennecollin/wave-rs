#[cfg(not(feature = "log-serial"))]
pub use defmt::*;
#[cfg(feature = "log-serial")]
pub use log::*;
