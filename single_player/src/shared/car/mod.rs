use crate::config::car::{CarConfig, WheelWellConfig};
use crate::shared::boom::Boom;
use crate::shared::input::Input;
use crate::shared::interactions::InteractionGroup;
use crate::shared::settings::GameSettings;
use crate::shared::traits::FromConfig;
use crate::shared::vectors::*;
use perigee::prelude::*;
use perigee::rapier3d::na::Translation3;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
struct Shock {
    pub translation: Translation3<f32>,
    pub last_toi: f32,
}

#[derive(Serialize, Deserialize)]
struct WheelWell {
    /// A structure to help manage the vehicle's
    /// suspension system.
    pub shock: Shock,
    /// A wheel that turns and controls the direction
    /// of the vehicle. If there is none, then the wheel is
    /// always rolling straight forward and doesn't turn.
    pub steer_state: Option<UnitQuaternion<f32>>,
    /// Whether this wheel receives power from the engine.
    /// Use this to help determine the drivetrain layout.
    pub receives_power: bool,
}

impl WheelWell {
    pub fn from_config(config: &WheelWellConfig, default_suspension_max_length: f32) -> Self {
        Self {
            receives_power: config.receives_power(),
            steer_state: if config.steers_on_input() {
                Some(UnitQuaternion::identity())
            } else {
                None
            },
            shock: Shock {
                translation: Translation::from(config.center_cabin_relative_position()),
                last_toi: config
                    .suspension_max_length()
                    .unwrap_or(default_suspension_max_length),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Car {
    config: Rc<CarConfig>,
    camera_boom: Boom,
    rigid_body_handle: RigidBodyHandle,
    suspension_ray: Ray,
    suspension_system: Vec<WheelWell>,
    cabin_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
}

impl FromConfig for Car {
    type Config<'a> = &'a Rc<CarConfig>;
    fn from_config<'a>(config: Self::Config<'a>) -> Self {
        let mut wheel_wells: Vec<WheelWell> = Vec::with_capacity(config.wheel_wells().len());
        for well_config in config.wheel_wells() {
            wheel_wells.push(WheelWell::from_config(
                well_config,
                config.suspension_max_length(),
            ));
        }
        Self {
            config: Rc::clone(config),
            rigid_body_handle: RigidBodyHandle::default(),
            suspension_ray: Ray::new(Point::new(0.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
            cabin_isometry: Isometry::default(),
            camera_boom: Boom::new(
                config.max_boom_length(),
                config.initial_boom_pitch_angle(),
                config.initial_boom_yaw_angle(),
                true,
            ),
            suspension_system: wheel_wells,
        }
    }

    fn set_config<'a>(&mut self, config: Self::Config<'a>) {
        self.config = Rc::clone(config);
    }
}

impl Car {
    pub fn add_to_physics_world(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        initial_isometry: Option<Isometry<f32, UnitQuaternion<f32>, 3>>,
    ) {
        let initial_isometry = if let Some(initial_isometry) = initial_isometry {
            initial_isometry
        } else {
            Isometry::from(Vector3::new(
                0.0,
                self.config.suspension_max_length() + 1.0,
                6.0,
            ))
        };

        let rigid_body = RigidBodyBuilder::dynamic()
            .position(initial_isometry)
            .build();
        let cabin_collider = ColliderBuilder::cuboid(
            self.config.cabin_half_width(),
            self.config.cabin_half_height(),
            self.config.cabin_half_length(),
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
        .density(self.config.mass())
        .build();

        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(cabin_collider, rigid_body_handle, rigid_body_set);
        self.rigid_body_handle = rigid_body_handle;
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
        delta_seconds: f32,
        input: &Input,
        settings: &GameSettings,
        physics: &mut PhysicsWorld,
    ) {
        let query_filter = QueryFilter::new();

        let cabin_body_handle = self.body_handle();
        let cloned_rigid_body_set = physics.rigid_body_set.clone();
        if let Some(cabin_body) = physics.rigid_body_set.get_mut(cabin_body_handle) {
            self.cabin_isometry = *cabin_body.position();
            let num_wheels = self.suspension_system.len();

            Self::update_boom_isometry(
                cabin_body,
                &mut self.camera_boom,
                -input.rotate_right()
                    * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0).to_radians(),
                input.rotate_up()
                    * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                self.config.max_look_up_angle(),
                self.config.min_look_up_angle(),
            );

            Self::prevent_camera_obstructions(
                cabin_body,
                &mut self.camera_boom,
                &mut physics.query_pipeline,
                &cloned_rigid_body_set.clone(),
                &physics.collider_set,
                query_filter.exclude_rigid_body(cabin_body_handle),
                &self.config,
            );

            for wheel_well in self.suspension_system.iter_mut() {
                let wheel_global_iso = cabin_body.position() * wheel_well.shock.translation;
                let shock_ray = self.suspension_ray.transform_by(&wheel_global_iso);

                if let Some((_, intersection_details)) =
                    physics.query_pipeline.cast_ray_and_get_normal(
                        &cloned_rigid_body_set,
                        &physics.collider_set,
                        &shock_ray,
                        self.config.suspension_max_length(),
                        true,
                        query_filter.exclude_rigid_body(cabin_body_handle),
                    )
                {
                    let global_steer_state = wheel_global_iso.rotation
                        * wheel_well.steer_state.unwrap_or(UnitQuaternion::identity());
                    let wheel_body_attachment_point = wheel_global_iso * Point::origin();

                    Self::simulate_suspension(
                        &self.config,
                        &mut wheel_well.shock,
                        &shock_ray,
                        &intersection_details,
                        cabin_body,
                        wheel_body_attachment_point,
                        delta_seconds,
                    );

                    Self::simulate_brake(
                        cabin_body,
                        &input,
                        &global_steer_state,
                        wheel_body_attachment_point,
                        &self.config,
                        delta_seconds,
                    );

                    if let Some(wheel_orientation) = wheel_well.steer_state.as_mut() {
                        Self::simulate_steering(&input, wheel_orientation, &self.config);
                    }

                    Self::simulate_wheel_grip(
                        cabin_body,
                        &global_steer_state,
                        wheel_body_attachment_point,
                        &self.config,
                        cabin_body.mass() / (num_wheels as f32),
                        delta_seconds,
                    );

                    if wheel_well.receives_power {
                        Self::simulate_throttle(
                            cabin_body,
                            &input,
                            &global_steer_state,
                            wheel_body_attachment_point,
                            &self.config,
                            delta_seconds,
                        );
                    }
                } else {
                    wheel_well.shock.last_toi = self.config.suspension_max_length();
                }
            }
        }
    }

    fn simulate_suspension(
        config: &CarConfig,
        shock: &mut Shock,
        shock_ray: &Ray,
        intersection_details: &RayIntersection,
        cabin_body: &mut RigidBody,
        force_app_location: Point<f32>,
        delta_seconds: f32,
    ) {
        let up = -shock_ray.dir;

        let spring_compression = config.suspension_max_length() - intersection_details.toi;

        let spring_force = up * config.shock_spring_constant() * spring_compression;

        let up_velocity = (intersection_details.toi - shock.last_toi) / delta_seconds;

        let dampening_force = up * up_velocity * config.shock_spring_dampening_factor();

        let shock_force = spring_force - dampening_force;

        cabin_body.apply_impulse_at_point(shock_force * delta_seconds, force_app_location, true);
        shock.last_toi = intersection_details.toi;
    }

    /// Apply force in the wheel forward direction
    /// based on input
    fn simulate_throttle(
        cabin_body: &mut RigidBody,
        input: &Input,
        global_steer_state: &UnitQuaternion<f32>,
        force_app_location: Point<f32>,
        config: &CarConfig,
        delta_seconds: f32,
    ) {
        let force_direction = global_steer_state * FORWARD_VECTOR;
        cabin_body.apply_impulse_at_point(
            force_direction
                * config.throttle_force()
                * f32::max(0.0, input.throttle())
                * delta_seconds,
            force_app_location,
            true,
        );
    }

    /// Apply force in the wheel back direction
    /// based on input
    fn simulate_brake(
        cabin_body: &mut RigidBody,
        input: &Input,
        global_steer_state: &UnitQuaternion<f32>,
        force_app_location: Point<f32>,
        config: &CarConfig,
        delta_seconds: f32,
    ) {
        let force_direction = global_steer_state * BACK_VECTOR;
        cabin_body.apply_impulse_at_point(
            force_direction * config.brake_force() * f32::max(0.0, input.brake()) * delta_seconds,
            force_app_location,
            true,
        );
    }

    /// Make sure wheel restricts non-forward direction
    /// movement based on its grip configuration
    fn simulate_wheel_grip(
        cabin_body: &mut RigidBody,
        global_steer_state: &UnitQuaternion<f32>,
        force_app_location: Point<f32>,
        config: &CarConfig,
        wheel_mass: f32,
        delta_seconds: f32,
    ) {
        let mut wheel_turn_velocity = cabin_body.velocity_at_point(&force_app_location);
        wheel_turn_velocity.y = 0.0;
        let wheel_local_turn_velocity =
            global_steer_state.inverse_transform_vector(&wheel_turn_velocity);

        let mut wheel_local_turn_velocity_no_drift = wheel_local_turn_velocity.clone();
        wheel_local_turn_velocity_no_drift.x = 0.0;

        let wheel_local_frame_goal_velocity = move_towards(
            &wheel_local_turn_velocity,
            &wheel_local_turn_velocity_no_drift,
            config.wheel_grip() * delta_seconds,
        );

        let wheel_local_frame_acceleration =
            wheel_local_frame_goal_velocity - wheel_local_turn_velocity;

        cabin_body.apply_impulse_at_point(
            global_steer_state.transform_vector(&(wheel_local_frame_acceleration * wheel_mass)),
            force_app_location,
            true,
        );
    }

    /// Rotate wheel based on input
    fn simulate_steering(
        input: &Input,
        wheel_orientation: &mut UnitQuaternion<f32>,
        config: &CarConfig,
    ) {
        let steer_factor = input.steer();

        let steer_lerp_t = remap(steer_factor, -1.0, 1.0, 0.0, 1.0);

        wheel_orientation.clone_from(
            &UnitQuaternion::from_axis_angle(
                &Unit::new_normalize(UP_VECTOR),
                config.wheel_left_turn_angle().to_radians(),
            )
            .slerp(
                &UnitQuaternion::from_axis_angle(
                    &Unit::new_normalize(UP_VECTOR),
                    config.wheel_right_turn_angle().to_radians(),
                ),
                steer_lerp_t,
            ),
        );
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
            config.max_boom_length(),
            true,
            query_filter_excluding_cabin,
        ) {
            camera_boom.set_length(hit_toi - 0.03);
        } else {
            camera_boom.set_length(config.max_boom_length());
        }
    }
}
