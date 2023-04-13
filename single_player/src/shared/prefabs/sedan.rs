use perigee::prelude::*;

use crate::config::SedanConfig;
use crate::shared::controllers::RaycastVehicleController;
use crate::shared::descriptor::Descriptor;
use crate::shared::input::Input;
use crate::shared::settings::GameSettings;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Sedan<'a> {
    pub descriptor: Descriptor<'a>,
    pub controller: RaycastVehicleController,
}

impl<'a> FromConfig for Sedan<'a> {
    type Config<'b> = &'b SedanConfig;

    fn from_config<'b>(config: Self::Config<'b>) -> Self {
        Self {
            controller: RaycastVehicleController::from_config(&config.raycast_vehicle_controller),
            // [P]re-[C]onfigured [S]edan
            descriptor: Descriptor::from_name("PCS"),
        }
    }
}

impl<'a> Sedan<'a> {
    pub fn initialize(
        &mut self,
        config: &SedanConfig,
        physics: &mut PhysicsWorld,
        initial_isometry: Option<Isometry<f32, Unit<Quaternion<f32>>, 3>>,
        descriptor_string: Option<impl Into<Descriptor<'a>>>,
    ) {
        self.controller.add_to_physics_world(
            &config.raycast_vehicle_controller,
            &mut physics.rigid_body_set,
            &mut physics.collider_set,
            initial_isometry,
        );

        if let Some(descriptor) = descriptor_string {
            self.descriptor = descriptor.into();
        }
    }

    pub fn scene_object_name(&self) -> &str {
        self.descriptor.object_name()
    }

    pub fn update(
        &mut self,
        config: &SedanConfig,
        settings: &GameSettings,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        self.controller.update(
            &config.raycast_vehicle_controller,
            settings,
            input,
            physics,
            delta_seconds,
        );
    }
}
