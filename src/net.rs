use std::net::Ipv4Addr;
use std::process::{Command, Output};

pub struct Net {
    pub idx: u32,
    pub name: String,
    mask_db: sled::Db,
    deleted_db: sled::Db,
}

impl Net {
    pub fn new(mac: &str) -> Result<Self, NetError> {
        let mac = Self::parse_mac(mac)?;
        let list = Self::list_ifaces()?;
        let entry = list
            .iter()
            .find(|&(_, _, v)| *v == mac)
            .ok_or(NetError::NotFound)?;
        let idx = entry.0;
        let name = entry.1.clone();
        let mask_db = sled::open("masks")?;
        let deleted_db = sled::open("deleted")?;
        Ok(Net { idx, name, mask_db, deleted_db })
    }

    pub fn list_ifaces() -> Result<Vec<(u32, String, [u8; 6])>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let ifaces = ifaces
            .into_iter()
            .filter_map(|(_, v)| {
                Self::parse_mac(&v.hw_addr.replace(":", ""))
                    .map(|m| (v.index, v.name, m))
                    .ok()
            })
            .collect::<Vec<(u32, String, [u8; 6])>>();
        Ok(ifaces)
    }

    pub fn get_addrs(&self) -> Result<Vec<(std::net::Ipv4Addr, Option<u8>)>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let iface = ifaces.get(&self.idx).ok_or(NetError::NotFound)?;
        let mut vec = iface
            .ipv4_ips()
            .cloned()
            .map(|ip| {
                let mask = self.mask_db.get(ip.octets()).unwrap_or(None).map(|m| m[0]);
                (ip, mask)
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

        // remember deleted address (up to 3)
        let next_idx = self
            .deleted_db
            .get("cnt")?
            .map(|v| v[0])
            .unwrap_or(0);
        let mut entry = [0u8; 5];
        entry[..4].copy_from_slice(&addr.octets());
        entry[4] = mask.unwrap_or(0);
        self.deleted_db.insert(next_idx.to_string(), &entry)?;
        self.deleted_db.insert("cnt", &[(next_idx + 1) % 3])?;
        self.deleted_db.flush()?;
        Ok(())
    }

    pub fn get_deleted_addrs(&self) -> Result<Vec<(Ipv4Addr, Option<u8>)>, NetError> {
        let mut entries: Vec<(u8, Ipv4Addr, Option<u8>)> = Vec::new();
        for i in 0..3u8 {
            if let Some(raw) = self.deleted_db.get(i.to_string())? {
                let octets: [u8; 4] = raw[..4].try_into().unwrap_or([0; 4]);
                let mask = if raw[4] != 0 { Some(raw[4]) } else { None };
                entries.push((i, Ipv4Addr::from(octets), mask));
            }
        }
        // sort by index to maintain insertion order
        entries.sort_by_key(|&(idx, _, _)| idx);
        Ok(entries.into_iter().map(|(_, ip, m)| (ip, m)).collect())
    }

    /// Remove a deleted address by its display index (0-based among deleted entries).
    /// Compacts the remaining entries and updates the counter.
    pub fn remove_deleted(&self, display_idx: u8) -> Result<(), NetError> {
        let cnt = self
            .deleted_db
            .get("cnt")?
            .map(|v| v[0])
            .unwrap_or(0);
        if display_idx >= cnt || cnt == 0 {
            return Ok(());
        }
        // shift all entries after display_idx down by one
        for i in display_idx..(cnt.saturating_sub(1)) {
            let next = (i + 1).to_string();
            if let Some(val) = self.deleted_db.get(&next)? {
                self.deleted_db.insert(i.to_string(), val)?;
                self.deleted_db.remove(&next)?;
            }
        }
        // clear the last used slot
        let last = (cnt - 1).to_string();
        self.deleted_db.remove(&last)?;
        // update counter
        let new_cnt = cnt - 1;
        self.deleted_db.insert("cnt", &[new_cnt])?;
        self.deleted_db.flush()?;
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
    #[error(transparent)]
    Sled(#[from] sled::Error),
    // #[error("can't get gateway IP address")]
    // GetGateway,
    // #[error("can't set gateway IP address")]
    // SetGateway,
}
