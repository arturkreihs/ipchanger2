use std::{io, io::Write};

use anyhow::Result;

mod settings;

mod net;
use net::Net;

mod parser;

fn main() -> Result<()> {
    // load settings; show MACs on error
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

    // main loop
    let mut line = String::new();
    loop {
        print!("> ");
        line.clear();

        // reading line
        std::io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        // executing cmd
        let cmd = line.get(0..1);
        let param = line.get(1..);
        let result = match cmd {
            Some("q") => Ok(false),
            Some("a") => parser::add_addr(&net, param).map(|_| true),
            Some("d") => parser::del_addr(&net, param).map(|_| true),
            Some("l") => parser::list_addrs(&net).map(|_| true),
            Some(_) | None => {
                println!("unknown command");
                Ok(true)
            }
        };

        match result {
            Ok(false) => break,
            Ok(true) => continue,
            Err(e) => eprintln!("{e}"),
        }
    }

    Ok(())
}
