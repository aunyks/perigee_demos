use perigee::prelude::*;

use crate::config::{sedan::CameraMode, SedanConfig};
use crate::shared::boom::Boom;
use crate::shared::controllers::RaycastVehicleController;
use crate::shared::descriptor::Descriptor;
use crate::shared::input::Input;
use crate::shared::settings::GameSettings;
use perigee::rapier3d::na::Translation3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FollowCamExtras {
    pivot_translation: Vector3<f32>,
    pivot_rotation: UnitQuaternion<f32>,
    lerp_factor: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Sedan<'a> {
    pub descriptor: Descriptor<'a>,
    pub controller: RaycastVehicleController,
    camera_boom: Boom,
    camera_mode: CameraMode,
    follow_cam_rig: FollowCamExtras,
    camera_iso: Isometry3<f32>,
}

impl<'a> FromConfig for Sedan<'a> {
    type Config<'b> = &'b SedanConfig;

    fn from_config<'b>(config: Self::Config<'b>) -> Self {
        let follow_cam_quat = UnitQuaternion::identity();
        let follow_cam_quat = follow_cam_quat.append_axisangle_linearized(
            &(Vector3::x() * config.track_mode_look_pitch_angle.to_radians()),
        );
        let follow_cam_quat = follow_cam_quat.append_axisangle_linearized(
            &(Vector3::y() * config.track_mode_look_yaw_angle.to_radians()),
        );
        Self {
            controller: RaycastVehicleController::from_config(&config.raycast_vehicle_controller),
            // [P]re-[C]onfigured [S]edan
            descriptor: Descriptor::from_name("PCS"),
            camera_boom: Boom::new(
                config.max_boom_length,
                config.initial_boom_pitch_angle,
                config.initial_boom_yaw_angle,
                true,
            ),
            camera_mode: config.initial_camera_mode,
            follow_cam_rig: FollowCamExtras {
                pivot_translation: Vector3::new(0.0, 0.0, config.max_boom_length).into(),
                pivot_rotation: follow_cam_quat,
                lerp_factor: config.track_mode_cam_lerp_factor,
            },
            camera_iso: Isometry::identity(),
        }
    }
}

impl<'a> Sedan<'a> {
    pub fn initialize(
        &mut self,
        config: &SedanConfig,
        physics: &mut PhysicsWorld,
        initial_isometry: Option<Isometry3<f32>>,
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

    pub fn camera_isometry(&self) -> Isometry3<f32> {
        self.camera_iso.clone()
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
            input,
            physics,
            delta_seconds,
        );

        if let Some(cabin_body) = physics
            .rigid_body_set
            .get(self.controller.cabin_body_handle())
        {
            if self.camera_mode == CameraMode::Free {
                Self::update_boom_isometry(
                    &mut self.camera_boom,
                    cabin_body,
                    -input.rotate_right()
                        * (2.5 * f32::from(settings.left_right_look_sensitivity()) / 5.0)
                            .to_radians(),
                    input.rotate_up()
                        * (5.0 * f32::from(settings.up_down_look_sensitivity()) / 5.0).to_radians(),
                    config.max_look_up_angle,
                    config.min_look_up_angle,
                );

                Self::prevent_boom_obstructions(
                    &mut self.camera_boom,
                    &config,
                    cabin_body,
                    &physics.query_pipeline,
                    &physics.rigid_body_set,
                    &physics.collider_set,
                    QueryFilter::new().exclude_rigid_body(self.controller.cabin_body_handle()),
                );

                self.camera_iso = self.camera_boom.end_isometry();
            } else {
                Self::update_follow_camera(
                    &physics.rigid_body_set,
                    &physics.collider_set,
                    &physics.query_pipeline,
                    QueryFilter::new().exclude_rigid_body(self.controller.cabin_body_handle()),
                    config,
                    cabin_body,
                    &mut self.follow_cam_rig,
                    &mut self.camera_iso,
                    config.track_mode_cam_lerp_factor,
                    delta_seconds,
                );
            }
        }
    }

    fn update_boom_isometry(
        boom: &mut Boom,
        cabin_body: &RigidBody,
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

    fn update_follow_camera(
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        query_pipeline: &QueryPipeline,
        query_filter_excluding_cabin: QueryFilter,
        config: &SedanConfig,
        cabin_body: &RigidBody,
        follow_rig: &mut FollowCamExtras,
        camera_iso: &mut Isometry3<f32>,
        lerp_factor: f32,
        delta_seconds: f32,
    ) {
        let mut target_iso = cabin_body.position()
            * follow_rig.pivot_rotation
            * Translation3::from(follow_rig.pivot_translation);

        let diff_vec = target_iso.translation.vector - cabin_body.position().translation.vector;

        target_iso.rotation = UnitQuaternion::face_towards(&diff_vec, &Vector3::y());

        target_iso.translation = camera_iso
            .translation
            .vector
            .lerp(
                &target_iso.translation.vector,
                framerate_independent_interp_t(lerp_factor, delta_seconds),
            )
            .into();

        let ray = Ray::new(
            Point {
                coords: cabin_body.position().translation.vector,
            },
            diff_vec.normalize(),
        );

        if let Some((_handle, hit_toi)) = query_pipeline.cast_ray(
            rigid_body_set,
            collider_set,
            &ray,
            config.max_boom_length,
            true,
            query_filter_excluding_cabin,
        ) {
            target_iso.translation = ray.point_at(hit_toi - 0.03).into();
        }

        camera_iso.clone_from(&target_iso);
    }

    fn prevent_boom_obstructions(
        camera_boom: &mut Boom,
        config: &SedanConfig,
        cabin_body: &RigidBody,
        query_pipeline: &QueryPipeline,
        rigid_body_set: &RigidBodySet,
        collider_set: &ColliderSet,
        query_filter_excluding_cabin: QueryFilter,
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
