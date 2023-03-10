use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, CopyGetters)]
pub struct WheelWellConfig {
    #[getset(get_copy = "pub")]
    receives_power: bool,
    #[getset(get_copy = "pub")]
    center_cabin_relative_position: [f32; 3],
    /// If `None` then default to the car suspension max length
    #[getset(get_copy = "pub")]
    suspension_max_length: Option<f32>,
    #[getset(get_copy = "pub")]
    steers_on_input: bool,
}

#[derive(Clone, Serialize, Deserialize, CopyGetters, Getters)]
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
    #[getset(get_copy = "pub")]
    initial_boom_pitch_angle: f32,
    #[getset(get_copy = "pub")]
    initial_boom_yaw_angle: f32,
    #[getset(get = "pub")]
    wheel_wells: Vec<WheelWellConfig>,
}

impl Default for CarConfig {
    fn default() -> Self {
        let cabin_half_width = 0.5;
        let cabin_half_length = 1.0;
        let cabin_half_height = 0.5;
        Self {
            cabin_half_width,
            cabin_half_height,
            cabin_half_length,
            shock_spring_constant: 7.0,
            shock_spring_dampening_factor: 3.0,
            mass: 4.0,
            suspension_max_length: 1.0,
            initial_boom_pitch_angle: -10.0,
            initial_boom_yaw_angle: 0.0,
            brake_force: 20.0,
            throttle_force: 20.0,
            wheel_grip: 3.0,
            wheel_left_turn_angle: 45.0_f32.to_radians(),
            wheel_right_turn_angle: -45.0_f32.to_radians(),
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            max_boom_length: 3.0,
            wheel_wells: vec![
                WheelWellConfig {
                    suspension_max_length: None,
                    receives_power: true,
                    center_cabin_relative_position: [
                        -cabin_half_width,
                        -cabin_half_height,
                        -cabin_half_length,
                    ],
                    steers_on_input: true,
                },
                WheelWellConfig {
                    suspension_max_length: None,
                    receives_power: true,
                    center_cabin_relative_position: [
                        cabin_half_width,
                        -cabin_half_height,
                        -cabin_half_length,
                    ],
                    steers_on_input: true,
                },
                WheelWellConfig {
                    suspension_max_length: None,
                    receives_power: false,
                    center_cabin_relative_position: [
                        -cabin_half_width,
                        -cabin_half_height,
                        cabin_half_length,
                    ],
                    steers_on_input: false,
                },
                WheelWellConfig {
                    suspension_max_length: None,
                    receives_power: true,
                    center_cabin_relative_position: [
                        cabin_half_width,
                        -cabin_half_height,
                        cabin_half_length,
                    ],
                    steers_on_input: false,
                },
            ],
        }
    }
}
