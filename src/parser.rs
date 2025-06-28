use crate::net::Net;
use anyhow::{Result, anyhow, bail};
use owo_colors::OwoColorize;
use regex::Regex;
use std::{net::Ipv4Addr, sync::LazyLock};

static IP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))/([0-9]|[1-2][0-9]|3[0-2])$")
        .unwrap()
});

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
    Ok(())
}

pub fn del_addr(net: &Net, param: Option<&str>) -> Result<()> {
    let idx: u8 = param.ok_or(anyhow!("no address index provided"))?.parse()?;
    if idx == 0 {
        bail!("idx is 0");
    }
    let idx = idx - 1;
    net.del_addr(
        &net.get_addrs()?
            .get(idx as usize)
            .ok_or(anyhow!("getting address"))?
            .0,
    )?;
    Ok(())
}

pub fn list_addrs(net: &Net, param: Option<&str>) -> Result<()> {
    if param.is_some() {
        bail!("list command shouldn't have any arguments");
    }
    for (idx, addr) in net.get_addrs()?.iter().enumerate() {
        let mask = addr
            .1
            .map(|m| m.to_string())
            .or_else(|| Some("?".into()))
            .map(|m| {
                let mut m = m;
                m.insert(0, '/');
                m
            })
            .unwrap();
        println!("{} - {}{}", (idx + 1).cyan(), addr.0, mask.bright_black());
    }
    Ok(())
}

pub fn help(_: &Net, _: Option<&str>) -> Result<()> {
    Ok(())
}
