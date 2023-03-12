use crate::config::{CarConfig, CharacterControllerConfig};
use perigee::{
    config::PhysicsConfig,
    toml,
    traits::{TryFromToml, TryToToml},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Level0Config {
    #[serde(default)]
    level_event_queue_capacity: Option<usize>,
    #[serde(default)]
    physics: PhysicsConfig,
    #[serde(default)]
    character_controller: Rc<CharacterControllerConfig>,
    #[serde(default)]
    car: Rc<CarConfig>,
}

impl Default for Level0Config {
    fn default() -> Self {
        Self {
            level_event_queue_capacity: Some(5),
            physics: PhysicsConfig::default(),
            character_controller: Rc::new(CharacterControllerConfig::default()),
            car: Rc::new(CarConfig::default()),
        }
    }
}

impl TryFromToml for Level0Config {
    fn try_from_toml(toml_str: &str) -> Result<Self, String> {
        match toml::from_str::<Level0Config>(toml_str) {
            Ok(config) => Ok(config),
            Err(toml_de_err) => Err(toml_de_err.to_string()),
        }
    }
}

impl TryToToml for Level0Config {
    fn try_to_toml(&self) -> Result<String, String> {
        match toml::to_string(self) {
            Ok(config_toml) => Ok(config_toml),
            Err(toml_ser_err) => Err(toml_ser_err.to_string()),
        }
    }
}

impl Level0Config {
    pub fn level_event_queue_capacity(&self) -> Option<usize> {
        self.level_event_queue_capacity
    }

    pub fn physics(&self) -> &PhysicsConfig {
        &self.physics
    }

    pub fn character_controller(&self) -> &Rc<CharacterControllerConfig> {
        &self.character_controller
    }

    pub fn car(&self) -> &Rc<CarConfig> {
        &self.car
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_to_toml() {
        // Test normal conditions
        let config = Level0Config::default();
        let config_toml = config.try_to_toml().unwrap();

        assert_eq!(config_toml, "level_event_queue_capacity = 5\n\n[physics]\ngravity = [0.0, -9.81, 0.0]\nevent_queue_capacity = 5\n\n[character_controller]\nmass = 1.0\nmax_look_up_angle = 1.5707964\nmin_look_up_angle = -1.3089969\nenter_head_tilt_factor = 0.12\nexit_head_tilt_factor = 0.08\nnonstationary_speed_threshold = 0.02\nmax_standing_move_speed = 7.5\nmax_crouched_move_speed = 2.5\nmax_standing_move_acceleration = 25.0\nmax_crouched_move_acceleration = 12.5\ncapsule_standing_height = 1.83\ncapsule_standing_radius = 0.4\ncapsule_crouched_height = 0.915\ncapsule_crouched_radius = 0.4\nstanding_head_translation_offset = [0.0, 0.7686, -0.32000002]\ncrouched_head_translation_offset = [0.0, 0.3843, -0.32000002]\nhead_crouch_lerp_factor = 0.2\nmax_jump_coyote_duration = 0.275\njump_standing_acceleration = 6.0\njump_crouched_acceleration = 3.5\nmin_jump_standing_cooldown_duration = 0.3\nmin_jump_crouched_cooldown_duration = 0.5\njump_wallrunning_scale = 1.0\njump_wallrunning_down_velocity_angle_threshold = 30.0\njump_wallrunning_normal_scale = 0.35\nwallrunning_ray_length = 0.4\nground_ray_length = 0.1\nmax_wallrunning_forward_angle = 75.0\nstart_wallrunning_up_impulse = 4.0\nstart_wallrunning_gravity_scale = 0.5\ngrounded_seconds_per_footstep = 0.25\nwallrunning_seconds_per_footstep = 0.16666667\nsliding_speed_factor = 0.8\nsliding_max_forward_angle = 30.0\nsliding_deceleration = [0.0, 0.0, 4.5]\nsliding_velocity_increase = [0.0, 0.0, -6.0]\nendless_slide_downhill_max_down_angle = 80.0\nendless_slide_ground_normal_max_up_angle = 30.0\nendless_sliding_acceleration = [0.0, 0.0, -10.0]\nevent_queue_capacity = 10\n");
    }
}
