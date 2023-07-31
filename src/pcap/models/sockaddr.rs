pub use std::net::{Ipv4Addr, Ipv6Addr};
pub use mac_address::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SockAddr {
    hardwareAddress: MacAddress,
    ipv4address: Ipv4address,
    ipv6address: Ipv6address,
}

impl std::fmt::Display for SockAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "mac: {}\nip: {}\nipv6: {}\n", self.hardwareAddress, self.ipv4address, ipv6address)
    }
}
