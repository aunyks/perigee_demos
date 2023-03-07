use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Spring {
    compression: f32,
    velocity: f32,
    strength: f32,
    dampening: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            compression: 0.0,
            velocity: 0.0,
            strength: 1.0,
            dampening: 1.0,
        }
    }
}

impl Spring {
    pub fn new(strength: f32, dampening: f32) -> Self {
        Self {
            strength,
            dampening,
            ..Default::default()
        }
    }

    pub fn force(&self) -> f32 {
        self.compression * self.strength - self.velocity * self.dampening
    }

    pub fn set_compression(&mut self, new_compression: f32) {
        self.compression = new_compression;
    }

    pub fn set_velocity(&mut self, new_velocity: f32) {
        self.velocity = new_velocity;
    }

    pub fn set_strength(&mut self, new_strength: f32) {
        self.strength = new_strength;
    }

    pub fn set_dampening(&mut self, new_dampening: f32) {
        self.dampening = new_dampening;
    }
}
