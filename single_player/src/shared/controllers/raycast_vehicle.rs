use crate::config::raycast_vehicle::RaycastVehicleConfig;
use crate::shared::input::Input;
use perigee::rapier3d::control::DynamicRayCastVehicleController;
use perigee::{prelude::*, rapier3d::control::WheelTuning};
use serde::{Deserialize, Serialize};

fn default_rapier_vehicle() -> DynamicRayCastVehicleController {
    DynamicRayCastVehicleController::new(RigidBodyHandle::default())
}

#[derive(Serialize, Deserialize)]
pub struct RaycastVehicleController {
    cabin_body_handle: RigidBodyHandle,
    cabin_isometry: Isometry3<f32>,
    #[serde(skip, default = "default_rapier_vehicle")]
    rapier_vehicle: DynamicRayCastVehicleController,
}

impl FromConfig for RaycastVehicleController {
    type Config<'a> = &'a RaycastVehicleConfig;
    fn from_config<'a>(_config: Self::Config<'a>) -> Self {
        let cabin_body_handle = RigidBodyHandle::default();
        let rapier_vehicle = DynamicRayCastVehicleController::new(cabin_body_handle);
        Self {
            cabin_body_handle,
            rapier_vehicle,
            cabin_isometry: Isometry::default(),
        }
    }
}

impl RaycastVehicleController {
    pub fn add_to_physics_world(
        &mut self,
        config: &RaycastVehicleConfig,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        initial_isometry: Option<Isometry3<f32>>,
    ) {
        let initial_isometry = if let Some(initial_isometry) = initial_isometry {
            initial_isometry
        } else {
            Isometry::from(Vector3::new(-2.0, config.suspension_rest_length + 3.0, 0.0))
        };

        let rigid_body = RigidBodyBuilder::dynamic()
            .position(initial_isometry)
            .additional_mass_properties(MassProperties::new(
                config.cabin_center_of_mass,
                0.0,
                Vector3::zeros(),
            ))
            .build();
        let cabin_collider = ColliderBuilder::cuboid(
            config.cabin_half_width,
            config.cabin_half_height,
            config.cabin_half_length,
        )
        // Listen for *all* collision and intersection events with
        // this collider
        .active_events(ActiveEvents::COLLISION_EVENTS)
        // Set the mass (in kg, I think) of the collider
        .density(config.mass)
        .build();

        let cabin_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(cabin_collider, cabin_body_handle, rigid_body_set);
        self.cabin_body_handle = cabin_body_handle;

        self.rapier_vehicle = DynamicRayCastVehicleController::new(self.cabin_body_handle);
        let wheel_tuning = WheelTuning::from(config);
        for wheel in config.wheels.iter() {
            self.rapier_vehicle.add_wheel(
                Point::from(wheel.center_cabin_relative_position),
                -Vector3::y(),
                Vector3::x(),
                wheel
                    .suspension_rest_length
                    .unwrap_or(config.suspension_rest_length),
                wheel.radius.unwrap_or(config.wheel_radius),
                &wheel_tuning,
            );
        }
    }

    pub fn cabin_body_handle(&self) -> RigidBodyHandle {
        self.cabin_body_handle
    }

    pub fn cabin_isometry(&self) -> &Isometry3<f32> {
        &self.cabin_isometry
    }

    pub fn wheel_isometry(&self, wheel_idx: usize) -> Isometry3<f32> {
        let wheel = self.rapier_vehicle.wheels()[wheel_idx];
        Isometry::from_parts(
            wheel.center().into(),
            self.cabin_isometry().rotation
                * UnitQuaternion::from_euler_angles(0.0, wheel.steering, 0.0),
        )
    }

    pub fn update(
        &mut self,
        config: &RaycastVehicleConfig,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        let steer_angle = lerp(
            config.wheel_left_turn_angle,
            config.wheel_right_turn_angle,
            remap(input.steer(), -1.0, 1.0, 0.0, 1.0),
        );
        for (wheel_index, wheel) in self.rapier_vehicle.wheels_mut().iter_mut().enumerate() {
            let wheel_config = config.wheels[wheel_index];
            wheel.engine_force = 0.0;
            // if wheel_config.receives_power {
            wheel.engine_force += config.throttle_force * input.throttle();
            // }
            wheel.engine_force -= config.brake_force * input.brake();
            if wheel_config.steers_on_input {
                wheel.steering = steer_angle.to_radians();
            }
        }

        self.rapier_vehicle.update_vehicle(
            delta_seconds,
            &mut physics.rigid_body_set,
            &physics.collider_set,
            &physics.query_pipeline,
            QueryFilter::new().exclude_rigid_body(self.cabin_body_handle),
        );

        if let Some(cabin_body) = physics.rigid_body_set.get(self.cabin_body_handle) {
            self.cabin_isometry = *cabin_body.position();
        }
    }
}
