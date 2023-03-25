use serde::{Deserialize, Serialize};

/// Assumes a two-wing aircraft
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlaneControllerConfig {
    pub fuselage_half_width: f32,
    pub fuselage_half_height: f32,
    pub fuselage_half_length: f32,
    pub mass: f32,
    pub min_thrust_force: f32,
    pub idle_thrust_force: f32,
    pub max_thrust_force: f32,
    pub max_roll_force: f32,
    pub max_pitch_force: f32,
    pub max_yaw_force: f32,
    /// Assume a reference area of 1.0.
    /// All other inputs to the drag equation will vary.
    pub drag_coefficient: f32,
    /// Assume a reference area of 1.0.
    /// All other inputs to the lift equation will vary.
    pub lift_coefficient: f32,
    pub max_boom_length: f32,
    pub initial_boom_pitch_angle: f32,
    pub initial_boom_yaw_angle: f32,
    pub max_look_up_angle: f32,
    pub min_look_up_angle: f32,
}

impl Default for PlaneControllerConfig {
    fn default() -> Self {
        let fuselage_half_width = 0.3;
        let fuselage_half_length = 1.0;
        let fuselage_half_height = 0.3;
        Self {
            fuselage_half_width,
            fuselage_half_length,
            fuselage_half_height,
            mass: 1.0,
            min_thrust_force: 0.0,
            idle_thrust_force: 50.0,
            max_thrust_force: 100.0,
            max_roll_force: 1.0,
            max_pitch_force: 1.0,
            max_yaw_force: 1.0,
            drag_coefficient: 1.0,
            lift_coefficient: 1.0,
            max_boom_length: 3.0,
            initial_boom_pitch_angle: -10.0,
            initial_boom_yaw_angle: 0.0,
            max_look_up_angle: 90.0,
            min_look_up_angle: -90.0,
        }
    }
}
