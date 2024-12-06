use crate::utils::error_handling::{Error, Result};

pub struct IPv4Addr {
    octets: [u8; 4],
}

impl IPv4Addr {
    pub fn wrapping_add(&self, other: &Self) -> Result<Self> {
        Ok(Self {
            octets: self
                .octets
                .into_iter()
                .zip(other.octets)
                .map(|(a, b)| a.wrapping_add(b))
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| anyhow::anyhow!("IPv4 address overflow"))?,
        })
    }

    pub fn wrapping_sub(&self, other: &Self) -> Result<Self> {
        Ok(Self {
            octets: self
                .octets
                .into_iter()
                .zip(other.octets)
                .map(|(a, b)| b.wrapping_sub(a))
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| anyhow::anyhow!("IPv4 address overflow"))?,
        })
    }
}

impl TryFrom<String> for IPv4Addr {
    type Error = Error;

    fn try_from(s: String) -> Result<Self> {
        let addr = s.parse::<std::net::Ipv4Addr>()?;
        Ok(Self {
            octets: addr.octets(),
        })
    }
}

impl From<IPv4Addr> for String {
    fn from(addr: IPv4Addr) -> Self {
        let addr = std::net::Ipv4Addr::from(addr.octets);
        addr.to_string()
    }
}

pub struct IPv6Addr {
    octets: [u16; 8],
}

impl IPv6Addr {
    pub fn xor(&self, other: &Self) -> Self {
        Self {
            octets: self
                .octets
                .into_iter()
                .zip(other.octets)
                .map(|(a, b)| a ^ b)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}

impl TryFrom<String> for IPv6Addr {
    type Error = Error;

    fn try_from(s: String) -> Result<Self> {
        let addr = s.parse::<std::net::Ipv6Addr>()?;
        Ok(Self {
            octets: addr.segments(),
        })
    }
}

impl From<IPv6Addr> for String {
    fn from(addr: IPv6Addr) -> Self {
        let addr = std::net::Ipv6Addr::from(addr.octets);
        addr.to_string()
    }
}
