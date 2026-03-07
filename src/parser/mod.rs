pub mod parser_common;
mod parser_find;
mod parser_net;

pub use parser_common::help;

use crate::net::Net;
use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

pub(super) static IP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))/([0-9]|[1-2][0-9]|3[0-2])$")
        .unwrap()
});

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