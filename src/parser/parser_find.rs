use std::time::Duration;

use crate::net::Net;
use anyhow::{anyhow, Result};
use pinger::Pinger;
use owo_colors::OwoColorize;

#[ipchanger_macros::command(key = 'c', name = "check", usage = "c<ip>", description = "Check IPv4 address")]
pub fn check(_: &Net, addr: Option<&str>) -> Result<()> {
    let p = Pinger::new()?
        .set_timeout(Duration::from_secs(1))?;
    p.ping(addr.ok_or(anyhow!("invalid address"))?.parse()?)?;
    println!("{}", "Success".green());
    Ok(())
}