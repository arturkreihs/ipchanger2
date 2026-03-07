use crate::net::Net;
use anyhow::Result;
use owo_colors::OwoColorize;

use super::COMMANDS_SLICE;

#[ipchanger_macros::command(key = 'h', name = "help", usage = "h", description = "Show this help")]
pub fn help(_: &Net, _: Option<&str>) -> Result<()> {
    println!("Available commands:");
    for cmd in COMMANDS_SLICE {
        println!("{} - {}: {}", cmd.usage.cyan(), cmd.name, cmd.description);
    }
    Ok(())
}
