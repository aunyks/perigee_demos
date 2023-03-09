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
    suspension_max_length: f32,
    #[getset(get_copy = "pub")]
    brake_force: f32,
    #[getset(get_copy = "pub")]
    throttle_force: f32,
    #[getset(get_copy = "pub")]
    wheel_grip: f32,
    #[getset(get_copy = "pub")]
    wheel_left_turn_angle: f32,
    #[getset(get_copy = "pub")]
    wheel_right_turn_angle: f32,
    #[getset(get_copy = "pub")]
    max_look_up_angle: f32,
    #[getset(get_copy = "pub")]
    min_look_up_angle: f32,
    #[getset(get_copy = "pub")]
    max_boom_length: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            cabin_half_width: 0.5,
            cabin_half_height: 0.5,
            cabin_half_length: 1.0,
            shock_spring_constant: 10.0,
            shock_spring_dampening_factor: 3.0,
            mass: 1.0,
            suspension_max_length: 1.0,
            brake_force: 5.0,
            throttle_force: 5.0,
            wheel_grip: 0.8,
            wheel_left_turn_angle: 45.0_f32.to_radians(),
            wheel_right_turn_angle: -45.0_f32.to_radians(),
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            max_boom_length: 3.0,
        }
    }
}
