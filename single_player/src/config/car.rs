use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, CopyGetters)]
pub struct CarConfig {
    #[getset(get_copy = "pub")]
    cabin_half_width: f32,
    #[getset(get_copy = "pub")]
    cabin_half_height: f32,
    #[getset(get_copy = "pub")]
    cabin_half_length: f32,
    #[getset(get_copy = "pub")]
    shock_spring_constant: f32,
    #[getset(get_copy = "pub")]
    shock_spring_dampening_factor: f32,
    #[getset(get_copy = "pub")]
    mass: f32,
    #[getset(get_copy = "pub")]
    desired_cabin_altitude: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            cabin_half_width: 0.5,
            cabin_half_height: 0.5,
            cabin_half_length: 1.0,
            shock_spring_constant: 10.0,
            shock_spring_dampening_factor: 1.0,
            mass: 1.0,
            desired_cabin_altitude: 1.0,
        }
    }
}
