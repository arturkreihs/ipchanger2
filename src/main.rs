use std::{io, io::Write, net::Ipv4Addr};

use anyhow::Result;

mod settings;

mod net;
use net::Net;

mod parser;

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

    let mut line = String::new();
    loop {
        print!("> ");
        line.clear();

        // reading line
        std::io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;

        // executing cmd
        match line.get(0..1) {
            Some("q") => break,
            Some("a") => parser::add_addr(&net, &line),
            Some(_) | None => println!("unknown command"),
        }
    }

    Ok(())
}
