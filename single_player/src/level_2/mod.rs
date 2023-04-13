use crate::config::Level2Config;
use crate::shared::{input::Input, prefabs::Sedan, settings::GameSettings};

use events::Level2Event;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};

mod events;

extern "C" {
    fn level_event_hook(event_type_ptr: *const u8, event_type_len: usize);
}

#[derive(Serialize, Deserialize)]
pub struct Sim<'a> {
    version: (u8, u8, u8),
    config: Level2Config,
    pub settings: GameSettings,
    pub physics: PhysicsWorld,
    pois: PointsOfInterest,
    pub car: Sedan<'a>,
    scene_gltf_bytes: &'a [u8],
    #[serde(skip)]
    pub input: Input,
}

impl<'a> FromConfig for Sim<'a> {
    type Config<'b> = Level2Config;

    fn from_config<'b>(config: Self::Config<'b>) -> Self {
        let physics = PhysicsWorld::from_config(&config.physics);
        let car = Sedan::from_config(&config.car);

        Self {
            version: (0, 0, 0),
            config,
            car,
            physics,
            settings: GameSettings::default(),
            input: Input::default(),
            scene_gltf_bytes: include_bytes!("../../../assets/gltf/levels/2/scene.glb"),
            pois: PointsOfInterest::default(),
        }
    }

    fn set_config<'b>(&mut self, _config: Self::Config<'b>) {
        warn!("Level 2 Sim doesn't allow resetting configuration");
    }
}

// Simple setup and accessors
impl<'a> Sim<'a> {
    pub fn scene_gltf_bytes(&self) -> &[u8] {
        self.scene_gltf_bytes
    }

    pub fn send_level_event(&self, evt: Level2Event) {
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

        self.car.initialize(
            &self.config.car,
            &mut self.physics,
            Some(self.pois["Test Area Start"]),
            Some(String::from("Sedan")),
        );

        loop_audio(self.car.scene_object_name(), "LEVEL_MUSIC", 1.0, 0.2);
    }
}

#[ffi]
impl<'a> Sim<'a> {
    pub fn scene_gltf_bytes_ptr(&self) -> *const u8 {
        self.scene_gltf_bytes().as_ptr()
    }

    pub fn scene_gltf_bytes_len(&self) -> usize {
        self.scene_gltf_bytes().len()
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

    #[slot_return]
    pub fn poi(&self, poi_name: &str) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.pois[poi_name]
    }

    // Making this an FFI-only wrapper because if the WASM has a
    // function "initialize" it's not obvious what type it's initializing.
    pub fn initialize_sim(&mut self) {
        self.initialize();
    }

    pub fn desired_fps(&self) -> f32 {
        30.0
    }

    /// Step the game simulation by the provided number of seconds.
    pub fn step(&mut self, delta_seconds: f32) {
        self.car.update(
            &self.config.car,
            &self.settings,
            &self.input,
            &mut self.physics,
            delta_seconds,
        );

        self.physics.step(delta_seconds);

        self.input.wipe();
    }

    pub fn settings_left_right_look_sensitivity(&self) -> u8 {
        self.settings.left_right_look_sensitivity()
    }

    pub fn settings_up_down_look_sensitivity(&self) -> u8 {
        self.settings.up_down_look_sensitivity()
    }

    pub fn settings_set_left_right_look_sensitivity(&mut self, new_sensitivity: i32) {
        self.settings
            .set_left_right_look_sensitivity(new_sensitivity as u8);
    }

    pub fn settings_set_up_down_look_sensitivity(&mut self, new_sensitivity: i32) {
        self.settings
            .set_up_down_look_sensitivity(new_sensitivity as u8);
    }

    pub fn input_set_move_forward(&mut self, new_magnitude: f32) {
        self.input.set_move_forward(new_magnitude);
    }

    pub fn input_set_move_right(&mut self, new_magnitude: f32) {
        self.input.set_move_right(new_magnitude);
    }

    pub fn input_set_rotate_up(&mut self, new_magnitude: f32) {
        self.input.set_rotate_up(new_magnitude);
    }

    pub fn input_set_rotate_right(&mut self, new_magnitude: f32) {
        self.input.set_rotate_right(new_magnitude);
    }

    pub fn input_set_jump(&mut self, _new_magnitude: f32) {}
    pub fn input_set_aim(&mut self, _new_magnitude: f32) {}

    #[slot_return]
    pub fn camera_global_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.car.controller.camera_isometry()
    }

    #[slot_return]
    pub fn car_cabin_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        *self.car.controller.cabin_isometry()
    }

    #[slot_return]
    pub fn wheel_isometry(&self, wheel_idx: u32) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.car.controller.wheel_isometry(wheel_idx as usize)
    }
}

#[no_mangle]
pub extern "C" fn destroy_sim(sim_ptr: *mut Sim) {
    // Box will deallocate the memory on drop
    unsafe { Box::from_raw(sim_ptr) };
}

#[no_mangle]
pub extern "C" fn create_sim() -> *mut Sim<'static> {
    init_perigee_logger();
    Box::into_raw(Box::new(Sim::from_config(Level2Config::default())))
}
