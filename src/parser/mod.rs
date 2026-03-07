pub mod parser_common;
mod parser_net;

pub use parser_common::help;

use crate::net::Net;
use anyhow::Result;

pub type CommandFn = fn(&Net, Option<&str>) -> Result<()>;

pub struct Command {
    pub key: char,
    pub name: &'static str,
    pub usage: &'static str,
    pub description: &'static str,
    pub func: CommandFn,
}

#[linkme::distributed_slice]
pub static COMMANDS_SLICE: [Command] = [..];