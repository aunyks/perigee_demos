use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::config::CharacterControllerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub character_controller: Rc<CharacterControllerConfig>,
    pub event_queue_capacity: usize,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            character_controller: Rc::new(CharacterControllerConfig {
                mass: 20.0,
                ..Default::default()
            }),
            event_queue_capacity: 10,
        }
    }
}
