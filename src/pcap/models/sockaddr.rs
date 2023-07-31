pub use std::net::{Ipv4Addr, Ipv6Addr};
pub use mac_address::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SockAddr {
    hardware: MacAddress,
    ipv4: Ipv4Addr,
    ipv6: Ipv6Addr,
}

impl std::fmt::Display for SockAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "mac: {}\nip: {}\nipv6: {}\n", self.hardware, self.ipv4, self.ipv6)
    }
}
