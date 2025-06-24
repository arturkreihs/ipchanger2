use std::{net::Ipv4Addr, sync::LazyLock};

use anyhow::{Result, anyhow};
use regex::Regex;

use crate::net::Net;

static IP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))/([0-9]|[1-2][0-9]|3[0-2])$")
        .unwrap()
});

pub fn add_addr(net: &Net, param: Option<&str>) -> Result<()> {
    let param = param.ok_or(anyhow!(""))?;
    let (addr, mask) = match IP_REGEX.captures(param) {
        None => {
            return Err(anyhow!("invalid address format, try ip_addr/mask"));
        }
        Some(caps) => match (caps.get(1), caps.get(2)) {
            (Some(addr), Some(mask)) => (addr.as_str(), mask.as_str()),
            _ => {
                return Err(anyhow!(""));
            }
        },
    };

    let addr: Ipv4Addr = addr.parse()?;
    let mask: u8 = mask.parse()?;

    net.add_addr(addr, mask)?;
    Ok(())
}

pub fn del_addr(net: &Net, param: Option<&str>) {}
pub fn list_addrs(net: &Net) {}
