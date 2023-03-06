use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Spring {
    offset: f32,
    velocity: f32,
    constant: f32,
    dampening: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            offset: 0.0,
            velocity: 0.0,
            constant: 1.0,
            dampening: 1.0,
        }
    }
}

impl Spring {
    pub fn new(constant: f32, dampening: f32) -> Self {
        Self {
            constant,
            dampening,
            ..Default::default()
        }
    }

    pub fn force(&self) -> f32 {
        self.offset * self.constant - self.velocity * self.dampening
    }

    pub fn set_offset(&mut self, new_offset: f32) {
        self.offset = new_offset;
    }

    pub fn set_velocity(&mut self, new_velocity: f32) {
        self.velocity = new_velocity;
    }

    pub fn set_constant(&mut self, new_constant: f32) {
        self.constant = new_constant;
    }

    pub fn set_dampening(&mut self, new_dampening: f32) {
        self.dampening = new_dampening;
    }
}
