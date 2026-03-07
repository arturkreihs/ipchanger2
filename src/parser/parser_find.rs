use crate::net::Net;
use anyhow::Result;

#[ipchanger_macros::command(key = 'c', name = "check", usage = "c<ip>", description = "Check IPv4 address")]
pub fn check(_: &Net, _: Option<&str>) -> Result<()> {
    // println!("checking");
    Ok(())
}
