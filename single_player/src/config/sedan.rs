use perigee::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::raycast_vehicle::WheelConfig;
use crate::config::RaycastVehicleConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SedanConfig {
    pub raycast_vehicle_controller: RaycastVehicleConfig,
}

impl Default for SedanConfig {
    fn default() -> Self {
        let cabin_half_height = 0.55;
        let cabin_half_width = 0.363;
        let cabin_half_length = 1.225;
        let wheel_horizontal_center_distance = 0.55;
        let wheel_vertical_center_distance = cabin_half_height * 0.8;
        Self {
            raycast_vehicle_controller: RaycastVehicleConfig {
                cabin_half_height,
                cabin_half_length,
                cabin_half_width,
                suspension_spring_stiffness: 70.0,
                suspension_spring_dampening: 5.0,
                cabin_center_of_mass: Point::new(0.0, -cabin_half_height * 0.9, 0.0),
                wheel_radius: 0.3,
                throttle_force: 100.0,
                mass: 50.0,
                max_boom_length: 6.0,
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
