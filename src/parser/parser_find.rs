use std::{net::Ipv4Addr, time::Duration};

use crate::net::Net;
use anyhow::{Result, anyhow, bail};
use ipnetwork::Ipv4Network;
use pinger::Pinger;
use owo_colors::OwoColorize;

const DEFAULT_MASK: u8 = 24;

#[ipchanger_macros::command(key = 'c', name = "check", usage = "c<ip>", description = "Check IPv4 address")]
pub fn check(net: &Net, param: Option<&str>) -> Result<()> {
    let param = param.ok_or(anyhow!("invalid address"))?;

    // convert partial IP from param to full IP with zero fill
    let mut buf = [0u8; 4];
    partial2full(&mut buf, param)?;
    let input = Ipv4Addr::from_octets(buf);

    // create rank
    let mut rank: Vec<(Ipv4Addr, u8)> = vec![];
    for addr in net.get_addrs()?.iter().map(|i| (i.0, i.1.unwrap_or(DEFAULT_MASK))) {
        let a = Ipv4Network::new(addr.0, addr.1)?.network().octets();
        let b = Ipv4Network::new(input, addr.1)?.network().octets();
        // compare nets only and assign points for rank
        let mut c = 0;
        for i in 0..=3usize {
            if a[i] == b[i] {
                c += 1;
            }
        }
        // merge input with net
        let mut out = buf;
        for idx in 0..=3usize {
            out[idx] |= a[idx];
        }
        // add to the rank
        rank.push((Ipv4Addr::from_octets(out), c));
    }

    let addr = rank.iter()
        .max_by_key(|(_, n)| *n)
        .map(|(ip, _)| *ip)
        .ok_or(anyhow!("can't find best match"))?;
    println!("{}", addr.cyan());

    // perform ping
    Pinger::new()?
        .set_timeout(Duration::from_secs(1))?
        .ping(addr)?;
    println!("{}", "Success".green());
    Ok(())
}

fn partial2full(buf: &mut [u8; 4], partial: &str) -> Result<()> {
    let mut idx = 3i8;
    for i in partial.split('.').rev().map(|o| o.parse::<u8>().unwrap_or(0)) {
        if idx == -1 {
            bail!("invalid IP address");
        }
        buf[idx as usize] = i;
        idx -= 1;
    }
    // while idx >= 0 {
    //     buf[idx as usize] = 0;
    //     idx -= 1;
    // }
    Ok(())
}

mod test {
    use super::*;

    #[test]
    fn test_partial2full() {
        let mut buf = [0u8; 4];
        partial2full(&mut buf, "1.2.3").unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);
        partial2full(&mut buf, "1.2.3.4").unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);
    }
}