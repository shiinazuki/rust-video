use std::{fs, path::Path};

use axum::http::Method;
use serde::{Deserialize, Deserializer};
use anyhow::Result;

use crate::ProjectRoutes;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub routes: ProjectRoutes,
}

impl ProjectConfig {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(filename)?;
        let config: ProjectConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectRoute {
    #[serde(deserialize_with = "deserialize_method")]
    pub method: Method,
    pub handler: String,
}

fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_uppercase().as_str() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        "CONNECT" => Ok(Method::CONNECT),
        "TRACE" => Ok(Method::TRACE),
        _ => Err(serde::de::Error::custom("invalid method")),
    }
}
