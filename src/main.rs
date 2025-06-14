use anyhow::Result;

mod net;

fn main() -> Result<()> {
    println!("Hello, world!");
    net::Net::list()?;
    Ok(())
}
