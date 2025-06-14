use anyhow::Result;

mod settings;

mod net;
use net::Net;

fn main() -> Result<()> {
    // settings::load()
    for (idx, mac) in Net::list()? {
        let mac = mac.iter().fold(String::new(), |mut acc, &byte| {
            acc.push_str(&format!("{:02x}", byte));
            acc
        });
        println!("{idx:4} {mac}");
    }
    Ok(())
}
