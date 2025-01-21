use anyhow::Result;
use config::Config;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub database: DatabaseConfig,
}

impl AppConfig {}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretBox<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
        )))
    }
}

pub fn get_configuration() -> Result<AppConfig, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("chat").join("configuration");
    let env_filename = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());
    let configuration_directory = configuration_directory.join(env_filename);

    let configs = Config::builder()
        .add_source(config::File::with_name(
            configuration_directory.to_str().unwrap_or_else(|| "/etc"),
        ))
        .build()?;
    Ok(configs.try_deserialize::<AppConfig>()?)
}

fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let port = String::deserialize(deserializer)?;
    let port = port.parse::<u16>().map_err(serde::de::Error::custom)?;
    Ok(port)
}
