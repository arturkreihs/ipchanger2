use std::net::Ipv4Addr;
use std::process::{Command, Output};

use default_net::get_interfaces;

pub struct Net {
    mac1: String,                        // primary interface
    mac2: Option<String>,                // secondary interface (optional)
    pos: usize,                          // 0 or 1, active slot
    pub idx: u32,
    pub name: String,
    mask_db: sled::Db,
    deleted_db: sled::Db,
}

impl Net {
    // ponytail: pos starts at 0; a blank mac1 simply fails to resolve (NotFound).
    pub fn new(mac1: &str, mac2: Option<&str>) -> Result<Self, NetError> {
        let mut net = Net {
            mac1: mac1.to_string(),
            mac2: mac2.map(|s| s.to_string()),
            pos: 0,
            idx: 0,
            name: String::new(),
            mask_db: sled::open("masks")?,
            deleted_db: sled::open("deleted")?,
        };
        net.resolve()?;
        Ok(net)
    }

    // ponytail: only block when there's nothing to switch to (mac2 unset).
    pub fn switch(&mut self) -> Result<(), NetError> {
        let next = self.pos ^ 1;
        if next == 1 && self.mac2.as_deref().is_none_or(|s| s.trim().is_empty()) {
            return Err(NetError::SecondNotConfigured);
        }
        self.pos = next;
        self.resolve()?;
        Ok(())
    }

    fn resolve(&mut self) -> Result<(), NetError> {
        let mac_str = if self.pos == 0 {
            &self.mac1
        } else {
            self.mac2.as_ref().ok_or(NetError::SecondNotConfigured)?
        };
        let mac = Self::parse_mac(mac_str)?;
        let iface = get_interfaces()
            .into_iter()
            .find(|i| i.mac_addr.as_ref().map(|m| m.octets()) == Some(mac))
            .ok_or_else(|| NetError::SecondIfaceNotFound(mac_str.to_string()))?;
        self.idx = iface.index;
        self.name = iface
            .friendly_name
            .clone()
            .unwrap_or_else(|| iface.name.clone());
        Ok(())
    }

    pub fn list_ifaces() -> Result<Vec<(u32, String, [u8; 6])>, NetError> {
        Ok(get_interfaces()
            .into_iter()
            .filter_map(|v| {
                let mac = v.mac_addr.map(|m| m.octets())?;
                let name = v.friendly_name.unwrap_or(v.name);
                Some((v.index, name, mac))
            })
            .collect())
    }

    pub fn get_addrs(&self) -> Result<Vec<(std::net::Ipv4Addr, Option<u8>)>, NetError> {
        let iface = get_interfaces()
            .into_iter()
            .find(|i| i.index == self.idx)
            .ok_or(NetError::NotFound)?;
        let mut vec = iface
            .ipv4
            .iter()
            .filter_map(|n| {
                let mask = self
                    .mask_db
                    .get(n.addr.octets())
                    .unwrap_or(None)
                    .map(|m| m[0]);
                Some((n.addr, mask))
            })
            .collect::<Vec<(Ipv4Addr, Option<u8>)>>();
        vec.sort_by_key(|&(ip, _)| ip);
        Ok(vec)
    }

    pub fn add_addr(&self, addr: &Ipv4Addr, mask: u8) -> Result<(), NetError> {
        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("add")
            .arg("address")
            .arg(format!("name={}", self.name))
            .arg(format!("address={addr}"))
            .arg(format!("mask={}", Self::parse_mask(mask)?))
            .output()
            .and_then(Self::output_handler("netsh"))?;
        let mask = sled::IVec::from(&[mask]);
        self.mask_db.insert(addr.octets(), mask)?;
        self.mask_db.flush()?;
        Ok(())
    }

    pub fn del_addr(&self, addr: &Ipv4Addr) -> Result<(), NetError> {
        // get the mask before removing
        let mask = self.mask_db.get(addr.octets())?.map(|m| m[0]);

        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("delete")
            .arg("address")
            .arg(format!("name={}", self.name))
            .arg(format!("address={addr}"))
            .output()
            .and_then(Self::output_handler("netsh"))?;
        self.mask_db.remove(addr.octets())?;
        self.mask_db.flush()?;

        self.remember_deleted(addr, mask)
    }

    // remember deleted address (up to 3, oldest first)
    fn remember_deleted(&self, addr: &Ipv4Addr, mask: Option<u8>) -> Result<(), NetError> {
        let mut list = self.deleted_raw()?;
        list.extend_from_slice(&addr.octets());
        list.push(mask.unwrap_or(0));
        let excess = list.len().saturating_sub(3 * 5);
        list.drain(..excess);
        self.deleted_db.insert("list", list)?;
        self.deleted_db.flush()?;
        Ok(())
    }

    // ponytail: one key holding 5-byte entries (ip + mask, 0 = unknown); legacy "0".."2"/"cnt" keys ignored.
    fn deleted_raw(&self) -> Result<Vec<u8>, NetError> {
        Ok(self.deleted_db.get("list")?.map(|v| v.to_vec()).unwrap_or_default())
    }

    pub fn get_deleted_addrs(&self) -> Result<Vec<(Ipv4Addr, Option<u8>)>, NetError> {
        Ok(self
            .deleted_raw()?
            .chunks_exact(5)
            .map(|e| (Ipv4Addr::new(e[0], e[1], e[2], e[3]), (e[4] != 0).then_some(e[4])))
            .collect())
    }

    /// Remove a deleted address by its display index (0-based among deleted entries).
    pub fn remove_deleted(&self, display_idx: u8) -> Result<(), NetError> {
        let mut list = self.deleted_raw()?;
        let start = display_idx as usize * 5;
        if start + 5 <= list.len() {
            list.drain(start..start + 5);
            self.deleted_db.insert("list", list)?;
            self.deleted_db.flush()?;
        }
        Ok(())
    }

    // pub fn get_gateway() -> Result<std::net::Ipv4Addr, NetError> {
    //     use default_net::get_default_gateway;
    //     get_default_gateway()
    //         .map(|g| match g.ip_addr {
    //             std::net::IpAddr::V4(ip) => ip,
    //             _ => 0.into(),
    //         })
    //         .map_err(|_| NetError::GetGateway)
    // }

    // pub fn set_gateway(&self, gateway: &Ipv4Addr) -> Result<(), NetError> {
    //     let cmd = |act: &str, args: &str| -> Result<Output, NetError> {
    //         Command::new("netsh")
    //             .arg("interface")
    //             .arg("ip")
    //             .arg(act)
    //             .arg("route")
    //             .arg("0.0.0.0/0")
    //             .arg(self.idx.to_string())
    //             .arg(args)
    //             .output()
    //             .map_err(|_| NetError::SetGateway)
    //     };
    //     cmd("del", "")?;
    //     cmd("add", &gateway.to_string())?;
    //     Ok(())
    // }

    fn output_handler(err: &'static str) -> impl Fn(Output) -> Result<(), std::io::Error> {
        move |o| {
            o.status.success()
                .then_some(())
                .ok_or_else(|| std::io::Error::other(err))
        }
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
    #[error("iface can't be found")]
    NotFound,
    #[error("second interface not configured - set mac2 in config.toml")]
    SecondNotConfigured,
    #[error("second interface MAC '{0}' not found on this system")]
    SecondIfaceNotFound(String),
    #[error("can't convert MAC from string")]
    MacConvert,
    #[error("can't convert Mask from string")]
    MaskConvert,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Sled(#[from] sled::Error),
    // #[error("can't get gateway IP address")]
    // GetGateway,
    // #[error("can't set gateway IP address")]
    // SetGateway,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleted_list_caps_and_removes() {
        let tmp = || sled::Config::new().temporary(true).open().unwrap();
        let net = Net { mac1: String::new(), mac2: None, pos: 0, idx: 0, name: String::new(), mask_db: tmp(), deleted_db: tmp() };
        (1..=4).for_each(|n| net.remember_deleted(&Ipv4Addr::new(10, 0, 0, n), Some(24)).unwrap());
        let ips = |n: &Net| n.get_deleted_addrs().unwrap().iter().map(|a| a.0.octets()[3]).collect::<Vec<_>>();
        assert_eq!(ips(&net), [2, 3, 4]);
        net.remove_deleted(0).unwrap();
        assert_eq!(ips(&net), [3, 4]);
        net.remove_deleted(5).unwrap();
        assert_eq!(ips(&net), [3, 4]);
    }
}
