use getset::Getters;
use perigee::rapier3d::control::WheelTuning;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WheelConfig {
    pub receives_power: bool,
    pub center_cabin_relative_position: [f32; 3],
    pub steers_on_input: bool,
    /// If `None` then default to the car suspension max length
    pub suspension_rest_length: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct RaycastVehicleConfig {
    pub cabin_half_width: f32,
    pub cabin_half_height: f32,
    pub cabin_half_length: f32,
    pub suspension_spring_stiffness: f32,
    pub suspension_spring_dampening: f32,
    pub mass: f32,
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
    #[serde(default)]
    pub suspension_rest_length: f32,
    #[serde(default)]
    pub wheel_radius: f32,
}

impl Default for RaycastVehicleConfig {
    fn default() -> Self {
        let cabin_half_width = 0.3;
        let cabin_half_length = 1.0;
        let cabin_half_height = 0.3;
        Self {
            cabin_half_width,
            cabin_half_height,
            cabin_half_length,
            suspension_spring_stiffness: 150.0,
            suspension_spring_dampening: 12.0,
            mass: 100.0,
            suspension_rest_length: cabin_half_height,
            wheel_radius: cabin_half_height / 4.0,
            initial_boom_pitch_angle: -10.0,
            initial_boom_yaw_angle: 0.0,
            brake_force: 30.0,
            throttle_force: 30.0,
            wheel_grip: 10.5,
            wheel_left_turn_angle: 40.0,
            wheel_right_turn_angle: -40.0,
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            max_boom_length: 3.0,
            wheels: vec![
                WheelConfig {
                    suspension_rest_length: None,
                    receives_power: true,
                    radius: None,
                    center_cabin_relative_position: [
                        -cabin_half_width * 0.75,
                        -cabin_half_height,
                        -cabin_half_length * 0.75,
                    ],
                    steers_on_input: true,
                },
                WheelConfig {
                    suspension_rest_length: None,
                    receives_power: true,
                    radius: None,
                    center_cabin_relative_position: [
                        cabin_half_width * 0.75,
                        -cabin_half_height,
                        -cabin_half_length * 0.75,
                    ],
                    steers_on_input: true,
                },
                WheelConfig {
                    suspension_rest_length: None,
                    receives_power: false,
                    radius: None,
                    center_cabin_relative_position: [
                        -cabin_half_width * 0.75,
                        -cabin_half_height,
                        cabin_half_length * 0.75,
                    ],
                    steers_on_input: false,
                },
                WheelConfig {
                    suspension_rest_length: None,
                    receives_power: true,
                    radius: None,
                    center_cabin_relative_position: [
                        cabin_half_width * 0.75,
                        -cabin_half_height,
                        cabin_half_length * 0.75,
                    ],
                    steers_on_input: false,
                },
            ],
        }
    }
}

impl From<&RaycastVehicleConfig> for WheelTuning {
    fn from(vehicle_config: &RaycastVehicleConfig) -> Self {
        Self {
            suspension_stiffness: vehicle_config.suspension_spring_stiffness,
            suspension_damping: vehicle_config.suspension_spring_dampening,
            friction_slip: vehicle_config.wheel_grip,
            ..Default::default()
        }
    }
}
