use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub taskwarrior: TaskwarriorSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub log_directive: String,
}

#[derive(Deserialize)]
pub struct TaskwarriorSettings {
    pub data_location: Option<String>,
    pub taskrc: Option<String>,
}

impl Settings {
    pub fn new_from_file(file: Option<String>) -> Result<Self, ConfigError> {
        let config_file_required = file.is_some();
        let config_file =
            file.unwrap_or_else(|| env::var("CONFIG").unwrap_or_else(|_| "dev".into()));

        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(File::with_name(&config_file).required(config_file_required))
            .add_source(Environment::with_prefix("cs"))
            .build()?;

        config.try_deserialize()
    }

    pub fn new() -> Result<Self, ConfigError> {
        Settings::new_from_file(None)
    }
}
