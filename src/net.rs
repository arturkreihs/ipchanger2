use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::Command;

pub struct Net {
    idx: u32,
}

impl Net {
    pub fn new(mac: &str) -> Result<Self, NetError> {
        let mac = Self::parse_mac(mac)?;
        let list = Self::list()?;
        let entry = list
            .iter()
            .find(|&(_, v)| v == &mac)
            .ok_or(NetError::NotFound)?;
        let idx = *entry.0;
        Ok(Net { idx })
    }

    pub fn list() -> Result<HashMap<u32, [u8; 6]>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let ifaces = ifaces
            .into_iter()
            .filter_map(|(k, v)| {
                Self::parse_mac(&v.hw_addr.replace(":", ""))
                    .map(|m| (k, m))
                    .ok()
            })
            .collect::<HashMap<u32, [u8; 6]>>();
        Ok(ifaces)
    }

    pub fn get_addrs(&self) -> Result<Vec<std::net::Ipv4Addr>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let iface = ifaces.get(&self.idx).ok_or(NetError::NotFound)?;
        Ok(iface.ipv4_ips().cloned().collect())
    }

    pub fn add_addr(&self, addr: Ipv4Addr, mask: u8) -> Result<(), NetError> {
        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("add")
            .arg("address")
            .arg(format!("name={}", self.idx))
            .arg(format!("address={}", addr))
            .arg(format!("mask={}", Self::parse_mask(mask)?))
            .output()?;
        Ok(())
    }

    fn parse_mac(mac: &str) -> Result<[u8; 6], NetError> {
        if mac.len() != 12 {
            return Err(NetError::MacConvert);
        }
        let mac = (0..6)
            .map(|i| &mac[i * 2..i * 2 + 2])
            .filter_map(|i| u8::from_str_radix(i, 16).ok())
            .collect::<Vec<u8>>();
        mac.try_into().map_err(|_| NetError::MacConvert)
    }

    fn parse_mask(mask: u8) -> Result<Ipv4Addr, NetError> {
        if mask > 32 {
            return Err(NetError::MaskConvert);
        }
        let mask = if mask == 0 {
            0u32
        } else {
            !((1u32 << (32 - mask)) - 1)
        };
        Ok(Ipv4Addr::from_bits(mask))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    NetWatcher(#[from] netwatcher::Error),
    #[error("iface can't be found")]
    NotFound,
    #[error("can't convert MAC from string")]
    MacConvert,
    #[error("can't convert Mask from string")]
    MaskConvert,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
