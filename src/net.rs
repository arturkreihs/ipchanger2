use std::collections::HashMap;

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

    pub fn get_addrs(&self) -> Result<Vec<std::net::Ipv4Addr>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let iface = ifaces.get(&self.idx).ok_or(NetError::NotFound)?;
        Ok(iface.ipv4_ips().cloned().collect())
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
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error(transparent)]
    NetWatcher(#[from] netwatcher::Error),
    #[error("iface can't be found")]
    NotFound,
    #[error("can't convert from string")]
    MacConvert,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
