use perigee::prelude::*;

use crate::config::player::PlayerConfig;
use crate::shared::controllers::character::utils::{CrouchState, PerspectiveMode};
use crate::shared::controllers::CharacterController;
use crate::shared::events::CharacterControllerEvent;
use crate::shared::input::Input;
use crate::shared::settings::GameSettings;
use crate::shared::vectors::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use utils::{MovementState, WalkDirection};

mod utils;

#[derive(Serialize, Deserialize)]
pub struct Player {
    config: Rc<PlayerConfig>,
    pub controller: CharacterController,
    scene_object_name: String,

    movement_state: StateMachine<MovementState>,
    #[serde(skip)]
    event_channel: EventChannel<CharacterControllerEvent>,
    #[serde(skip)]
    animation_manager: AnimationManager,
}

impl FromConfig for Player {
    type Config<'a> = &'a Rc<PlayerConfig>;

    fn from_config<'a>(config: Self::Config<'a>) -> Self {
        Self {
            config: Rc::clone(config),
            controller: CharacterController::from_config(config.character_controller()),
            // [P]re-[C]onfigured [P]layer
            scene_object_name: String::from("PCP"),
            movement_state: StateMachine::new(MovementState::default()),
            event_channel: EventChannel::with_capacity(config.event_queue_capacity()),
            animation_manager: AnimationManager::default(),
        }
    }

    fn set_config<'a>(&mut self, _config: Self::Config<'a>) {
        self.config = Rc::clone(_config);
    }
}

impl Player {
    pub fn initialize(
        &mut self,
        gltf: &Gltf,
        physics: &mut PhysicsWorld,
        initial_isometry: Option<Isometry<f32, Unit<Quaternion<f32>>, 3>>,
        scene_object_name: Option<String>,
    ) {
        if let Some(new_object_name) = scene_object_name {
            self.scene_object_name = new_object_name;
        }
        self.controller.add_to_physics_world(
            &mut physics.rigid_body_set,
            &mut physics.collider_set,
            initial_isometry,
        );

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
            Some(&self.scene_object_name().clone()),
        );
    }

    pub fn update(
        &mut self,
        settings: &GameSettings,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        let previous_tick_movement_state = *self.movement_state.current_state();

        self.controller
            .update(delta_seconds, input, settings, physics);

        self.determine_movement_state(&mut physics.rigid_body_set);

        if *self.movement_state.current_state() != previous_tick_movement_state {
            self.animation_manager.stop_animation(
                &previous_tick_movement_state.to_string(),
                Some(&self.scene_object_name().clone()),
            );
            self.animation_manager.loop_animation(
                &self.movement_state.current_state().to_string(),
                Some(&self.scene_object_name().clone()),
            );
        }

        self.animation_manager.update(delta_seconds);
    }

    pub fn scene_object_name(&self) -> &String {
        &self.scene_object_name
    }

    pub fn body_isometry(&self) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        self.controller.body_isometry()
    }

    pub fn get_event(&self) -> Result<CharacterControllerEvent, TryRecvError> {
        self.controller.get_event()
    }

    fn determine_movement_state(&mut self, rigid_body_set: &mut RigidBodySet) {
        let linvel = self.controller.body_linear_velocity();
        let body_handle = self.controller.body_handle();
        if let Some(body) = rigid_body_set.get(body_handle) {
            if !self.controller.is_grounded() {
                self.movement_state.transition_to(MovementState::InAir);
                return;
            }

            match self.controller.perspective_mode() {
                PerspectiveMode::ThirdPersonCombat => {
                    let inversely_transformed_linvel =
                        body.position().inverse_transform_vector(&linvel);
                    self.movement_state.transition_to(
                        if self.controller.crouch_state() == &CrouchState::Upright {
                            if inversely_transformed_linvel
                                .angle(&FORWARD_VECTOR)
                                .to_degrees()
                                <= self
                                    .config
                                    .character_controller()
                                    .max_sprint_forward_angle_threshold_discrete()
                                && linvel.magnitude()
                                    >= self
                                        .config
                                        .character_controller()
                                        .standing_sprint_speed_discrete()
                                        * self
                                            .config
                                            .character_controller()
                                            .discrete_movement_factor()
                            {
                                MovementState::Sprinting
                            } else if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .standing_run_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Running
                            } else if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .standing_walk_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
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
                                MovementState::Stationary(self.controller.crouch_state().clone())
                            }
                        } else {
                            if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .crouched_creep_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Creeping
                            } else {
                                MovementState::Stationary(self.controller.crouch_state().clone())
                            }
                        },
                    );
                }
                PerspectiveMode::ThirdPersonBasic | PerspectiveMode::FirstPerson => {
                    self.movement_state.transition_to(
                        if self.controller.crouch_state() == &CrouchState::Upright {
                            if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .standing_sprint_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Sprinting
                            } else if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .standing_run_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Running
                            } else if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .standing_walk_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Walking(WalkDirection::Forward)
                            } else {
                                MovementState::Stationary(self.controller.crouch_state().clone())
                            }
                        } else {
                            if linvel.magnitude()
                                >= self
                                    .config
                                    .character_controller()
                                    .crouched_creep_speed_discrete()
                                    * self
                                        .config
                                        .character_controller()
                                        .discrete_movement_factor()
                            {
                                MovementState::Creeping
                            } else {
                                MovementState::Stationary(self.controller.crouch_state().clone())
                            }
                        },
                    );
                }
            };
        }
    }
}
