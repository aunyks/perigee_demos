use crate::config::car::CarConfig;
use crate::shared::boom::Boom;
use crate::shared::input::Input;
use crate::shared::interactions::InteractionGroup;
use crate::shared::settings::GameSettings;
use crate::shared::vectors::*;
use perigee::prelude::*;
use perigee::rapier3d::control::DynamicRayCastVehicleController;
use serde::{Deserialize, Serialize};

fn default_rapier_vehicle() -> DynamicRayCastVehicleController {
    DynamicRayCastVehicleController::new(RigidBodyHandle::default())
}

#[derive(Serialize, Deserialize)]
pub struct Car {
    camera_boom: Boom,
    rigid_body_handle: RigidBodyHandle,
    cabin_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
    #[serde(skip, default = "default_rapier_vehicle")]
    rapier_vehicle: DynamicRayCastVehicleController,
}

impl FromConfig for Car {
    type Config<'a> = &'a CarConfig;
    fn from_config<'a>(config: Self::Config<'a>) -> Self {
        let rigid_body_handle = RigidBodyHandle::default();
        let rapier_vehicle = DynamicRayCastVehicleController::new(rigid_body_handle);
        Self {
            rigid_body_handle,
            rapier_vehicle,
            cabin_isometry: Isometry::default(),
            camera_boom: Boom::new(
                config.max_boom_length,
                config.initial_boom_pitch_angle,
                config.initial_boom_yaw_angle,
                true,
            ),
        }
    }
}

impl Car {
    pub fn add_to_physics_world(
        &mut self,
        config: &CarConfig,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        initial_isometry: Option<Isometry<f32, UnitQuaternion<f32>, 3>>,
    ) {
        let initial_isometry = if let Some(initial_isometry) = initial_isometry {
            initial_isometry
        } else {
            Isometry::from(Vector3::new(0.0, config.suspension_max_length + 1.0, 6.0))
        };

        let rigid_body = RigidBodyBuilder::dynamic()
            .position(initial_isometry)
            .build();
        let cabin_collider = ColliderBuilder::cuboid(
            config.cabin_half_width,
            config.cabin_half_height,
            config.cabin_half_length,
        )
        .collision_groups(
            InteractionGroups::all().with_memberships(Group::from_bits_truncate(
                InteractionGroup::DynamicLevelObjects.into(),
            )),
        )
        // Listen for *all* collision and intersection events with
        // this collider
        .active_events(ActiveEvents::COLLISION_EVENTS)
        // Set the mass (in kg, I think) of the collider
        .density(config.mass)
        .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(cabin_collider, rigid_body_handle, rigid_body_set);
        self.rigid_body_handle = rigid_body_handle;

        self.rapier_vehicle = DynamicRayCastVehicleController::new(self.rigid_body_handle);
    }

    pub fn body_handle(&self) -> RigidBodyHandle {
        self.rigid_body_handle
    }

    pub fn cabin_isometry(&self) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        &self.cabin_isometry
    }

    pub fn camera_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.camera_boom.end_isometry()
    }

    pub fn update(
        &mut self,
        config: &CarConfig,
        settings: &GameSettings,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        let cabin_body_handle = self.body_handle();
        let cloned_body_set = physics.rigid_body_set.clone();

        self.rapier_vehicle.update_vehicle(
            delta_seconds,
            &mut physics.rigid_body_set,
            &physics.collider_set,
            &physics.query_pipeline,
            QueryFilter::exclude_dynamic().exclude_rigid_body(self.body_handle()),
        );

        if let Some(cabin_body) = physics.rigid_body_set.get_mut(cabin_body_handle) {
            Self::update_boom_isometry(
                cabin_body,
                &mut self.camera_boom,
                -input.rotate_right()
                    * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0).to_radians(),
                input.rotate_up()
                    * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                config.max_look_up_angle,
                config.min_look_up_angle,
            );

            Self::prevent_camera_obstructions(
                cabin_body,
                &mut self.camera_boom,
                &mut physics.query_pipeline,
                &cloned_body_set,
                &physics.collider_set,
                QueryFilter::new().exclude_rigid_body(cabin_body_handle),
                &config,
            );
        }
    }

    fn update_boom_isometry(
        cabin_body: &mut RigidBody,
        boom: &mut Boom,
        yaw_magnitude: f32,
        pitch_magnitude: f32,
        min_pitch_angle: f32,
        max_pitch_angle: f32,
    ) {
        boom.translation = cabin_body.position().translation;

        boom.z_rotation =
            boom.z_rotation
                .append_axisangle_linearized(&Vector3::new(0.0, yaw_magnitude, 0.0));

        let (x_roll, x_pitch, x_yaw) = boom.x_rotation.euler_angles();
        boom.x_rotation = UnitQuaternion::from_euler_angles(
            (x_roll + pitch_magnitude)
                .clamp(max_pitch_angle.to_radians(), min_pitch_angle.to_radians()),
            x_pitch,
            x_yaw,
        );
    }

    fn prevent_camera_obstructions(
        cabin_body: &mut RigidBody,
        camera_boom: &mut Boom,
        query_pipeline: &mut QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        query_filter_excluding_cabin: QueryFilter,
        config: &CarConfig,
    ) {
        let body_translation = cabin_body.position().translation;
        let diff_vec = camera_boom.end_isometry().translation.vector - body_translation.vector;
        if let Some((_handle, hit_toi)) = query_pipeline.cast_ray(
            rigid_body_set,
            collider_set,
            &Ray::new(
                Point {
                    coords: body_translation.vector,
                },
                diff_vec.normalize(),
            ),
            config.max_boom_length,
            true,
            query_filter_excluding_cabin,
        ) {
            camera_boom.set_length(hit_toi - 0.03);
        } else {
            camera_boom.set_length(config.max_boom_length);
        }
    }
}
