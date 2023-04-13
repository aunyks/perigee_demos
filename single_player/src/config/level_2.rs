use crate::config::SedanConfig;
use perigee::{
    config::PhysicsConfig,
    toml,
    traits::{TryFromToml, TryToToml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Level2Config {
    #[serde(default)]
    pub physics: PhysicsConfig,
    #[serde(default)]
    pub car: SedanConfig,
}

impl Default for Level2Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            car: SedanConfig::default(),
        }
    }
}

impl TryFromToml for Level2Config {
    fn try_from_toml(toml_str: &str) -> Result<Self, String> {
        match toml::from_str::<Level2Config>(toml_str) {
            Ok(config) => Ok(config),
            Err(toml_de_err) => Err(toml_de_err.to_string()),
        }
    }
}

impl TryToToml for Level2Config {
    fn try_to_toml(&self) -> Result<String, String> {
        match toml::to_string(self) {
            Ok(config_toml) => Ok(config_toml),
            Err(toml_ser_err) => Err(toml_ser_err.to_string()),
        }
    }
}
