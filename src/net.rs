use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::{Command, Output};

pub struct Net {
    idx: u32,
    mask_db: sled::Db,
}

impl Net {
    pub fn new(mac: &str) -> Result<Self, NetError> {
        let mask_db = sled::open("masks")?;
        let mac = Self::parse_mac(mac)?;
        let list = Self::list_ifaces()?;
        let entry = list
            .iter()
            .find(|&(_, v)| v.0 == mac)
            .ok_or(NetError::NotFound)?;
        let idx = *entry.0;
        Ok(Net { idx, mask_db })
    }

    pub fn list_ifaces() -> Result<HashMap<u32, ([u8; 6], String)>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let ifaces = ifaces
            .into_iter()
            .filter_map(|(k, v)| {
                Self::parse_mac(&v.hw_addr.replace(":", ""))
                    .map(|m| (k, (m, v.name)))
                    .ok()
            })
            .collect::<HashMap<u32, ([u8; 6], String)>>();
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
            .arg(format!("name={}", self.idx))
            .arg(format!("address={addr}"))
            .arg(format!("mask={}", Self::parse_mask(mask)?))
            .output()?;
        let mask = sled::IVec::from(&[mask]);
        self.mask_db.insert(addr.octets(), mask)?;
        self.mask_db.flush()?;
        Ok(())
    }

    pub fn del_addr(&self, addr: &Ipv4Addr) -> Result<(), NetError> {
        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("delete")
            .arg("address")
            .arg(format!("name={}", self.idx))
            .arg(format!("address={addr}"))
            .output()?;
        self.mask_db.remove(addr.octets())?;
        self.mask_db.flush()?;
        Ok(())
    }

    pub fn get_gateway() -> Result<std::net::Ipv4Addr, NetError> {
        use default_net::get_default_gateway;
        get_default_gateway()
            .map(|g| match g.ip_addr {
                std::net::IpAddr::V4(ip) => ip,
                _ => 0.into(),
            })
            .map_err(|_| NetError::GetGateway)
    }

    pub fn set_gateway(&self, gateway: &Ipv4Addr) -> Result<(), NetError> {
        let cmd = |act: &str, args: &str| -> Result<Output, NetError> {
            Command::new("netsh")
                .arg("interface")
                .arg("ip")
                .arg(act)
                .arg("route")
                .arg("0.0.0.0/0")
                .arg(self.idx.to_string())
                .arg(args)
                .output()
                .map_err(|_| NetError::SetGateway)
        };
        cmd("del", "")?;
        cmd("add", &gateway.to_string())?;
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
    #[error(transparent)]
    Sled(#[from] sled::Error),
    #[error("can't get gateway IP address")]
    GetGateway,
    #[error("can't set gateway IP address")]
    SetGateway,
}
