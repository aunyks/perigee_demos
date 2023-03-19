use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WheelConfig {
    pub receives_power: bool,
    pub center_cabin_relative_position: [f32; 3],
    /// If `None` then default to the car suspension max length
    pub suspension_max_length: Option<f32>,
    pub steers_on_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct CarConfig {
    pub cabin_half_width: f32,
    pub cabin_half_height: f32,
    pub cabin_half_length: f32,
    pub suspension_spring_stiffness: f32,
    pub suspension_spring_dampening: f32,
    pub mass: f32,
    pub suspension_max_length: f32,
    pub brake_force: f32,
    pub throttle_force: f32,
    pub wheel_grip: f32,
    pub wheel_left_turn_angle: f32,
    pub wheel_right_turn_angle: f32,
    pub max_look_up_angle: f32,
    pub min_look_up_angle: f32,
    pub max_boom_length: f32,
    pub initial_boom_pitch_angle: f32,
    pub initial_boom_yaw_angle: f32,
    #[getset(get = "pub")]
    pub wheels: Vec<WheelConfig>,
}

impl Default for CarConfig {
    fn default() -> Self {
        let cabin_half_width = 0.5;
        let cabin_half_length = 1.0;
        let cabin_half_height = 0.25;
        Self {
            cabin_half_width,
            cabin_half_height,
            cabin_half_length,
            suspension_spring_stiffness: 100.0,
            suspension_spring_dampening: 10.0,
            mass: 100.0,
            suspension_max_length: 1.0,
            initial_boom_pitch_angle: -10.0,
            initial_boom_yaw_angle: 0.0,
            brake_force: 20.0,
            throttle_force: 20.0,
            wheel_grip: 10.5,
            wheel_left_turn_angle: 45.0,
            wheel_right_turn_angle: -45.0,
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            max_boom_length: 3.0,
            wheels: vec![
                WheelConfig {
                    suspension_max_length: None,
                    receives_power: true,
                    center_cabin_relative_position: [
                        -cabin_half_width,
                        -cabin_half_height,
                        -cabin_half_length,
                    ],
                    steers_on_input: true,
                },
                WheelConfig {
                    suspension_max_length: None,
                    receives_power: true,
                    center_cabin_relative_position: [
                        cabin_half_width,
                        -cabin_half_height,
                        -cabin_half_length,
                    ],
                    steers_on_input: true,
                },
                WheelConfig {
                    suspension_max_length: None,
                    receives_power: false,
                    center_cabin_relative_position: [
                        -cabin_half_width,
                        -cabin_half_height,
                        cabin_half_length,
                    ],
                    steers_on_input: false,
                },
                WheelConfig {
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
