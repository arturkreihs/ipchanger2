use crate::net::Net;
use anyhow::{Result, anyhow, bail};
use owo_colors::OwoColorize;
use std::{
    net::Ipv4Addr,
    sync::Mutex,
};

use super::IP_REGEX;

#[ipchanger_macros::command(key = 's', name = "switch", usage = "s", description = "Switch management to the other configured network card")]
pub fn switch(net: &mut Net, _: Option<&str>) -> Result<()> {
    net.switch()?;
    println!("Now managing {} ({})", net.name, net.idx);
    list_addrs(net, None)?; // refresh view on the new interface
    Ok(())
}

static IP_CACHE: Mutex<Vec<Ipv4Addr>> = Mutex::new(vec![]);
static DELETED_CACHE: Mutex<Vec<(Ipv4Addr, Option<u8>)>> = Mutex::new(vec![]);

/// Re-add a deleted address by its display index (0-based among deleted entries).
fn readd_deleted(net: &mut Net, idx: usize) -> Result<()> {
    let (ip, mask) = {
        let cache = DELETED_CACHE
            .lock()
            .map_err(|_| anyhow!("cannot lock DELETED_CACHE"))?;
        cache
            .get(idx)
            .ok_or(anyhow!("invalid deleted address index"))?
            .clone()
    };
    net.add_addr(&ip, mask.unwrap_or(24))?;
    net.remove_deleted(idx as u8)?;
    Ok(())
}

#[ipchanger_macros::command(key = 'a', name = "add", usage = "a<ip/mask> or a<index>", description = "Add IPv4 address in CIDR notation, or re-add a deleted address by index")]
pub fn add_addr(net: &mut Net, param: Option<&str>) -> Result<()> {
    let param = param.ok_or(anyhow!("no address provided"))?;

    // if param is a plain number, try to re-add a deleted address
        if !param.contains('/') && !param.contains('.') {
            let idx: usize = param.parse()?;
            if idx == 0 || idx == 1 {
                bail!("index {} is invalid", idx);
            }
            let active_count = net.get_addrs()?.len();
            // deleted indices start after active addresses
            let del_idx = idx
                .checked_sub(active_count + 1)
                .ok_or(anyhow!("index {} is not a deleted address index", idx))?;
            readd_deleted(net, del_idx)?;
            list_addrs(net, None)?;
            return Ok(());
        }

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
pub fn del_addr(net: &mut Net, param: Option<&str>) -> Result<()> {
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
pub fn list_addrs(net: &mut Net, param: Option<&str>) -> Result<()> {
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
    // cache and display deleted addresses with continuation indices
        {
            let base_idx = net.get_addrs()?.len() + 1;
            let mut cache = DELETED_CACHE
                .lock()
                .map_err(|_| anyhow!("cannot lock DELETED_CACHE for clearing"))?;
            cache.clear();
            for (idx, (ip, mask)) in net.get_deleted_addrs()?.into_iter().enumerate() {
                let mask_str = mask
                    .map(|m| m.to_string())
                    .or_else(|| Some("?".into()))
                    .map(|mut m| {
                        m.insert(0, '/');
                        m
                    })
                    .ok_or(anyhow!("can't get mask"))?;
                let display_idx = base_idx + idx;
                println!(
                    "{} - {}{}",
                    display_idx.red(),
                    ip.red(),
                    mask_str.red()
                );
                cache.push((ip, mask));
            }
        }
    Ok(())
}
