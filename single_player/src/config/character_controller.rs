use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PerspectiveMode {
    #[serde(rename = "first-person")]
    FirstPerson,
    #[serde(rename = "third-person-basic")]
    ThirdPersonBasic,
    #[serde(rename = "third-person-combat")]
    ThirdPersonCombat,
}

impl PerspectiveMode {
    pub fn is_third_person(&self) -> bool {
        self == &PerspectiveMode::ThirdPersonBasic || self == &PerspectiveMode::ThirdPersonCombat
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    #[serde(rename = "discrete")]
    Discrete,
    #[serde(rename = "continuous")]
    Continuous,
}

/// Configuration parameters for the [FirstPersonCharacterController](crate::shared::character_controller::FirstPersonCharacterController).
/// These should not be editable at runtime.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, CopyGetters)]
pub struct CharacterControllerConfig {
    /// The mass of the character controller's body (via its collider).
    #[getset(get_copy = "pub")]
    mass: f32,
    /// How high the character controller can look (max X axis rotation of the head or viewpoint).
    #[getset(get_copy = "pub")]
    max_look_up_angle: f32,
    /// How low the character controller can look (min X axis rotation of the head or viewpoint).
    #[getset(get_copy = "pub")]
    min_look_up_angle: f32,
    /// The per-frame lerp factor (alpha) used when entering a wallrunning head tilt.
    #[getset(get_copy = "pub")]
    enter_head_tilt_factor: f32,
    /// The per-frame lerp factor (alpha) used when exiting a wallrunning head tilt.
    #[getset(get_copy = "pub")]
    exit_head_tilt_factor: f32,
    /// How fast the character controller must be moving to be considered
    /// moving.
    #[getset(get_copy = "pub")]
    nonstationary_speed_threshold: f32,
    /// The max speed to which character controller's own forces can move its rigid body when standing in the continuous movement mode.
    #[getset(get_copy = "pub")]
    max_standing_move_speed_continuous: f32,
    /// The max speed to which character controller's own forces can move its rigid body when crouched in the continuous movement mode.
    #[getset(get_copy = "pub")]
    max_crouched_move_speed_continuous: f32,
    /// The max acceleration force the character controller can apply to its rigid body when standing in the continuous movement mode.
    #[getset(get_copy = "pub")]
    max_standing_move_acceleration_continuous: f32,
    /// The max acceleration force the character controller can apply to its rigid body when crouched in the continuous movement mode.
    #[getset(get_copy = "pub")]
    max_crouched_move_acceleration_continuous: f32,
    /// The walk speed of the character controller when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_walk_speed_discrete: f32,
    /// The run speed of the character controller when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_run_speed_discrete: f32,
    /// The sprint speed of the character controller when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_sprint_speed_discrete: f32,
    /// The creep speed of the character controller when crouched in the discrete movement mode.
    #[getset(get_copy = "pub")]
    crouched_creep_speed_discrete: f32,
    /// The walk acceleration force the character controller applies to its rigid body when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_walk_acceleration_discrete: f32,
    /// The run acceleration force the character controller applies to its rigid body when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_run_acceleration_discrete: f32,
    /// The sprint acceleration force the character controller applies to its rigid body when standing in the discrete movement mode.
    #[getset(get_copy = "pub")]
    standing_sprint_acceleration_discrete: f32,
    /// The creep acceleration force the character controller applies to its rigid body when crouched in the discrete movement mode.
    #[getset(get_copy = "pub")]
    crouched_creep_acceleration_discrete: f32,
    /// The threshold (between 0 and 1) above which a character controller's movement vector's magnitude must be greater than to trigger a sprint.
    #[getset(get_copy = "pub")]
    standing_sprint_input_threshold: f32,
    /// The max angle between the forward vector and the movement vector under which the character controller can sprint in discrete movement mode.
    #[getset(get_copy = "pub")]
    max_sprint_forward_angle_threshold_discrete: f32,
    /// How much of the discrete speed the character controller must be moving at to be considered moving at the speed
    #[getset(get_copy = "pub")]
    discrete_movement_factor: f32,
    /// The threshold (between 0 and 1) above which a character controller's movement vector's magnitude must be greater than to trigger a run.
    #[getset(get_copy = "pub")]
    standing_run_input_threshold: f32,
    /// The total height of the character controller's capsule when standing.
    #[getset(get_copy = "pub")]
    capsule_standing_total_height: f32,
    /// The radius of the cylinder part of the character controller and each half-sphere of the capsule when standing.
    #[getset(get_copy = "pub")]
    capsule_standing_radius: f32,
    /// The total height of the character controller's capsule when crouched.
    #[getset(get_copy = "pub")]
    capsule_crouched_total_height: f32,
    /// The radius of the cylinder part of the character controller and each half-sphere of the capsule when standing.
    #[getset(get_copy = "pub")]
    capsule_crouched_radius: f32,
    /// The translational offset of the head (which is often tracked by the camera) from the center of the
    /// capsule when standing.
    #[getset(get_copy = "pub")]
    standing_head_translation_offset: [f32; 3],
    /// The translational offset of the head (which is often tracked by the camera) from the center of the
    /// capsule when crouched.
    #[getset(get_copy = "pub")]
    crouched_head_translation_offset: [f32; 3],
    /// How quickly the character controller's head lerps between its standing
    /// translational offset and crouched translational offset.
    #[getset(get_copy = "pub")]
    head_crouch_lerp_factor: f32,
    /// How many seconds after no longer being grounded or wallrunning
    /// the character controller can still jump.
    #[getset(get_copy = "pub")]
    max_jump_coyote_duration: f32,
    /// How much force is used to make the character controller jump when standing.
    #[getset(get_copy = "pub")]
    jump_standing_acceleration: f32,
    /// How much force is used to make the character controller jump when crouched.
    #[getset(get_copy = "pub")]
    jump_crouched_acceleration: f32,
    /// How many seconds must pass before another jump is possible
    /// while the character controller is standing up.
    #[getset(get_copy = "pub")]
    min_jump_standing_cooldown_duration: f32,
    /// How many seconds must pass before another jump is possible.
    /// while the character controller is crouching.
    #[getset(get_copy = "pub")]
    min_jump_crouched_cooldown_duration: f32,
    /// The scale factor of jump force (up + forward) when wallrunning.
    #[getset(get_copy = "pub")]
    jump_wallrunning_scale: f32,
    /// How close to straight down the body must be moving when wallrunning for the
    /// vertical velocity to be canceled before jumping off the wall.
    ///
    /// This is the minimum angle between the velocity and the down vector
    /// to be considered wallrunning downward.
    #[getset(get_copy = "pub")]
    jump_wallrunning_down_velocity_angle_threshold: f32,
    /// The scale factor of jump force in the direction of the wall normal when wallrunning.
    #[getset(get_copy = "pub")]
    jump_wallrunning_normal_scale: f32,
    /// How far from the character controller the rays used to determine whether it's
    /// wallrunning go.
    #[getset(get_copy = "pub")]
    wallrunning_ray_length: f32,
    /// How far below the character controller the ray used to determine whether it's
    /// grounded goes.
    #[getset(get_copy = "pub")]
    ground_ray_length: f32,
    /// How far straight ahead the character controller must be moving next to a wall
    /// to be considered wallrunning. Values closer to 1 mean more straightforwardness.
    #[getset(get_copy = "pub")]
    max_wallrunning_forward_angle: f32,
    /// The vertical acceleration applied to the character controller's
    /// body once wallrunning has started.
    #[getset(get_copy = "pub")]
    start_wallrunning_up_impulse: f32,
    /// The gravity scale of the character controller's body once wallrunning
    /// has started.
    #[getset(get_copy = "pub")]
    start_wallrunning_gravity_scale: f32,
    /// How many seconds should pass before another footstep is taken
    /// when moving at the max speed while grounded.
    #[getset(get_copy = "pub")]
    grounded_seconds_per_footstep: f32,
    /// How many seconds should pass before another footstep is taken
    /// when moving at the max speed while wallrunning.
    #[getset(get_copy = "pub")]
    wallrunning_seconds_per_footstep: f32,
    /// How much of the max standing speed must the character controller
    /// be moving in order to slide when the crouch input is hit.
    #[getset(get_copy = "pub")]
    sliding_speed_factor: f32,
    /// How straightforward the character controller must be moving
    /// before entering a slide.
    #[getset(get_copy = "pub")]
    sliding_max_forward_angle: f32,
    /// The acceleration vector applied to the rigid body
    /// when the character controller starts sliding.
    #[getset(get_copy = "pub")]
    sliding_deceleration: [f32; 3],
    /// The increase in velocity applied to the rigid
    /// body when the character controller starts sliding.
    #[getset(get_copy = "pub")]
    sliding_velocity_increase: [f32; 3],
    /// The minimum dot factor of the character controller's velocity with
    /// a vector facing (0, -1, -1) needed for the character controller to be
    /// considered traveling downhill.
    #[getset(get_copy = "pub")]
    endless_slide_downhill_max_down_angle: f32,
    /// The maximum dot factor of the character controller's ground normal
    /// with the up vector (0, 1, 0) to be considered traveling downhill.
    #[getset(get_copy = "pub")]
    endless_slide_ground_normal_max_up_angle: f32,
    /// The acceleration applied to endless / downhill slides.
    #[getset(get_copy = "pub")]
    endless_sliding_acceleration: [f32; 3],
    /// The max capacity of the event channel used by the character controller structure.
    #[getset(get_copy = "pub")]
    event_queue_capacity: usize,
    /// The length of the default boom arm.
    #[getset(get_copy = "pub")]
    default_boom_arm_length: f32,
    /// The pitch angle (about X axis) of the default boom arm in degrees.
    #[getset(get_copy = "pub")]
    default_boom_arm_pitch_angle: f32,
    /// The yaw angle (about Y axis) of the default boom arm in degrees.
    #[getset(get_copy = "pub")]
    default_boom_arm_yaw_angle: f32,
    /// How quickly the third person boom moves between the default and aim booms.
    #[getset(get_copy = "pub")]
    boom_lerp_factor: f32,
    /// The length of the aiming boom arm.
    #[getset(get_copy = "pub")]
    aim_boom_arm_length: f32,
    /// The pitch angle (about X axis) of the aiming boom arm in degrees.
    #[getset(get_copy = "pub")]
    aim_boom_arm_pitch_angle: f32,
    /// The yaw angle (about Y axis) of the aiming boom arm in degrees.
    #[getset(get_copy = "pub")]
    aim_boom_arm_yaw_angle: f32,
    /// The lerp factor of character controller body isometry to the boom isometry
    /// while in third person combat mode.
    #[getset(get_copy = "pub")]
    tpcombat_boom_rotation_lerp_factor: f32,
    /// The lerp factor for the character controller body to rotate in the character controller's movement direction.
    #[getset(get_copy = "pub")]
    rotate_body_to_movement_dir_lerp_factor: f32,
    #[getset(get_copy = "pub")]
    initial_perspective_mode: PerspectiveMode,
    #[getset(get_copy = "pub")]
    movement_mode: MovementMode,
}

impl Default for CharacterControllerConfig {
    fn default() -> Self {
        let capsule_total_height = 1.83;
        let capsule_radius = 0.4;
        Self {
            mass: 1.0,
            max_look_up_angle: 90.0,
            min_look_up_angle: -60.0,
            enter_head_tilt_factor: 0.12,
            exit_head_tilt_factor: 0.08,
            nonstationary_speed_threshold: 0.02,
            max_standing_move_speed_continuous: 5.0,
            max_crouched_move_speed_continuous: 2.5,
            max_standing_move_acceleration_continuous: 25.0,
            max_crouched_move_acceleration_continuous: 12.5,
            standing_walk_speed_discrete: 3.0,
            standing_run_speed_discrete: 5.0,
            standing_sprint_speed_discrete: 7.5,
            crouched_creep_speed_discrete: 1.0,
            standing_walk_acceleration_discrete: 30.0,
            standing_run_acceleration_discrete: 35.0,
            standing_sprint_acceleration_discrete: 40.0,
            crouched_creep_acceleration_discrete: 28.0,
            standing_sprint_input_threshold: 0.9,
            max_sprint_forward_angle_threshold_discrete: 22.4,
            standing_run_input_threshold: 0.5,
            discrete_movement_factor: 0.75,
            capsule_standing_total_height: capsule_total_height,
            capsule_standing_radius: capsule_radius,
            capsule_crouched_total_height: capsule_total_height / 2.0,
            capsule_crouched_radius: capsule_radius,
            standing_head_translation_offset: [
                0.0,
                capsule_total_height / 2.0 * 0.84,
                capsule_radius * 0.23,
            ],
            crouched_head_translation_offset: [
                0.0,
                capsule_total_height / 4.0 * 0.84,
                -capsule_radius * 0.8,
            ],
            head_crouch_lerp_factor: 0.2,
            max_jump_coyote_duration: 0.275,
            jump_standing_acceleration: 6.0,
            jump_crouched_acceleration: 3.5,
            min_jump_standing_cooldown_duration: 0.3,
            min_jump_crouched_cooldown_duration: 0.5,
            jump_wallrunning_scale: 1.0,
            jump_wallrunning_normal_scale: 0.35,
            jump_wallrunning_down_velocity_angle_threshold: 30.0,
            ground_ray_length: 0.1,
            wallrunning_ray_length: 0.4,
            max_wallrunning_forward_angle: 75.0,
            start_wallrunning_up_impulse: 4.0,
            start_wallrunning_gravity_scale: 0.5,
            grounded_seconds_per_footstep: 1.0 / 4.0,
            wallrunning_seconds_per_footstep: 1.0 / 6.0,
            sliding_speed_factor: 0.8,
            sliding_max_forward_angle: 30.0,
            sliding_velocity_increase: [0.0, 0.0, -6.0],
            sliding_deceleration: [0.0, 0.0, 4.5],
            endless_slide_downhill_max_down_angle: 80.0,
            endless_slide_ground_normal_max_up_angle: 30.0,
            endless_sliding_acceleration: [0.0, 0.0, -10.0],
            event_queue_capacity: 10,
            default_boom_arm_length: 3.0,
            default_boom_arm_pitch_angle: 0.0,
            default_boom_arm_yaw_angle: 0.0,
            boom_lerp_factor: 0.9999,
            aim_boom_arm_length: 2.0,
            aim_boom_arm_pitch_angle: 0.0,
            aim_boom_arm_yaw_angle: 20.0,
            tpcombat_boom_rotation_lerp_factor: 0.9,
            rotate_body_to_movement_dir_lerp_factor: 0.999,
            initial_perspective_mode: PerspectiveMode::ThirdPersonBasic,
            movement_mode: MovementMode::Discrete,
        }
    }
}

impl CharacterControllerConfig {
    /// The height of the cylinder part of the character controller's capsule when standing.
    pub fn capsule_standing_half_height(&self) -> f32 {
        self.capsule_standing_total_height / 2.0 - self.capsule_standing_radius
    }

    /// The height of the cylinder part of the character controller's capsule when crouched.
    pub fn capsule_crouched_half_height(&self) -> f32 {
        self.capsule_crouched_total_height / 2.0 - self.capsule_crouched_radius
    }
}
