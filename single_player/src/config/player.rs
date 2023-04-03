use serde::{Deserialize, Serialize};

use crate::config::CharacterControllerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub character_controller: CharacterControllerConfig,
    pub event_queue_capacity: usize,
    pub aerial_max_move_acceleration: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            character_controller: CharacterControllerConfig {
                jump_standing_acceleration: 7.0,
                mass: 20.0,
                ..Default::default()
            },
            event_queue_capacity: 10,
            aerial_max_move_acceleration: 5.0,
        }
    }
}
