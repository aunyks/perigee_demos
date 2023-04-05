use crate::shared::{
    descriptor::Descriptor, input::Input, prefabs::Player, settings::GameSettings,
    vectors::FORWARD_VECTOR,
};
use crate::{config::Level1Config, shared::events::CharacterControllerEvent};
use events::Level1Event;
use moving_platform::MovingPlatform;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};

extern "C" {
    fn level_event_hook(event_type_ptr: *const u8, event_type_len: usize);
}

mod events;
mod moving_platform;

#[derive(Serialize, Deserialize)]
pub struct Sim<'a> {
    version: (u8, u8, u8),
    config: Level1Config,
    pub settings: GameSettings,
    pub physics: PhysicsWorld,
    pois: PointsOfInterest,
    pub player: Player<'a>,
    moving_platforms: [MovingPlatform<'a>; 2],
    scene_gltf_bytes: &'a [u8],
    player_gltf_bytes: &'a [u8],
    #[serde(skip)]
    animation_manager: AnimationManager,
    #[serde(skip)]
    player_event_channel: ColliderEventChannel,
    #[serde(skip)]
    launch_sensor_event_channel: ColliderEventChannel,
    #[serde(skip)]
    finish_sensor_event_channel: ColliderEventChannel,
    #[serde(skip)]
    pub input: Input,
}

impl<'a> FromConfig for Sim<'a> {
    type Config<'b> = Level1Config;

    fn from_config<'b>(config: Self::Config<'b>) -> Self {
        let physics = PhysicsWorld::from_config(&config.physics);
        let player = Player::from_config(&config.player);

        Self {
            version: (0, 0, 0),
            config,
            player,
            physics,
            settings: GameSettings::default(),
            input: Input::default(),
            scene_gltf_bytes: include_bytes!("../../../assets/gltf/levels/1/scene.glb"),
            player_gltf_bytes: include_bytes!("../../../assets/gltf/shared/player-character.glb"),
            pois: PointsOfInterest::default(),
            animation_manager: AnimationManager::default(),
            moving_platforms: [
                MovingPlatform::new(Descriptor::from_name("Plat 3"), "Plat 3 Sensor"),
                MovingPlatform::new(Descriptor::from_name("Plat 3 2"), "Plat 3 Sensor 2"),
            ],
            player_event_channel: ColliderEventChannel::default(),
            launch_sensor_event_channel: ColliderEventChannel::default(),
            finish_sensor_event_channel: ColliderEventChannel::default(),
        }
    }

    fn set_config<'b>(&mut self, _config: Self::Config<'b>) {
        warn!("Level 0 Sim doesn't allow resetting configuration");
    }
}

impl<'a> Default for Sim<'a> {
    fn default() -> Self {
        Self::from_config(Level1Config::default())
    }
}

// Simple setup and accessors
impl<'a> Sim<'a> {
    pub fn scene_gltf_bytes(&self) -> &[u8] {
        self.scene_gltf_bytes
    }

    pub fn player_gltf_bytes(&self) -> &[u8] {
        self.player_gltf_bytes
    }

    pub fn send_level_event(&self, evt: Level1Event) {
        let level_event = evt.as_ref();
        unsafe { level_event_hook(level_event.as_ptr(), level_event.len()) };
    }

    pub fn initialize(&mut self) {
        // Load static colliders using trimeshes extracted from geometries
        // within a glTF. This lets you create a level using your favoritte 3D
        // modeling tool.
        let scene_gltf = Gltf::from_slice(self.scene_gltf_bytes).unwrap();

        self.physics.load_from_gltf(&scene_gltf).unwrap();
        self.pois.load_from_gltf(&scene_gltf).unwrap();

        self.animation_manager
            .extend(AnimationManager::import_from_gltf(&scene_gltf));

        self.player.initialize(
            &self.config.player,
            &Gltf::from_slice(self.player_gltf_bytes).unwrap(),
            &mut self.physics,
            Some(self.pois["Player Start"]),
            Some(String::from("PLAYER")),
        );

        for platform in &mut self.moving_platforms {
            platform.initialize(
                vec![
                    self.pois["Plat 3 Start Point"],
                    self.pois["Plat 3 End Point"],
                ],
                &mut self.physics,
            );
        }

        self.physics.listen_to_collider(
            self.physics.named_sensors["Launch Sensor"],
            ColliderEventRelayer::from(self.launch_sensor_event_channel.clone_sender()),
        );

        self.physics.listen_to_collider(
            self.physics.named_sensors["Finish Sensor"],
            ColliderEventRelayer::from(self.finish_sensor_event_channel.clone_sender()),
        );

        self.physics.listen_to_collider(
            self.player.controller.collider_handle(),
            ColliderEventRelayer::from(self.player_event_channel.clone_sender()),
        );
    }
}

#[ffi]
impl<'a> Sim<'a> {
    #[ffi_only]
    pub fn scene_gltf_bytes_ptr(&self) -> *const u8 {
        self.scene_gltf_bytes().as_ptr()
    }

    #[ffi_only]
    pub fn scene_gltf_bytes_len(&self) -> usize {
        self.scene_gltf_bytes().len()
    }

    #[ffi_only]
    pub fn player_gltf_bytes_ptr(&self) -> *const u8 {
        self.player_gltf_bytes().as_ptr()
    }

    #[ffi_only]
    pub fn player_gltf_bytes_len(&self) -> usize {
        self.player_gltf_bytes().len()
    }

    #[slot_return]
    pub fn prop_isometry(&self, prop_name: &str) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        let prop_body_handle = self
            .physics
            .named_rigid_bodies
            .handle_with_name(prop_name)
            .expect("No prop has provided name.");
        self.physics
            .rigid_body_set
            .get(*prop_body_handle)
            .expect("Prop with provided name doesn't exist in physics world.")
            .position()
    }

    // Making this an FFI-only wrapper because if the WASM has a
    // function "initialize" it's not obvious what type it's initializing.
    #[ffi_only]
    pub fn initialize_sim(&mut self) {
        self.initialize();
    }

    pub fn desired_fps(&self) -> f32 {
        30.0
    }

    /// Step the game simulation by the provided number of seconds.
    pub fn step(&mut self, delta_seconds: f32) {
        self.animation_manager.update(delta_seconds);

        self.player.update(
            &self.config.player,
            &self.settings,
            &self.input,
            &mut self.physics,
            delta_seconds,
        );

        for platform in &mut self.moving_platforms {
            platform.update(&mut self.physics, delta_seconds);
        }

        self.physics.step(delta_seconds);

        while let Ok(launch_sensor_event) = self.launch_sensor_event_channel.get_message() {
            match launch_sensor_event {
                ColliderEvent::IntersectionStart(other) => {
                    let launch_direction = self.pois["Launch Iso"]
                        .rotation
                        .transform_vector(&FORWARD_VECTOR);

                    // Get the rigid body of the other collider if it exists
                    if let Some(other_body) = self
                        .physics
                        .collider_set
                        .get(other)
                        .and_then(|other_collider| other_collider.parent())
                        .filter(|other_body_handle| {
                            other_body_handle
                                == &self.physics.named_rigid_bodies[self.player.descriptor.as_ref()]
                        })
                        .and_then(|other_body_handle| {
                            self.physics.rigid_body_set.get_mut(other_body_handle)
                        })
                    {
                        other_body.apply_impulse(
                            launch_direction * self.config.launch_impulse * other_body.mass(),
                            true,
                        );
                    }
                }
                _ => {}
            }
        }

        while let Ok(finish_sensor_event) = self.finish_sensor_event_channel.get_message() {
            match finish_sensor_event {
                ColliderEvent::IntersectionStart(other) => {
                    // Get the rigid body of the other collider if it exists
                    if self
                        .physics
                        .collider_set
                        .get(other)
                        .and_then(|other_collider| other_collider.parent())
                        .filter(|other_body_handle| {
                            other_body_handle
                                == &self.physics.named_rigid_bodies[self.player.descriptor.as_ref()]
                        })
                        .is_some()
                    {
                        self.send_level_event(Level1Event::LevelCompleted);
                    }
                }
                _ => {}
            }
        }

        while let Ok(player_collider_event) = self.player_event_channel.get_message() {
            match player_collider_event {
                ColliderEvent::IntersectionStart(other) => {
                    // Get the rigid body of the other collider if it exists
                    if let Some(sensor_name) = self.physics.named_sensors.name_of_handle(&other) {
                        if Descriptor::from_name(sensor_name).has_tag("oob") {
                            debug!("out of bounds!");
                        }
                    }
                }
                _ => {}
            }
        }

        // Ease the pressure of this channel
        while let Ok(player_event) = self.player.get_event() {
            match player_event {
                CharacterControllerEvent::Stepped => {
                    play_audio(self.player.scene_object_name(), "STEP", 1.0)
                }
                CharacterControllerEvent::Jump => {
                    play_audio(self.player.scene_object_name(), "JUMP", 1.0)
                }
                CharacterControllerEvent::StartedWallRunning => {
                    loop_audio(self.player.scene_object_name(), "WALLRUN", 1.0)
                }
                CharacterControllerEvent::StoppedWallRunning => {
                    stop_audio(self.player.scene_object_name(), "WALLRUN")
                }
                CharacterControllerEvent::StartedSliding => {
                    loop_audio(self.player.scene_object_name(), "SLIDE", 1.0)
                }
                CharacterControllerEvent::StoppedSliding => {
                    stop_audio(self.player.scene_object_name(), "SLIDE")
                }
                _ => {}
            };
        }

        self.input.wipe();
    }

    #[ffi_only]
    pub fn settings_left_right_look_sensitivity(&self) -> u8 {
        self.settings.left_right_look_sensitivity()
    }

    #[ffi_only]
    pub fn settings_up_down_look_sensitivity(&self) -> u8 {
        self.settings.up_down_look_sensitivity()
    }

    #[ffi_only]
    pub fn settings_set_left_right_look_sensitivity(&mut self, new_sensitivity: i32) {
        self.settings
            .set_left_right_look_sensitivity(new_sensitivity as u8);
    }

    #[ffi_only]
    pub fn settings_set_up_down_look_sensitivity(&mut self, new_sensitivity: i32) {
        self.settings
            .set_up_down_look_sensitivity(new_sensitivity as u8);
    }

    #[ffi_only]
    pub fn input_set_move_forward(&mut self, new_magnitude: f32) {
        self.input.set_move_forward(new_magnitude);
    }

    #[ffi_only]
    pub fn input_set_move_right(&mut self, new_magnitude: f32) {
        self.input.set_move_right(new_magnitude);
    }

    #[ffi_only]
    pub fn input_set_rotate_up(&mut self, new_magnitude: f32) {
        self.input.set_rotate_up(new_magnitude);
    }

    #[ffi_only]
    pub fn input_set_rotate_right(&mut self, new_magnitude: f32) {
        self.input.set_rotate_right(new_magnitude);
    }

    #[ffi_only]
    pub fn input_set_jump(&mut self, jump_val: u8) {
        self.input.set_jump(jump_val > 0)
    }

    #[ffi_only]
    pub fn input_set_crouch(&mut self, crouch_val: u8) {
        self.input.set_crouch(crouch_val > 0)
    }

    #[ffi_only]
    pub fn input_set_aim(&mut self, aim_val: u8) {
        self.input.set_aim(aim_val > 0)
    }

    #[slot_return]
    pub fn camera_global_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        // The player's head position
        self.player.controller.camera_isometry()
    }

    #[slot_return]
    pub fn player_body_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        *self.player.body_isometry()
    }
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn destroy_sim(sim_ptr: *mut Sim) {
    // Box will deallocate the memory on drop
    unsafe { Box::from_raw(sim_ptr) };
}

#[cfg(feature = "ffi")]
#[no_mangle]
pub extern "C" fn create_sim() -> *mut Sim<'static> {
    init_perigee_logger();
    Box::into_raw(Box::new(Sim::default()))
}
