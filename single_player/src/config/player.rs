use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::config::CharacterControllerConfig;

#[derive(Debug, Clone, Serialize, Deserialize, CopyGetters, Getters)]
pub struct PlayerConfig {
    #[getset(get = "pub")]
    character_controller: Rc<CharacterControllerConfig>,
    #[getset(get_copy = "pub")]
    event_queue_capacity: usize,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            character_controller: Rc::new(CharacterControllerConfig::default()),
            event_queue_capacity: 10,
        }
    }
}
