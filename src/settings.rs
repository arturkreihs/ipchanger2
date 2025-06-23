use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use std::io::Write;

const FILENAME: &str = "config.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub mac: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::ser::Error),
}

pub fn load() -> Result<Settings, SettingsError> {
    let settings = Config::builder()
        .add_source(config::File::with_name(FILENAME))
        .build()?
        .try_deserialize::<Settings>()?;
    Ok(settings)
}

pub fn save(s: &Settings) -> Result<(), SettingsError> {
    let mut file = std::fs::File::create(FILENAME)?;
    file.write_all(toml::to_string(s)?.as_bytes())?;
    Ok(())
}
