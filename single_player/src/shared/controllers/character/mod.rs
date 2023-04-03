use crate::config::character_controller::{CharacterControllerConfig, MovementMode};
use crate::shared::boom::Boom;
use crate::shared::controllers::character::utils::*;
use crate::shared::events::CharacterControllerEvent;
use crate::shared::input::Input;
use crate::shared::interactions::InteractionGroup;
use crate::shared::settings::GameSettings;
use crate::shared::vectors::*;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod utils;

#[derive(Serialize, Deserialize)]
pub struct CharacterController {
    // Head up down rotation
    head_x_rotation: UnitQuaternion<f32>,
    // Head tilt rotation
    head_z_rotation: UnitQuaternion<f32>,
    head_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
    body_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
    pub boom: Boom,
    default_boom: Boom,
    aim_boom: Boom,
    pub perspective_mode: StateMachine<CharacterPerspectiveMode>,
    pub movement_mode: StateMachine<MovementMode>,
    body_linear_velocity: Vector3<f32>,
    rigid_body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    pub wallrunning_state: StateMachine<WallRunning>,
    pub crouch_state: StateMachine<CrouchState>,
    ground_normal: Option<Vector3<f32>>,
    coyote_timer: PassiveClock,
    jump_cooldown_timer: PassiveClock,
    pub sliding_state: StateMachine<SlidingState>,
    #[serde(skip)]
    event_channel: EventChannel<CharacterControllerEvent>,
}

impl FromConfig for CharacterController {
    type Config<'a> = &'a CharacterControllerConfig;
    fn from_config<'a>(config: Self::Config<'a>) -> Self {
        Self {
            head_x_rotation: UnitQuaternion::identity(),
            head_z_rotation: UnitQuaternion::identity(),
            head_isometry: Isometry::from(Vector3::from(config.standing_head_translation_offset)),
            body_isometry: Isometry::identity(),
            boom: Boom::new(
                config.default_boom_arm_length,
                config.default_boom_arm_pitch_angle,
                config.default_boom_arm_yaw_angle,
                false,
            ),
            default_boom: Boom::new(
                config.default_boom_arm_length,
                config.default_boom_arm_pitch_angle,
                config.default_boom_arm_yaw_angle,
                false,
            ),
            aim_boom: Boom::new(
                config.aim_boom_arm_length,
                config.aim_boom_arm_pitch_angle,
                config.aim_boom_arm_yaw_angle,
                false,
            ),
            perspective_mode: StateMachine::new(config.initial_perspective_mode),
            movement_mode: StateMachine::new(config.movement_mode),
            body_linear_velocity: Vector3::default(),
            rigid_body_handle: RigidBodyHandle::default(),
            collider_handle: ColliderHandle::default(),
            wallrunning_state: StateMachine::new(WallRunning::None),
            crouch_state: StateMachine::new(CrouchState::Upright),
            ground_normal: Some(Vector::y()),
            coyote_timer: PassiveClock::default(),
            jump_cooldown_timer: PassiveClock::default(),
            sliding_state: StateMachine::new(SlidingState::None),
            event_channel: EventChannel::with_capacity(config.event_queue_capacity),
        }
    }
}

impl CharacterController {
    pub fn get_event(&self) -> Result<CharacterControllerEvent, TryRecvError> {
        self.event_channel.get_message()
    }

    fn build_collider(
        &self,
        config: &CharacterControllerConfig,
        capsule_half_height: f32,
        capsule_radius: f32,
    ) -> Collider {
        ColliderBuilder::capsule_y(capsule_half_height, capsule_radius)
            .collision_groups(
                InteractionGroups::all().with_memberships(Group::from_bits_truncate(
                    InteractionGroup::CharacterController.into(),
                )),
            )
            // Listen for *all* collision and intersection events with
            // this collider
            .active_events(ActiveEvents::COLLISION_EVENTS)
            // Set the mass (in kg, I think) of the collider
            .density(config.mass)
            .build()
    }

    pub fn capsule_values(&self, config: &CharacterControllerConfig) -> (f32, f32) {
        match self.crouch_state.current_state() {
            CrouchState::Upright => (
                config.capsule_standing_half_height(),
                config.capsule_standing_radius,
            ),
            CrouchState::Crouched => (
                config.capsule_crouched_half_height(),
                config.capsule_crouched_radius,
            ),
        }
    }

    /// Create a rigid body and collider for the character controller based on the the provided configuration parameters
    /// and / or default parameters, then add them to the provided `RigidBodySet` and `ColliderSet`.
    ///
    /// This should be called after creating the character controller and before updating the character controller.
    pub fn add_to_physics_world(
        &mut self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        initial_isometry: Option<Isometry<f32, UnitQuaternion<f32>, 3>>,
    ) {
        let initial_isometry = if let Some(initial_isometry) = initial_isometry {
            initial_isometry
        } else {
            Isometry::from_parts(
                Translation::new(2.0, 1.0, 0.0),
                UnitQuaternion::from_quaternion(Quaternion::identity()),
            )
        };
        let rigid_body = RigidBodyBuilder::dynamic()
            .position(initial_isometry)
            // Don't tip over
            .enabled_rotations(false, true, false)
            // Don't ever rotate without being explicitly set
            .angular_damping(1000.0)
            .linear_damping(0.2)
            .build();
        let (capsule_half_height, capsule_radius) = self.capsule_values(config);
        let collider = self.build_collider(config, capsule_half_height, capsule_radius);

        let player_body_handle = rigid_body_set.insert(rigid_body);
        let player_body_collider =
            collider_set.insert_with_parent(collider, player_body_handle, rigid_body_set);
        self.set_body_handle(player_body_handle);
        self.set_collider_handle(player_body_collider);
    }

    /// Update the character controller based on what it knows about its internal properties
    /// and the properties of its rigid body. This should be called after the [Input](perigee::input::Input)
    /// and [GameSettings](perigee::settings::GameSettings) are updated but before the [PhysicsWorld](perigee::physics::PhysicsWorld)
    /// steps and the `CharacterController`'s events are extracted this frame.
    pub fn update(
        &mut self,
        config: &CharacterControllerConfig,
        settings: &GameSettings,
        physics: &mut PhysicsWorld,
        input: &Input,
        delta_seconds: f32,
    ) {
        self.update_body_isometry(&mut physics.rigid_body_set);
        self.update_head_isometry(config, delta_seconds);

        if self.perspective_mode.is_third_person() {
            self.update_boom_isometry(
                -input.rotate_right()
                    * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0).to_radians(),
                input.rotate_up()
                    * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                config.max_look_up_angle,
                config.min_look_up_angle,
            );
        }

        if self.perspective_mode == CharacterPerspectiveMode::FirstPerson
            || (self.perspective_mode == CharacterPerspectiveMode::ThirdPersonCombat
                && self.body_linear_velocity().magnitude() > config.nonstationary_speed_threshold)
        {
            self.rotate_body(
                config,
                -input.rotate_right()
                    * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0).to_radians(),
                &mut physics.rigid_body_set,
                delta_seconds,
            );
            if self.perspective_mode == CharacterPerspectiveMode::FirstPerson {
                self.rotate_head(
                    input.rotate_up()
                        * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                    config.max_look_up_angle,
                    config.min_look_up_angle,
                );
            }
        }

        if self.is_grounded() && self.perspective_mode == CharacterPerspectiveMode::ThirdPersonBasic
        {
            self.face_body_in_moving_direction(
                config,
                input.move_right(),
                input.move_forward(),
                &mut physics.rigid_body_set,
                delta_seconds,
            );
        }

        let (_capsule_half_height, capsule_radius) = self.capsule_values(config);

        let previous_tick_grounded_state = self.is_grounded();
        self.determine_grounded_state(
            config,
            &mut physics.rigid_body_set,
            &mut physics.query_pipeline,
            &mut physics.collider_set,
        );
        let previous_tick_wallrunning_state = *self.wallrunning_state.current_state();
        self.determine_wallrunning_state(
            config,
            &mut physics.rigid_body_set,
            capsule_radius,
            &mut physics.query_pipeline,
            &mut physics.collider_set,
        );
        self.determine_linear_velocity(&mut physics.rigid_body_set);
        let previous_tick_sliding_state = *self.sliding_state.current_state();
        self.determine_sliding_state(config);

        if previous_tick_grounded_state != self.is_grounded() {
            // We've just landed
            if self.is_grounded() {
                // We can't be grounded and wallrunning at the same time
                self.wallrunning_state.transition_to(WallRunning::None);
                self.event_channel.send(CharacterControllerEvent::Landed);
                self.coyote_timer.reset();
            } else {
                // We've just taken off
            }
        }
        if previous_tick_wallrunning_state != *self.wallrunning_state.current_state() {
            // We're entered a new wallrun
            if self.wallrunning_state != WallRunning::None && !self.is_grounded() {
                self.start_wallrunning(config, &mut physics.rigid_body_set);
                self.event_channel
                    .send(CharacterControllerEvent::StartedWallRunning);
                self.coyote_timer.reset();
            } else {
                // We've exited a wallrun
                self.stop_wallrunning(&mut physics.rigid_body_set);
                self.event_channel
                    .send(CharacterControllerEvent::StoppedWallRunning);
            }
        }
        self.tilt_head(config, delta_seconds);

        if previous_tick_sliding_state != *self.sliding_state.current_state() {
            if self.sliding_state.current_state() != &SlidingState::None {
                self.start_sliding(config, &mut physics.rigid_body_set);
                self.event_channel
                    .send(CharacterControllerEvent::StartedSliding);
            } else {
                self.stop_sliding(&mut physics.rigid_body_set);
                self.event_channel
                    .send(CharacterControllerEvent::StoppedSliding);
                if self.is_grounded() && self.crouch_state.current_state() == &CrouchState::Crouched
                {
                    self.event_channel.send(CharacterControllerEvent::Crouched);
                }
            }
        }

        self.jump_cooldown_timer.tick(delta_seconds);
        let max_jump_cooldown_timer_duration = match self.crouch_state.current_state() {
            CrouchState::Upright => config.min_jump_standing_cooldown_duration,
            CrouchState::Crouched => config.min_jump_crouched_cooldown_duration,
        };
        if !self.is_grounded() && self.wallrunning_state == WallRunning::None {
            self.coyote_timer.tick(delta_seconds);
        }

        if self.is_grounded()
            && !input.jump()
            && self.sliding_state.current_state() == &SlidingState::None
        {
            match self.movement_mode.current_state() {
                MovementMode::Continuous => self.move_body_continuous(
                    config,
                    delta_seconds,
                    input.move_right(),
                    input.move_forward(),
                    &mut physics.rigid_body_set,
                ),
                MovementMode::Discrete => self.move_body_discrete(
                    config,
                    delta_seconds,
                    input.move_right(),
                    input.move_forward(),
                    &mut physics.rigid_body_set,
                ),
            };
        }

        if input.jump() {
            let jump_has_cooled_down = self.jump_cooldown_timer.elapsed()
                > Duration::from_secs_f32(max_jump_cooldown_timer_duration);
            let is_grounded_or_wallrunning =
                self.wallrunning_state != WallRunning::None || self.is_grounded();
            let can_coyote_jump = self.coyote_timer.elapsed()
                < Duration::from_secs_f32(config.max_jump_coyote_duration);

            if jump_has_cooled_down
                && (is_grounded_or_wallrunning || can_coyote_jump)
                && self.sliding_state.current_state() != &SlidingState::Normal
            {
                self.jump(config, &mut physics.rigid_body_set);
            }
        }

        match (input.crouch(), self.crouch_state.current_state()) {
            (true, &CrouchState::Upright) => {
                self.change_crouch_state(
                    config,
                    config.capsule_crouched_half_height(),
                    config.capsule_crouched_radius,
                    &mut physics.rigid_body_set,
                    &mut physics.collider_set,
                    &mut physics.island_manager,
                );
                // If we're moving fast enough, then this is a slide.
                // Otherwise it's a normal crouch
                if self.body_linear_velocity().xz().magnitude()
                    < config.sliding_speed_factor * config.max_standing_move_speed_continuous
                {
                    self.event_channel.send(CharacterControllerEvent::Crouched);
                }
            }
            (false, &CrouchState::Crouched) => {
                if self.can_stand_up(
                    config,
                    &mut physics.rigid_body_set,
                    &mut physics.query_pipeline,
                    &mut physics.collider_set,
                    self.is_grounded(),
                ) {
                    self.change_crouch_state(
                        config,
                        config.capsule_standing_half_height(),
                        config.capsule_standing_radius,
                        &mut physics.rigid_body_set,
                        &mut physics.collider_set,
                        &mut physics.island_manager,
                    );
                    self.event_channel
                        .send(CharacterControllerEvent::StoodUpright);
                }
            }
            _ => {}
        }

        self.aim(config, input.aim(), delta_seconds);

        if self.perspective_mode.is_third_person() {
            let max_boom_arm_length = if input.aim() {
                config.aim_boom_arm_length
            } else {
                config.default_boom_arm_length
            };
            self.prevent_camera_obstructions(
                &mut physics.query_pipeline,
                &mut physics.rigid_body_set,
                &mut physics.collider_set,
                max_boom_arm_length,
            );
        }
    }

    fn update_body_isometry(&mut self, rigid_body_set: &mut RigidBodySet) {
        let mut body_isometry: Isometry<f32, UnitQuaternion<f32>, 3> = Isometry::identity();
        let body_handle = self.body_handle();
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            body_isometry = *body.position();
        }
        self.body_isometry = body_isometry;
    }

    fn update_head_isometry(&mut self, config: &CharacterControllerConfig, delta_seconds: f32) {
        let target_translation = match self.crouch_state.current_state() {
            CrouchState::Upright => self.head_standing_isometry(config).translation.vector,
            CrouchState::Crouched => self.head_crouched_isometry(config).translation.vector,
        };
        let lerp_factor = if self.is_grounded() {
            config.head_crouch_lerp_factor
        } else {
            1.0
        };
        self.head_isometry = Isometry::from_parts(
            self.head_isometry()
                .translation
                .vector
                .lerp(
                    &target_translation,
                    framerate_independent_interp_t(lerp_factor, delta_seconds),
                )
                .into(),
            self.head_rotation(),
        );
    }

    fn update_boom_isometry(
        &mut self,
        yaw_magnitude: f32,
        pitch_magnitude: f32,
        min_pitch_angle: f32,
        max_pitch_angle: f32,
    ) {
        self.boom.translation = self.body_isometry().translation;

        self.boom.z_rotation = self
            .boom
            .z_rotation
            .append_axisangle_linearized(&Vector3::new(0.0, yaw_magnitude, 0.0));

        let (x_roll, x_pitch, x_yaw) = self.boom.x_rotation.euler_angles();
        self.boom.x_rotation = UnitQuaternion::from_euler_angles(
            (x_roll + pitch_magnitude)
                .clamp(max_pitch_angle.to_radians(), min_pitch_angle.to_radians()),
            x_pitch,
            x_yaw,
        );
    }

    fn aim(&mut self, config: &CharacterControllerConfig, is_aiming: bool, delta_seconds: f32) {
        if self.perspective_mode.is_third_person() {
            let target_boom = if is_aiming {
                self.perspective_mode
                    .transition_to(CharacterPerspectiveMode::ThirdPersonCombat);
                self.aim_boom
            } else {
                self.perspective_mode
                    .transition_to(CharacterPerspectiveMode::ThirdPersonBasic);
                self.default_boom
            };

            self.boom.lerp_mut(
                &target_boom,
                framerate_independent_interp_t(config.boom_lerp_factor, delta_seconds),
            );
        }
    }

    /// Rotate the character controller's rigid body about the Y axis (left / right) based on user input.
    pub fn rotate_body(
        &self,
        config: &CharacterControllerConfig,
        y_axis_rotation: f32,
        rigid_body_set: &mut RigidBodySet,
        delta_seconds: f32,
    ) {
        let body_handle = self.body_handle();
        let rotation_scale = match self.sliding_state.current_state() {
            SlidingState::Downhill | SlidingState::Normal => 0.2,
            _ => 1.0,
        };
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            if self.perspective_mode == CharacterPerspectiveMode::FirstPerson {
                let new_body_rotation =
                    body.position()
                        .rotation
                        .append_axisangle_linearized(&Vector3::new(
                            0.0,
                            y_axis_rotation * rotation_scale,
                            0.0,
                        ));
                body.set_position(
                    Isometry::from_parts(body.position().translation, new_body_rotation),
                    false,
                );
            } else if self.perspective_mode == CharacterPerspectiveMode::ThirdPersonCombat {
                let boom_yaw_isometry =
                    Isometry::from_parts(self.boom.translation, self.boom.z_rotation);
                body.set_position(
                    body.position()
                        .try_lerp_slerp(
                            &boom_yaw_isometry,
                            framerate_independent_interp_t(
                                config.tpcombat_boom_rotation_lerp_factor,
                                delta_seconds,
                            ),
                            0.0,
                        )
                        .expect("Couldn't lerp slerp body isometry to boom isometry"),
                    false,
                );
            } else {
            }
        }
    }

    /// Rotate the character controller head about the X axis (up / down) based on user input, not exceeding the min or max look angles.
    pub fn rotate_head(&mut self, x_axis_rotation: f32, min_look_angle: f32, max_look_angle: f32) {
        let (roll, pitch, yaw) = self.head_x_rotation.euler_angles();

        self.head_x_rotation = UnitQuaternion::from_euler_angles(
            (roll + x_axis_rotation)
                .clamp(max_look_angle.to_radians(), min_look_angle.to_radians()),
            pitch,
            yaw,
        );
    }

    pub fn face_body_in_moving_direction(
        &mut self,
        config: &CharacterControllerConfig,
        left_right_magnitude: f32,
        forward_back_magnitude: f32,
        rigid_body_set: &mut RigidBodySet,
        delta_seconds: f32,
    ) {
        let movement_vector = Vector3::new(left_right_magnitude, 0.0, forward_back_magnitude);
        let trying_to_move = movement_vector.magnitude() > 0.0;

        let boom_isometry = Isometry::from_parts(self.boom.translation, self.boom.z_rotation);
        let body_handle = self.body_handle();

        if trying_to_move {
            if let Some(body) = rigid_body_set.get_mut(body_handle) {
                let body_iso = body.position();
                let move_direction = boom_isometry.transform_vector(&movement_vector);
                let new_body_rotation = match body_iso.rotation.try_slerp(
                    &UnitQuaternion::face_towards(&-move_direction, &Vector3::y_axis()),
                    framerate_independent_interp_t(
                        config.rotate_body_to_movement_dir_lerp_factor,
                        delta_seconds,
                    ),
                    0.01,
                ) {
                    Some(quat) => quat,
                    None => return,
                };
                body.set_position(
                    Isometry::from_parts(body_iso.translation, new_body_rotation),
                    false,
                );
            }
        }
    }

    fn prevent_camera_obstructions(
        &mut self,
        query_pipeline: &mut QueryPipeline,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        max_boom_length: f32,
    ) {
        let body_translation = self.body_isometry().translation;
        let diff_vec = self.boom.end_isometry().translation.vector - body_translation.vector;
        if let Some((_handle, hit_toi)) = query_pipeline.cast_ray(
            rigid_body_set,
            collider_set,
            &Ray::new(
                Point {
                    coords: body_translation.vector,
                },
                diff_vec.normalize(),
            ),
            max_boom_length,
            true,
            query_filter_excluding_player(),
        ) {
            self.boom.set_length(hit_toi - 0.03);
        } else {
            self.boom.set_length(max_boom_length);
        }
    }

    pub fn camera_isometry(&self) -> Isometry<f32, Unit<Quaternion<f32>>, 3> {
        match self.perspective_mode.current_state() {
            CharacterPerspectiveMode::ThirdPersonBasic
            | CharacterPerspectiveMode::ThirdPersonCombat => self.boom.end_isometry(),
            CharacterPerspectiveMode::FirstPerson => self.body_isometry() * self.head_isometry(),
        }
    }

    pub fn set_body_handle(&mut self, body_handle: RigidBodyHandle) {
        self.rigid_body_handle = body_handle;
    }

    pub fn set_collider_handle(&mut self, collider_handle: ColliderHandle) {
        self.collider_handle = collider_handle;
    }

    pub fn body_handle(&self) -> RigidBodyHandle {
        self.rigid_body_handle
    }

    pub fn collider_handle(&self) -> ColliderHandle {
        self.collider_handle
    }

    pub fn head_rotation(&self) -> UnitQuaternion<f32> {
        self.head_x_rotation * self.head_z_rotation
    }

    pub fn head_standing_isometry(
        &self,
        config: &CharacterControllerConfig,
    ) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        Isometry::from_parts(
            config.standing_head_translation_offset.into(),
            self.head_rotation(),
        )
    }

    pub fn head_crouched_isometry(
        &self,
        config: &CharacterControllerConfig,
    ) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        Isometry::from_parts(
            config.crouched_head_translation_offset.into(),
            self.head_rotation(),
        )
    }

    pub fn head_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.head_isometry
    }

    /// Get the isometry (position and orientation) of the character controller's rigid body.
    pub fn body_isometry(&self) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        &self.body_isometry
    }

    pub fn is_grounded(&self) -> bool {
        self.ground_normal().is_some()
    }

    pub fn ground_normal(&self) -> &Option<Vector3<f32>> {
        &self.ground_normal
    }

    pub fn body_linear_velocity(&self) -> &Vector3<f32> {
        &self.body_linear_velocity
    }

    pub fn pivot_isometry(&self) -> Isometry<f32, Unit<Quaternion<f32>>, 3> {
        match self.perspective_mode.current_state() {
            CharacterPerspectiveMode::ThirdPersonBasic
            | CharacterPerspectiveMode::ThirdPersonCombat => {
                Isometry::from_parts(self.boom.translation, self.boom.z_rotation)
            }
            CharacterPerspectiveMode::FirstPerson => *self.body_isometry(),
        }
    }

    fn move_body(
        &mut self,
        max_velocity: &Vector3<f32>,
        delta_seconds: f32,
        max_move_acceleration: f32,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let current_velocity = self.body_linear_velocity();
        let body_handle = self.body_handle();
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            let pivot_isometry = self.pivot_isometry();
            // The max velocity transformed by the isometry (position & orientation)
            // of the pivot (boom or body position).
            let transformed_max_velocity = pivot_isometry.transform_vector(&max_velocity);
            // The character controller isometry-transformed max velocity rotated to point in the
            // direction of the slope the character controller is currently on
            let planar_transformed_max_velocity = if let Some(ground_normal) = self.ground_normal()
            {
                project_on_plane(&transformed_max_velocity, &ground_normal)
            } else {
                Vector3::new(
                    transformed_max_velocity.x,
                    current_velocity.y,
                    transformed_max_velocity.z,
                )
            };
            // Calculate the velocity that the body will have *after*
            // this frame
            let frame_goal_velocity = move_towards(
                &current_velocity,
                &planar_transformed_max_velocity,
                max_move_acceleration * delta_seconds,
            );
            // Only grab the acceleration that we need to apply from the previous calculation
            let frame_acceleration = frame_goal_velocity - current_velocity;
            // Apply the acceleration.
            // This is basically `body.linvel += frame_acceleration * delta_seconds`
            // where frame_acceleration scales to ensure we never voluntarily exceed our
            // max velocity and frame_acceleration itself never exceeds our max move acceleration
            body.apply_impulse(frame_acceleration * body.mass(), true);
        }
    }

    pub fn move_body_continuous(
        &mut self,
        config: &CharacterControllerConfig,
        delta_seconds: f32,
        left_right_magnitude: f32,
        forward_back_magnitude: f32,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let max_move_speed = match self.crouch_state.current_state() {
            CrouchState::Upright => config.max_standing_move_speed_continuous,
            CrouchState::Crouched => config.max_crouched_move_speed_continuous,
        };
        let max_move_acceleration = match self.crouch_state.current_state() {
            CrouchState::Upright => config.max_standing_move_acceleration_continuous,
            CrouchState::Crouched => config.max_crouched_move_acceleration_continuous,
        };
        let movement_vector = Vector3::new(left_right_magnitude, 0.0, forward_back_magnitude);
        let trying_to_move = movement_vector.magnitude() > 0.0;
        let max_velocity: Vector<f32> = if trying_to_move {
            // If we don't have this check, we'd be dividing 0 by 0 and
            // have a vector of NaNs
            movement_vector.cap_magnitude(1.0) * max_move_speed
        } else {
            movement_vector * max_move_speed
        };
        self.move_body(
            &max_velocity,
            delta_seconds,
            max_move_acceleration,
            rigid_body_set,
        );
    }

    pub fn move_body_discrete(
        &mut self,
        config: &CharacterControllerConfig,
        delta_seconds: f32,
        left_right_magnitude: f32,
        forward_back_magnitude: f32,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let movement_vector = Vector3::new(left_right_magnitude, 0.0, forward_back_magnitude);
        let trying_to_move = movement_vector.magnitude() > 0.0;
        let capped_movement_vector = if trying_to_move {
            movement_vector.cap_magnitude(1.0)
        } else {
            movement_vector
        };
        let is_moving_forward = match self.perspective_mode.current_state() {
            CharacterPerspectiveMode::FirstPerson | CharacterPerspectiveMode::ThirdPersonBasic => {
                true
            }
            CharacterPerspectiveMode::ThirdPersonCombat => {
                capped_movement_vector.angle(&FORWARD_VECTOR).to_degrees()
                    <= config.max_sprint_forward_angle_threshold_discrete
            }
        };
        let movement_mag = capped_movement_vector.magnitude();
        let move_speed = match self.crouch_state.current_state() {
            CrouchState::Upright => {
                if is_moving_forward {
                    if movement_mag >= config.standing_sprint_input_threshold {
                        config.standing_sprint_speed_discrete
                    } else if movement_mag >= config.standing_run_input_threshold {
                        config.standing_run_speed_discrete
                    } else {
                        config.standing_walk_speed_discrete
                    }
                } else {
                    config.standing_walk_speed_discrete
                }
            }
            CrouchState::Crouched => config.crouched_creep_speed_discrete,
        };
        let move_acceleration = match self.crouch_state.current_state() {
            CrouchState::Upright => {
                if is_moving_forward && movement_mag > config.standing_sprint_input_threshold {
                    config.standing_sprint_acceleration_discrete
                } else if movement_mag > config.standing_run_input_threshold {
                    config.standing_run_acceleration_discrete
                } else {
                    config.standing_walk_acceleration_discrete
                }
            }
            CrouchState::Crouched => config.crouched_creep_acceleration_discrete,
        };

        self.move_body(
            &(capped_movement_vector * move_speed),
            delta_seconds,
            move_acceleration,
            rigid_body_set,
        );
    }

    fn jump(&mut self, config: &CharacterControllerConfig, rigid_body_set: &mut RigidBodySet) {
        let jump_acceleration = match self.crouch_state.current_state() {
            CrouchState::Upright => config.jump_standing_acceleration,
            CrouchState::Crouched => match self.sliding_state.current_state() {
                SlidingState::None => config.jump_crouched_acceleration,
                SlidingState::Normal => 0.0,
                SlidingState::Downhill => config.jump_standing_acceleration,
            },
        };
        self.jump_body(config, jump_acceleration, rigid_body_set);
        self.event_channel.send(CharacterControllerEvent::Jump);
        self.jump_cooldown_timer.reset();
    }

    /// Make the character controller's rigid body jump. If the character controller is wallrunning, it will jump on the
    /// opposite direction of the wall it's running on. If not wallrunning, it will jump straight up.
    fn jump_body(
        &mut self,
        &config: &CharacterControllerConfig,
        jump_acceleration: f32,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let body_handle = self.body_handle();
        let body_isometry = self.body_isometry();
        // We always wanna jump up and forward
        let untransformed_jump_direction_vector = Vector3::new(0.0, 1.0, -0.5).normalize();
        // And also away from the wall
        let jump_vector = match self.wallrunning_state.current_state() {
            WallRunning::OnRight(untransformed_wall_normal) => {
                untransformed_wall_normal * config.jump_wallrunning_normal_scale
                    + untransformed_jump_direction_vector * config.jump_wallrunning_scale
            }
            WallRunning::OnLeft(untransformed_wall_normal) => {
                untransformed_wall_normal * config.jump_wallrunning_normal_scale
                    + untransformed_jump_direction_vector * config.jump_wallrunning_scale
            }
            WallRunning::None => UP_VECTOR,
        } * jump_acceleration;
        let current_velocity = self.body_linear_velocity();
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            let transformed_jump_vector = body_isometry.transform_vector(&jump_vector);
            body.reset_forces(true);
            // If the body is moving down enough, then
            // we cancel the vertical velocity so the jump impulse isn't
            // canceled out by the existing downward movement.
            body.set_linvel(
                Vector3::new(current_velocity.x, 0.0, current_velocity.z),
                true,
            );
            body.apply_impulse(transformed_jump_vector * body.mass(), true);
        }
    }

    fn determine_linear_velocity(&mut self, rigid_body_set: &mut RigidBodySet) {
        let body_handle = self.body_handle();
        if let Some(body) = rigid_body_set.get(body_handle) {
            self.body_linear_velocity = *body.linvel();
        }
    }

    /// Determine whether the character controller has a collider just below it, functioning
    /// as ground. Also calculate the normal of this surface.
    fn determine_grounded_state(
        &mut self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
        query_pipeline: &mut QueryPipeline,
        collider_set: &mut ColliderSet,
    ) {
        let body_handle = self.body_handle();
        let body_isometry = self.body_isometry();
        let query_filter = QueryFilter::new();
        let (cap_halfheight, cap_radius) = self.capsule_values(config);

        if rigid_body_set.get_mut(body_handle).is_some() {
            if let Some((_, shape_hit)) = query_pipeline.cast_shape(
                rigid_body_set,
                collider_set,
                body_isometry,
                &DOWN_VECTOR,
                &Capsule::new_y(cap_halfheight, cap_radius),
                config.ground_ray_length,
                true,
                query_filter.exclude_collider(self.collider_handle()), // query_filter_excluding_player(),
            ) {
                self.ground_normal = Some(*shape_hit.normal1);
                return;
            }
        }
        self.ground_normal = None;
    }

    /// Determine whether the character controller has a collider on the right or left side by firing a ray in those directions.
    ///
    /// Note: This will update the state *regardless* of whether the character controller is grounded, so you must ensure by
    /// yourself that the character controller isn't already grounded when responding to this state.
    fn determine_wallrunning_state(
        &mut self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
        player_radius: f32,
        query_pipeline: &mut QueryPipeline,
        collider_set: &mut ColliderSet,
    ) {
        if self.crouch_state.current_state() == &CrouchState::Crouched {
            self.wallrunning_state.transition_to(WallRunning::None);
            return;
        }
        let body_handle = self.body_handle();
        let body_isometry = self.body_isometry();
        let ray_distance_from_body = config.wallrunning_ray_length;
        let body_linear_velocity = self.body_linear_velocity();
        if rigid_body_set.get(body_handle).is_some() {
            // Can only wallrun if moving forward enough
            let transformed_forward_vector = self.body_isometry().transform_vector(&FORWARD_VECTOR);
            if body_linear_velocity
                .angle(&transformed_forward_vector)
                .to_degrees()
                > config.max_wallrunning_forward_angle
            {
                self.wallrunning_state.transition_to(WallRunning::None);
                return;
            }

            let right_wall_ray =
                Ray::new(point![0.0, 0.0, 0.0], RIGHT_VECTOR).transform_by(&body_isometry);

            if let Some((_handle, ray_intersection)) = query_pipeline.cast_ray_and_get_normal(
                rigid_body_set,
                collider_set,
                &right_wall_ray,
                (player_radius - COLLIDER_RAYCAST_OFFSET) + ray_distance_from_body,
                false,
                query_filter_excluding_player(),
            ) {
                let ray_normal = ray_intersection.normal;
                let transformed_wall_normal = (-right_wall_ray.dir + ray_normal).normalize();
                let wall_normal = body_isometry.inverse_transform_vector(&transformed_wall_normal);
                self.wallrunning_state
                    .transition_to(WallRunning::OnRight(wall_normal));
                return;
            }

            let left_wall_ray =
                Ray::new(point![0.0, 0.0, 0.0], LEFT_VECTOR).transform_by(&body_isometry);
            if let Some((_handle, ray_intersection)) = query_pipeline.cast_ray_and_get_normal(
                rigid_body_set,
                collider_set,
                &left_wall_ray,
                (player_radius - COLLIDER_RAYCAST_OFFSET) + ray_distance_from_body,
                false,
                query_filter_excluding_player(),
            ) {
                let ray_normal = ray_intersection.normal;
                let transformed_wall_normal = (-left_wall_ray.dir + ray_normal).normalize();
                let wall_normal = body_isometry.inverse_transform_vector(&transformed_wall_normal);
                self.wallrunning_state
                    .transition_to(WallRunning::OnLeft(wall_normal));
                return;
            }
        }
        self.wallrunning_state.transition_to(WallRunning::None);
    }

    fn determine_sliding_state(&mut self, config: &CharacterControllerConfig) {
        let body_isometry = self.body_isometry();
        if let Some(ground_normal) = self.ground_normal() {
            let planar_forward = project_on_plane(&FORWARD_VECTOR, &ground_normal);
            let on_slope = ground_normal.angle(&UP_VECTOR).to_degrees()
                <= config.endless_slide_ground_normal_max_up_angle;
            let moving_downhill = self.body_linear_velocity().angle(&DOWN_VECTOR).to_degrees()
                <= config.endless_slide_downhill_max_down_angle;

            let is_sliding = self.crouch_state.current_state() == &CrouchState::Crouched
                && self
                    .body_linear_velocity
                    .angle(&body_isometry.transform_vector(&planar_forward))
                    .to_degrees()
                    <= config.sliding_max_forward_angle;
            let sliding_type = if on_slope && moving_downhill {
                SlidingState::Downhill
            } else if self.body_linear_velocity().magnitude()
                >= config.sliding_speed_factor * config.max_standing_move_speed_continuous
            {
                SlidingState::Normal
            } else {
                SlidingState::None
            };

            if is_sliding {
                self.sliding_state.transition_to(sliding_type);
            } else {
                self.sliding_state.transition_to(SlidingState::None);
            }
        }
    }

    /// Tilt the head of the character controller about the Z axis based on the current wall running state.
    /// If the character controller is on a wall on the right, tilt the head left. If the wall is on the left, tilt
    /// the head right. If not wall running, don't tilt the head.
    fn tilt_head(&mut self, config: &CharacterControllerConfig, delta_seconds: f32) {
        let z_axis = Unit::new_normalize(BACK_VECTOR);
        let max_tilt = 10.0f32.to_radians();
        let target_head_z_rotation = if !self.is_grounded() {
            match self.wallrunning_state.current_state() {
                WallRunning::OnRight(_) => UnitQuaternion::from_axis_angle(&z_axis, max_tilt),
                WallRunning::OnLeft(_) => UnitQuaternion::from_axis_angle(&z_axis, -max_tilt),
                WallRunning::None => UnitQuaternion::from_axis_angle(&z_axis, 0.0),
            }
        } else {
            UnitQuaternion::from_axis_angle(&z_axis, 0.0)
        };
        let tilt_speed = if self.wallrunning_state == WallRunning::None {
            config.enter_head_tilt_factor
        } else {
            config.exit_head_tilt_factor
        };
        self.head_z_rotation = self
            .head_z_rotation
            .try_slerp(&target_head_z_rotation, framerate_independent_interp_t(tilt_speed, delta_seconds), 0.0).expect("Could not tilt character_controller head as found and desired quaternions were 180 degrees apart");
    }

    /// Determine whether the character controller can stand up by casting a ray straight above the head.
    fn can_stand_up(
        &self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
        query_pipeline: &mut QueryPipeline,
        collider_set: &mut ColliderSet,
        is_grounded: bool,
    ) -> bool {
        let standing_collider = self.build_collider(
            config,
            config.capsule_standing_half_height(),
            config.capsule_standing_radius,
        );
        let standing_shape = standing_collider.shape();
        let distance_between_standing_and_crouched_heights =
            config.capsule_standing_total_height - config.capsule_crouched_total_height;
        if let Some(body) = rigid_body_set.get(self.body_handle()) {
            let next_body_isometry = body.next_position();
            let standing_trans =
                (next_body_isometry * self.head_standing_isometry(config)).translation;
            let crouched_trans =
                (next_body_isometry * self.head_crouched_isometry(config)).translation;
            let mut standing_isometry = *self.body_isometry();

            if is_grounded {
                standing_isometry.translation.y += distance_between_standing_and_crouched_heights;
                if query_pipeline
                    .cast_shape(
                        rigid_body_set,
                        collider_set,
                        &standing_isometry,
                        &UP_VECTOR,
                        standing_shape,
                        0.0,
                        false,
                        query_filter_excluding_player(),
                    )
                    .is_some()
                {
                    return false;
                }
            } else {
                standing_isometry.translation.y -=
                    (standing_trans.vector - crouched_trans.vector).y;
                if query_pipeline
                    .cast_shape(
                        rigid_body_set,
                        collider_set,
                        &standing_isometry,
                        &DOWN_VECTOR,
                        standing_shape,
                        0.0,
                        false,
                        query_filter_excluding_player(),
                    )
                    .is_some()
                {
                    return false;
                }
            }
        }
        true
    }

    /// Change the size of the character controller collider.
    fn change_crouch_state(
        &mut self,
        config: &CharacterControllerConfig,
        new_capsule_half_height: f32,
        new_capsule_radius: f32,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        island_manager: &mut IslandManager,
    ) {
        let distance_between_standing_and_crouched_heights =
            config.capsule_standing_total_height - config.capsule_crouched_total_height;

        if let Some(body) = rigid_body_set.get_mut(self.body_handle()) {
            let next_body_isometry = body.next_position();
            let mut new_pos = *next_body_isometry;
            let standing_trans =
                (next_body_isometry * self.head_standing_isometry(config)).translation;
            let crouched_trans =
                (next_body_isometry * self.head_crouched_isometry(config)).translation;

            // Toggle crouch state
            self.crouch_state
                .transition_to(match self.crouch_state.current_state() {
                    CrouchState::Upright => {
                        if self.is_grounded() {
                            // Put the smaller collider straight on the ground
                            new_pos.translation.y -=
                                distance_between_standing_and_crouched_heights / 2.0;
                        } else {
                            new_pos.translation.y +=
                                (standing_trans.vector - crouched_trans.vector).y;
                        }
                        body.set_position(new_pos, true);
                        CrouchState::Crouched
                    }
                    CrouchState::Crouched => {
                        if self.is_grounded() {
                            // Prevent any intersections between the larger collider and the ground
                            new_pos.translation.y +=
                                distance_between_standing_and_crouched_heights / 2.0;
                        } else {
                            new_pos.translation.y -=
                                (standing_trans.vector - crouched_trans.vector).y;
                        }
                        body.set_position(new_pos, true);
                        // // Or something like this
                        // body.add_force(Vector3::new(0.0,  125.0,  0.0), true);
                        CrouchState::Upright
                    }
                });
            let new_collider =
                self.build_collider(config, new_capsule_half_height, new_capsule_radius);
            collider_set.remove(self.collider_handle(), island_manager, rigid_body_set, true);
            self.set_collider_handle(collider_set.insert_with_parent(
                new_collider,
                self.body_handle(),
                rigid_body_set,
            ));
        }
    }

    fn start_wallrunning(
        &mut self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let current_velocity = self.body_linear_velocity();
        if let Some(body) = rigid_body_set.get_mut(self.body_handle()) {
            body.reset_forces(true);
            body.set_gravity_scale(config.start_wallrunning_gravity_scale, true);
            let new_linvel = Vector3::new(
                current_velocity.x,
                config.start_wallrunning_up_impulse,
                current_velocity.z,
            );
            body.set_linvel(new_linvel, true);
        }
    }

    fn stop_wallrunning(&mut self, rigid_body_set: &mut RigidBodySet) {
        if let Some(body) = rigid_body_set.get_mut(self.body_handle()) {
            body.set_gravity_scale(1.0, true);
        }
    }

    fn start_sliding(
        &mut self,
        config: &CharacterControllerConfig,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let body_isometry = self.body_isometry();
        if let Some(body) = rigid_body_set.get_mut(self.body_handle()) {
            body.reset_forces(true);

            match *self.sliding_state.current_state() {
                SlidingState::Downhill => {
                    let planar_endless_sliding_acceleration = project_on_plane(
                        &config.endless_sliding_acceleration.into(),
                        &self.ground_normal().unwrap(),
                    );
                    let transformed_endless_sliding_acceleration =
                        body_isometry.transform_vector(&planar_endless_sliding_acceleration);
                    body.add_force(transformed_endless_sliding_acceleration * body.mass(), true);
                }
                SlidingState::Normal => {
                    let planar_sliding_deceleration = project_on_plane(
                        &config.sliding_deceleration.into(),
                        &self.ground_normal().unwrap(),
                    );
                    let transformed_sliding_deceleration =
                        body_isometry.transform_vector(&planar_sliding_deceleration);
                    let planar_sliding_velocity_increase = project_on_plane(
                        &config.sliding_velocity_increase.into(),
                        &self.ground_normal().as_ref().unwrap(),
                    );
                    let transformed_sliding_velocity_increase =
                        body_isometry.transform_vector(&planar_sliding_velocity_increase);
                    body.apply_impulse(transformed_sliding_velocity_increase * body.mass(), true);
                    body.add_force(transformed_sliding_deceleration * body.mass(), true);
                }
                _ => {}
            }
        }
    }

    fn stop_sliding(&mut self, rigid_body_set: &mut RigidBodySet) {
        if let Some(body) = rigid_body_set.get_mut(self.body_handle()) {
            if matches!(self.sliding_state.current_state(), &SlidingState::Downhill) {
                body.reset_forces(true);
            }
        }
    }
}
