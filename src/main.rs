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
            for (idx, mac) in Net::list()? {
                let mac = mac.iter().fold(String::new(), |mut acc, &byte| {
                    acc.push_str(&format!("{:02x}", byte));
                    acc
                });
                println!("{idx:4} {mac}");
            }
            return Ok(());
        }
    };

    let net = Net::new(settings.idx);
    for ip in net.get_addrs()? {
        println!("{ip:?}");
    }

    Ok(())
}
