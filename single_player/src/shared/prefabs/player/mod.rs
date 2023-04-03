use perigee::prelude::*;

use crate::config::player::PlayerConfig;
use crate::shared::controllers::character::utils::{CharacterPerspectiveMode, CrouchState};
use crate::shared::controllers::CharacterController;
use crate::shared::descriptor::Descriptor;
use crate::shared::events::CharacterControllerEvent;
use crate::shared::input::Input;
use crate::shared::settings::GameSettings;
use crate::shared::vectors::*;
use serde::{Deserialize, Serialize};
use utils::{MovementState, WalkDirection};

mod utils;

#[derive(Serialize, Deserialize)]
pub struct Player<'a> {
    pub descriptor: Descriptor<'a>,
    pub controller: CharacterController,
    movement_state: StateMachine<MovementState>,
    #[serde(skip)]
    event_channel: EventChannel<CharacterControllerEvent>,
    #[serde(skip)]
    animation_manager: AnimationManager,
}

impl<'a> FromConfig for Player<'a> {
    type Config<'b> = &'b PlayerConfig;

    fn from_config<'b>(config: Self::Config<'b>) -> Self {
        Self {
            controller: CharacterController::from_config(&config.character_controller),
            // [P]re-[C]onfigured [P]layer
            descriptor: Descriptor::from_name("PCP"),
            movement_state: StateMachine::new(MovementState::default()),
            event_channel: EventChannel::with_capacity(config.event_queue_capacity),
            animation_manager: AnimationManager::default(),
        }
    }
}

impl<'a> Player<'a> {
    pub fn initialize(
        &mut self,
        config: &PlayerConfig,
        gltf: &Gltf,
        physics: &mut PhysicsWorld,
        initial_isometry: Option<Isometry<f32, Unit<Quaternion<f32>>, 3>>,
        descriptor_string: Option<impl Into<Descriptor<'a>>>,
    ) {
        if let Some(new_object_name) = descriptor_string {
            self.descriptor = new_object_name.into();
        }
        self.controller.add_to_physics_world(
            &config.character_controller,
            &mut physics.rigid_body_set,
            &mut physics.collider_set,
            initial_isometry,
        );

        physics
            .named_rigid_bodies
            .insert(self.descriptor.as_ref(), self.controller.body_handle());

        let animation_manager = AnimationManager::import_from_gltf(gltf);
        self.animation_manager.extend(animation_manager);
        let _player_event_sender = self.event_channel.clone_sender();
        let on_run_step = move || {
            // player_event_sender
            //     .send(CharacterControllerEvent::Stepped)
            //     .unwrap();
        };
        self.animation_manager
            .get_mut("SPRINT_FORWARD")
            .unwrap()
            .animation
            .on_frame(5, on_run_step.clone());
        self.animation_manager
            .get_mut("SPRINT_FORWARD")
            .unwrap()
            .animation
            .on_frame(15, on_run_step);
        self.animation_manager.loop_animation(
            &self.movement_state.current_state().to_string(),
            Some(&self.scene_object_name().to_owned()),
        );
    }

    pub fn update(
        &mut self,
        config: &PlayerConfig,
        settings: &GameSettings,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        let previous_tick_movement_state = *self.movement_state.current_state();

        self.controller.update(
            &config.character_controller,
            settings,
            physics,
            input,
            delta_seconds,
        );

        if !self.controller.is_grounded() {
            self.nudge_in_air(
                &config,
                delta_seconds,
                input.move_right(),
                input.move_forward(),
                &mut physics.rigid_body_set,
            );

            if self.controller.perspective_mode.current_state()
                == &CharacterPerspectiveMode::ThirdPersonBasic
            {
                self.controller.face_body_in_moving_direction(
                    &config.character_controller,
                    input.move_right(),
                    input.move_forward(),
                    &mut physics.rigid_body_set,
                    delta_seconds,
                );
            }
        }
        self.determine_movement_state(config, &mut physics.rigid_body_set);

        if *self.movement_state.current_state() != previous_tick_movement_state {
            self.animation_manager.stop_animation(
                &previous_tick_movement_state.to_string(),
                Some(&self.scene_object_name().to_owned()),
            );
            self.animation_manager.loop_animation(
                &self.movement_state.current_state().to_string(),
                Some(&self.scene_object_name().to_owned()),
            );
        }

        self.animation_manager.update(delta_seconds);
    }

    pub fn scene_object_name(&self) -> &str {
        self.descriptor.object_name()
    }

    pub fn body_isometry(&self) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        self.controller.body_isometry()
    }

    pub fn get_event(&self) -> Result<CharacterControllerEvent, TryRecvError> {
        self.controller.get_event()
    }

    fn determine_movement_state(
        &mut self,
        config: &PlayerConfig,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let linvel = self.controller.body_linear_velocity();
        let body_handle = self.controller.body_handle();
        if let Some(body) = rigid_body_set.get(body_handle) {
            if !self.controller.is_grounded() {
                self.movement_state.transition_to(MovementState::InAir);
                return;
            }

            match self.controller.perspective_mode.current_state() {
                CharacterPerspectiveMode::ThirdPersonCombat => {
                    let inversely_transformed_linvel =
                        body.position().inverse_transform_vector(&linvel);
                    self.movement_state.transition_to(
                        if self.controller.crouch_state.current_state() == &CrouchState::Upright {
                            if inversely_transformed_linvel
                                .angle(&FORWARD_VECTOR)
                                .to_degrees()
                                <= config
                                    .character_controller
                                    .max_sprint_forward_angle_threshold_discrete
                                && linvel.magnitude()
                                    >= config.character_controller.standing_sprint_speed_discrete
                                        * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Sprinting
                            } else if linvel.magnitude()
                                >= config.character_controller.standing_run_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Running
                            } else if linvel.magnitude()
                                >= config.character_controller.standing_walk_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                let isometry_inverted_linvel = self
                                    .controller
                                    .body_isometry()
                                    .inverse_transform_vector(&linvel);
                                let walk_direction = WalkDirection::from_movement_vector(
                                    &isometry_inverted_linvel,
                                )
                                .expect(
                                    "Can't get walk direction when movement vector has 0 magnitude",
                                );
                                MovementState::Walking(walk_direction)
                            } else {
                                MovementState::Stationary(
                                    self.controller.crouch_state.current_state().clone(),
                                )
                            }
                        } else {
                            if linvel.magnitude()
                                >= config.character_controller.crouched_creep_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Creeping
                            } else {
                                MovementState::Stationary(
                                    self.controller.crouch_state.current_state().clone(),
                                )
                            }
                        },
                    );
                }
                CharacterPerspectiveMode::ThirdPersonBasic
                | CharacterPerspectiveMode::FirstPerson => {
                    self.movement_state.transition_to(
                        if self.controller.crouch_state.current_state() == &CrouchState::Upright {
                            if linvel.magnitude()
                                >= config.character_controller.standing_sprint_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Sprinting
                            } else if linvel.magnitude()
                                >= config.character_controller.standing_run_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Running
                            } else if linvel.magnitude()
                                >= config.character_controller.standing_walk_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Walking(WalkDirection::Forward)
                            } else {
                                MovementState::Stationary(
                                    self.controller.crouch_state.current_state().clone(),
                                )
                            }
                        } else {
                            if linvel.magnitude()
                                >= config.character_controller.crouched_creep_speed_discrete
                                    * config.character_controller.discrete_movement_factor
                            {
                                MovementState::Creeping
                            } else {
                                MovementState::Stationary(
                                    self.controller.crouch_state.current_state().clone(),
                                )
                            }
                        },
                    );
                }
            };
        }
    }

    fn nudge_in_air(
        &mut self,
        config: &PlayerConfig,
        delta_seconds: f32,
        left_right_magnitude: f32,
        forward_back_magnitude: f32,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let perspective_mode = self.controller.perspective_mode.current_state();
        if let Some(body) = rigid_body_set.get_mut(self.controller.body_handle()) {
            let pivot_isometry = match perspective_mode {
                CharacterPerspectiveMode::ThirdPersonBasic
                | CharacterPerspectiveMode::ThirdPersonCombat => Isometry::from_parts(
                    self.controller.boom.translation,
                    self.controller.boom.z_rotation,
                ),
                CharacterPerspectiveMode::FirstPerson => *body.position(),
            };

            let movement_vector = Vector3::new(left_right_magnitude, 0.0, forward_back_magnitude)
                .cap_magnitude(1.0)
                * body.mass()
                * config.aerial_max_move_acceleration
                * delta_seconds;
            body.apply_impulse(pivot_isometry.transform_vector(&movement_vector), true);
        }
    }
}
