use embassy_net::{Ipv4Address, Ipv4Cidr};

pub mod network_stack;
pub mod utils;
pub mod web_server;

/// Wether to use DHCP to get the IP address or use a static one.
pub const USE_DHCP: bool = false;

// =============================================================================
// DHCP
// =============================================================================
/// Hostname of the device.
pub const HOSTNAME: &str = "wave-rs";
// =============================================================================

// =============================================================================
// Static IP
// =============================================================================
/// IP address of the device.
pub const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new(169, 254, 0, 1), 16);
/// Gateway of the device.
pub const GATEWAY: Option<Ipv4Address> = None;
// =============================================================================
