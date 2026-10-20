use std::io::{self, BufRead, Write};

use anyhow::{anyhow, Result};
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
            for (_, name, mac) in Net::list_ifaces()? {
                let mac = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                println!("{mac} - {name}");
            }
            println!("\nPress Enter to exit...");
            io::stdin().lock().read_line(&mut String::new())?;
            return Ok(());
        }
    };

    let mut net = Net::new(&settings.mac1, settings.mac2.as_deref())?;
    // println!("Network index = {}", net.idx);

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
                let ch = cmd.chars().next().ok_or(anyhow!("can't get character"))?;
                let func = parser::COMMANDS_SLICE
                    .iter()
                    .find(|&c| c.key == ch)
                    .map(|c| c.func)
                    .unwrap_or(parser::help);
                func(&mut net, param).map(|_| true)
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
