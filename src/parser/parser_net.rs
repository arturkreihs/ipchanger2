use crate::net::Net;
use anyhow::{Result, anyhow, bail};
use owo_colors::OwoColorize;
use std::{
    net::Ipv4Addr,
    sync::Mutex,
};

use super::IP_REGEX;

static IP_CACHE: Mutex<Vec<Ipv4Addr>> = Mutex::new(vec![]);

#[ipchanger_macros::command(key = 'a', name = "add", usage = "a<ip/mask>", description = "Add IPv4 address in CIDR notation to the interface")]
pub fn add_addr(net: &Net, param: Option<&str>) -> Result<()> {
    let param = param.ok_or(anyhow!("no address provided"))?;
    let (addr, mask) = match IP_REGEX.captures(param) {
        None => bail!("invalid address format, try ip_addr/mask"),
        Some(caps) => match (caps.get(1), caps.get(2)) {
            (Some(addr), Some(mask)) => (addr.as_str(), mask.as_str()),
            _ => bail!("addr or mask is invalid"),
        },
    };
    let addr: Ipv4Addr = addr.parse()?;
    let mask: u8 = mask.parse()?;
    net.add_addr(&addr, mask)?;
    list_addrs(net, None)?;
    Ok(())
}

#[ipchanger_macros::command(key = 'd', name = "delete", usage = "d<index>", description = "Delete IPv4 address by its index from the list")]
pub fn del_addr(net: &Net, param: Option<&str>) -> Result<()> {
    let idx: u8 = param.ok_or(anyhow!("no address index provided"))?.parse()?;
    if idx == 0 {
        bail!("idx is 0");
    }
    let idx = idx - 1;
    let ip = {
        let ip = IP_CACHE
            .lock()
            .map_err(|_| anyhow!("cannot lock IP_CACHE for deleting"))?;
        *ip
            .get(idx as usize)
            .ok_or(anyhow!("cannot get address by id"))?
    };
    net.del_addr(&ip)?;
    list_addrs(net, None)?;
    Ok(())
}

#[ipchanger_macros::command(key = 'l', name = "list", usage = "l", description = "List IPv4 addresses on the interface")]
pub fn list_addrs(net: &Net, param: Option<&str>) -> Result<()> {
    if param.is_some() {
        bail!("list command shouldn't have any arguments");
    }
    IP_CACHE
        .lock()
        .map_err(|_| anyhow!("cannot lock IP_CACHE for clearing"))?
        .clear();
    for (idx, addr) in net.get_addrs()?.iter().enumerate() {
        let mask = addr
            .1
            .map(|m| m.to_string())
            .or_else(|| Some("?".into()))
            .map(|m| {
                let mut m = m;
                m.insert(0, '/');
                m
            }).ok_or(anyhow!("can't get mask"))?;
        println!("{} - {}{}", (idx + 1).cyan(), addr.0, mask.bright_black());
        IP_CACHE
            .lock()
            .map_err(|_| anyhow!("cannot lock IP_CACHE for pushing"))?
            .push(addr.0);
    }
    Ok(())
}
