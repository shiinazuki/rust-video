use std::str::FromStr;

use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationConfig,
}

impl AppConfig {}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub sender_email: String,
    pub metadata: String,
    pub notification: String,
    pub user_stats: String,
}

pub fn get_configuration() -> Result<AppConfig, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("crm-server").join("configuration");
    let env_filename = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());
    let configuration_directory = configuration_directory.join(env_filename);

    let configs = Config::builder()
        .add_source(config::File::with_name(
            configuration_directory.to_str().unwrap_or("/etc"),
        ))
        .build()?;

    let app_config = configs.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}

fn deserialize_number_from_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Display,
{
    let port = String::deserialize(deserializer)?;
    let port = port.parse::<T>().map_err(serde::de::Error::custom)?;
    Ok(port)
}
