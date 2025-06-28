use std::{io, io::Write};

use anyhow::Result;
use owo_colors::OwoColorize;

mod settings;

mod net;
use net::Net;

mod parser;

fn main() -> Result<()> {
    enable_ansi_support();

    // load settings; show MACs on error
    let settings = match settings::load() {
        Ok(s) => s,
        Err(_) => {
            settings::save(&settings::Settings::default())?;

            // print interfaces and MACs
            for (_, (mac, name)) in Net::list_ifaces()? {
                let mac = mac.iter().fold(String::new(), |mut acc, &byte| {
                    acc.push_str(&format!("{:02x}", byte));
                    acc
                });
                println!("{mac} - {name}");
            }
            return Ok(());
        }
    };

    let net = Net::new(&settings.mac)?;

    // main loop
    let mut line = String::new();
    loop {
        print!("{}", "ipchanger> ".yellow());
        line.clear();

        // reading line
        std::io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        // executing cmd
        let cmd = line.get(0..1);
        let param = line.get(1..).filter(|&p| !p.is_empty());
        let result = match cmd {
            None => continue,
            Some("q") => Ok(false),
            Some(cmd) => {
                let func = match cmd {
                    "l" => parser::list_addrs,
                    "a" => parser::add_addr,
                    "d" => parser::del_addr,
                    _ => parser::help,
                };
                func(&net, param).map(|_| true)
            }
        };

        match result {
            Ok(false) => break,
            Ok(true) => continue,
            Err(e) => eprintln!("{} {}", "Error:".cyan(), e.red()),
        }
    }

    Ok(())
}

fn enable_ansi_support() {
    use winapi::um::{
        consoleapi::{GetConsoleMode, SetConsoleMode},
        processenv::GetStdHandle,
        winbase::STD_OUTPUT_HANDLE,
        wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    };

    unsafe {
        let std_out = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode = 0;
        GetConsoleMode(std_out, &mut mode);
        SetConsoleMode(std_out, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
}
