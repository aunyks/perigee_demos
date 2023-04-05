use crate::config::{PlayerConfig, RaycastVehicleConfig};
use perigee::{
    config::PhysicsConfig,
    toml,
    traits::{TryFromToml, TryToToml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Level1Config {
    pub launch_impulse: f32,
    #[serde(default)]
    pub physics: PhysicsConfig,
    #[serde(default)]
    pub player: PlayerConfig,
    #[serde(default)]
    pub car: RaycastVehicleConfig,
}

impl Default for Level1Config {
    fn default() -> Self {
        Self {
            launch_impulse: 25.0,
            physics: PhysicsConfig::default(),
            player: PlayerConfig::default(),
            car: RaycastVehicleConfig::default(),
        }
    }
}

impl TryFromToml for Level1Config {
    fn try_from_toml(toml_str: &str) -> Result<Self, String> {
        match toml::from_str::<Level1Config>(toml_str) {
            Ok(config) => Ok(config),
            Err(toml_de_err) => Err(toml_de_err.to_string()),
        }
    }
}

impl TryToToml for Level1Config {
    fn try_to_toml(&self) -> Result<String, String> {
        match toml::to_string(self) {
            Ok(config_toml) => Ok(config_toml),
            Err(toml_ser_err) => Err(toml_ser_err.to_string()),
        }
    }
}
