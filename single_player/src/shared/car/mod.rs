use crate::config::CarConfig;
use crate::shared::input::Input;
use crate::shared::interactions::InteractionGroup;
use perigee::prelude::*;
use perigee::rapier3d::na::Translation3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Shock {
    pub translation: Translation3<f32>,
    pub last_toi: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Car {
    config: CarConfig,
    rigid_body_handle: RigidBodyHandle,
    suspension_ray: Ray,
    fl: Shock,
    fr: Shock,
    bl: Shock,
    br: Shock,
    cabin_isometry: Isometry<f32, UnitQuaternion<f32>, 3>,
}

impl Default for Car {
    fn default() -> Self {
        let car_config = CarConfig::default();
        Self {
            config: car_config,
            rigid_body_handle: RigidBodyHandle::default(),
            suspension_ray: Ray::new(Point::new(0.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
            cabin_isometry: Isometry::default(),
            fl: Shock {
                translation: Translation3::from(Vector3::new(
                    -car_config.cabin_half_width(),
                    -car_config.cabin_half_height(),
                    -car_config.cabin_half_length(),
                )),
                last_toi: car_config.suspension_max_length(),
            },
            fr: Shock {
                translation: Translation3::from(Vector3::new(
                    car_config.cabin_half_width(),
                    -car_config.cabin_half_height(),
                    -car_config.cabin_half_length(),
                )),
                last_toi: car_config.suspension_max_length(),
            },
            bl: Shock {
                translation: Translation3::from(Vector3::new(
                    -car_config.cabin_half_width(),
                    -car_config.cabin_half_height(),
                    car_config.cabin_half_length(),
                )),
                last_toi: car_config.suspension_max_length(),
            },
            br: Shock {
                translation: Translation3::from(Vector3::new(
                    car_config.cabin_half_width(),
                    -car_config.cabin_half_height(),
                    car_config.cabin_half_length(),
                )),
                last_toi: car_config.suspension_max_length(),
            },
        }
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
                -6.0,
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

    pub fn update(&mut self, delta_seconds: f32, _input: &Input, physics: &mut PhysicsWorld) {
        let query_filter = QueryFilter::new();

        let cabin_body_handle = self.body_handle();
        let cloned_rigid_body_set = physics.rigid_body_set.clone();
        if let Some(cabin_body) = physics.rigid_body_set.get_mut(cabin_body_handle) {
            self.cabin_isometry = *cabin_body.position();

            for shock in [&mut self.fl, &mut self.fr, &mut self.bl, &mut self.br].iter_mut() {
                let wheel_global_iso = cabin_body.position() * shock.translation;
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
                    Self::simulate_suspension(
                        &self.config,
                        shock,
                        &shock_ray,
                        &intersection_details,
                        cabin_body,
                        delta_seconds,
                    );
                } else {
                    shock.last_toi = self.config.suspension_max_length();
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
        delta_seconds: f32,
    ) {
        let global_intersection_point = shock_ray.point_at(intersection_details.toi);
        let up = -shock_ray.dir;

        let spring_compression = config.suspension_max_length() - intersection_details.toi;

        let spring_force = up * config.shock_spring_constant() * spring_compression;

        let up_velocity = (intersection_details.toi - shock.last_toi) / delta_seconds;

        let dampening_force = up * up_velocity * config.shock_spring_dampening_factor();

        let shock_force = spring_force - dampening_force;

        cabin_body.apply_impulse_at_point(
            shock_force * delta_seconds,
            global_intersection_point,
            false,
        );
        shock.last_toi = intersection_details.toi;
    }
}