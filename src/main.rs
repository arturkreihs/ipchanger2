use std::net::Ipv4Addr;

use anyhow::Result;

mod settings;

mod net;
use net::Net;

fn main() -> Result<()> {
    let settings = match settings::load() {
        Ok(s) => s,
        Err(_) => {
            settings::save(&settings::Settings::default())?;

            // print interfaces and MACs
            for (_, mac) in Net::list()? {
                let mac = mac.iter().fold(String::new(), |mut acc, &byte| {
                    acc.push_str(&format!("{:02x}", byte));
                    acc
                });
                println!("{mac}");
            }
            return Ok(());
        }
    };

    let net = Net::new(&settings.mac)?;
    // for ip in net.get_addrs()? {
    //     println!("{ip:?}");
    // }
    net.add_addr(Ipv4Addr::new(10, 10, 9, 9), 22)?;

    Ok(())
}
