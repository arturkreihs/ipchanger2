use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Settings {
    idx: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error(transparent)]
    Config(#[from] ConfigError),
}

pub fn load() -> Result<Settings, SettingsError> {
    let settings = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()?
        .try_deserialize::<Settings>()?;
    Ok(settings)
}
