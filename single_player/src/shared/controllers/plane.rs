use crate::config::PlaneControllerConfig;
use crate::shared::boom::Boom;
use crate::shared::input::Input;
use crate::shared::interactions::InteractionGroup;
use crate::shared::settings::GameSettings;
use crate::shared::vectors::*;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlaneController {
    fuselage_body_handle: RigidBodyHandle,
    fuselage_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
    camera_boom: Boom,
    air_density: f32,
}

impl FromConfig for PlaneController {
    type Config<'a> = &'a PlaneControllerConfig;
    fn from_config<'a>(config: Self::Config<'a>) -> Self {
        Self {
            fuselage_body_handle: RigidBodyHandle::default(),
            fuselage_isometry: Isometry::identity(),
            camera_boom: Boom::new(
                config.max_boom_length,
                config.initial_boom_pitch_angle,
                config.initial_boom_yaw_angle,
                true,
            ),
            air_density: 1.0,
        }
    }
}

impl PlaneController {
    pub fn add_to_physics_world(
        &mut self,
        config: &PlaneControllerConfig,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        initial_isometry: Option<Isometry<f32, UnitQuaternion<f32>, 3>>,
    ) {
        let initial_isometry = if let Some(initial_isometry) = initial_isometry {
            initial_isometry
        } else {
            Isometry::from(Vector3::new(0.0, 100.0, 0.0))
        };

        let rigid_body = RigidBodyBuilder::dynamic()
            .position(initial_isometry)
            .build();
        let fuselage_collider = ColliderBuilder::cuboid(
            config.fuselage_half_width,
            config.fuselage_half_height,
            config.fuselage_half_length,
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

        let fuselage_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(fuselage_collider, fuselage_body_handle, rigid_body_set);
        self.fuselage_body_handle = fuselage_body_handle;
    }

    pub fn fuselage_body_handle(&self) -> RigidBodyHandle {
        self.fuselage_body_handle
    }

    pub fn fuselage_isometry(&self) -> &Isometry<f32, UnitQuaternion<f32>, 3> {
        &self.fuselage_isometry
    }

    pub fn camera_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.camera_boom.end_isometry()
    }

    fn update_boom_isometry(
        boom: &mut Boom,
        fuselage_body: &RigidBody,
        yaw_magnitude: f32,
        pitch_magnitude: f32,
        min_pitch_angle: f32,
        max_pitch_angle: f32,
    ) {
        boom.translation = fuselage_body.position().translation;

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
        camera_boom: &mut Boom,
        config: &PlaneControllerConfig,
        fuselage_body: &RigidBody,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        query_filter_excluding_fuselage: QueryFilter,
    ) {
        let body_translation = fuselage_body.position().translation;
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
            query_filter_excluding_fuselage,
        ) {
            camera_boom.set_length(hit_toi - 0.03);
        } else {
            camera_boom.set_length(config.max_boom_length);
        }
    }

    pub fn update(
        &mut self,
        config: &PlaneControllerConfig,
        settings: &GameSettings,
        input: &Input,
        physics: &mut PhysicsWorld,
        delta_seconds: f32,
    ) {
        if let Some(fuselage_body) = physics.rigid_body_set.get_mut(self.fuselage_body_handle) {
            self.fuselage_isometry = *fuselage_body.position();
            let fuselage_velocity = *fuselage_body.linvel();

            Self::simulate_lift(
                config,
                fuselage_body,
                &self.fuselage_isometry,
                &fuselage_velocity,
                self.air_density,
                delta_seconds,
            );
            Self::simulate_drag(
                config,
                fuselage_body,
                &self.fuselage_isometry,
                &fuselage_velocity,
                self.air_density,
                delta_seconds,
            );

            Self::simulate_thrust();
            Self::simulate_roll();
            Self::simulate_pitch();
            Self::simulate_yaw();
        }

        if let Some(fuselage_body) = physics.rigid_body_set.get(self.fuselage_body_handle) {
            Self::update_boom_isometry(
                &mut self.camera_boom,
                fuselage_body,
                -input.rotate_right()
                    * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0).to_radians(),
                input.rotate_up()
                    * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                config.max_look_up_angle,
                config.min_look_up_angle,
            );

            Self::prevent_camera_obstructions(
                &mut self.camera_boom,
                &config,
                fuselage_body,
                &physics.query_pipeline,
                &physics.rigid_body_set,
                &physics.collider_set,
                QueryFilter::new().exclude_rigid_body(self.fuselage_body_handle),
            );
        }
    }

    fn simulate_lift(
        config: &PlaneControllerConfig,
        fuselage_body: &mut RigidBody,
        fuselage_isometry: &Isometry<f32, UnitQuaternion<f32>, 3>,
        fuselage_linvel: &Vector3<f32>,
        air_density: f32,
        delta_seconds: f32,
    ) {
        let lift_magnitude =
            config.lift_coefficient * air_density * (fuselage_linvel.magnitude().powi(2) / 2.0);
        let lift_force = UP_VECTOR * lift_magnitude;

        fuselage_body.apply_impulse(
            fuselage_isometry.transform_vector(&lift_force) * delta_seconds,
            true,
        );
    }

    fn simulate_drag(
        config: &PlaneControllerConfig,
        fuselage_body: &mut RigidBody,
        fuselage_isometry: &Isometry<f32, UnitQuaternion<f32>, 3>,
        fuselage_linvel: &Vector3<f32>,
        air_density: f32,
        delta_seconds: f32,
    ) {
        let drag_magnitude =
            config.drag_coefficient * air_density * (fuselage_linvel.magnitude().powi(2) / 2.0);
        let drag_force = BACK_VECTOR * drag_magnitude;

        fuselage_body.apply_impulse(
            fuselage_isometry.transform_vector(&drag_force) * delta_seconds,
            true,
        );
    }

    fn simulate_thrust() {}

    fn simulate_roll() {}

    fn simulate_pitch() {}

    fn simulate_yaw() {}
}
