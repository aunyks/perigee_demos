use perigee::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::raycast_vehicle::WheelConfig;
use crate::config::RaycastVehicleConfig;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "track")]
    Track,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SedanConfig {
    pub raycast_vehicle_controller: RaycastVehicleConfig,
    pub initial_boom_pitch_angle: f32,
    pub initial_boom_yaw_angle: f32,
    pub initial_camera_mode: CameraMode,
    pub max_boom_length: f32,
    pub max_look_up_angle: f32,
    pub min_look_up_angle: f32,
    pub track_mode_look_pitch_angle: f32,
    pub track_mode_look_yaw_angle: f32,
    pub track_mode_cam_lerp_factor: f32,
}

impl Default for SedanConfig {
    fn default() -> Self {
        let cabin_half_height = 0.55;
        let cabin_half_width = 0.635;
        let cabin_half_length = 1.225;
        let wheel_horizontal_center_distance = 0.55;
        let wheel_vertical_center_distance = cabin_half_height * 0.8;
        Self {
            initial_boom_pitch_angle: -10.0,
            initial_boom_yaw_angle: 0.0,
            max_boom_length: 6.0,
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            initial_camera_mode: CameraMode::Track,
            track_mode_look_pitch_angle: -15.0,
            track_mode_look_yaw_angle: 0.0,
            track_mode_cam_lerp_factor: 0.9999,
            raycast_vehicle_controller: RaycastVehicleConfig {
                cabin_half_height,
                cabin_half_length,
                cabin_half_width,
                suspension_spring_stiffness: 70.0,
                suspension_spring_dampening: 5.0,
                cabin_center_of_mass: Point::new(0.0, -cabin_half_height * 0.9, 0.0),
                wheel_radius: 0.3,
                throttle_force: 100.0,
                wheel_left_turn_angle: 20.0,
                wheel_right_turn_angle: -20.0,
                mass: 50.0,
                suspension_rest_length: 0.0,
                wheels: vec![
                    WheelConfig {
                        suspension_rest_length: None,
                        receives_power: true,
                        radius: None,
                        center_cabin_relative_position: [
                            -wheel_horizontal_center_distance,
                            -wheel_vertical_center_distance,
                            -cabin_half_length * 0.54,
                        ],
                        steers_on_input: true,
                    },
                    WheelConfig {
                        suspension_rest_length: None,
                        receives_power: true,
                        radius: None,
                        center_cabin_relative_position: [
                            wheel_horizontal_center_distance,
                            -wheel_vertical_center_distance,
                            -cabin_half_length * 0.54,
                        ],
                        steers_on_input: true,
                    },
                    WheelConfig {
                        suspension_rest_length: None,
                        receives_power: false,
                        radius: None,
                        center_cabin_relative_position: [
                            -wheel_horizontal_center_distance,
                            -wheel_vertical_center_distance,
                            cabin_half_length * 0.54,
                        ],
                        steers_on_input: false,
                    },
                    WheelConfig {
                        suspension_rest_length: None,
                        receives_power: true,
                        radius: None,
                        center_cabin_relative_position: [
                            wheel_horizontal_center_distance,
                            -wheel_vertical_center_distance,
                            cabin_half_length * 0.54,
                        ],
                        steers_on_input: false,
                    },
                ],
                ..Default::default()
            },
        }
    }
}
