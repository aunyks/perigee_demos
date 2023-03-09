use perigee::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub struct Boom {
    pub translation: Translation<f32>,
    pub z_rotation: UnitQuaternion<f32>,
    pub x_rotation: UnitQuaternion<f32>,
    arm_pivot: Isometry<f32, UnitQuaternion<f32>, 3>,
    length: f32,
}

impl std::fmt::Debug for Boom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.arm_pivot.fmt(f)
    }
}

impl Boom {
    pub fn new(length: f32, arm_pitch_angle: f32, arm_yaw_angle: f32) -> Self {
        let mut new_boom = Self::default();
        new_boom.set_length(length);

        new_boom.arm_pivot.rotation = UnitQuaternion::from_euler_angles(
            arm_pitch_angle.to_radians(),
            arm_yaw_angle.to_radians(),
            0.0,
        );

        new_boom
    }

    pub fn set_length(&mut self, new_length: f32) {
        self.length = new_length
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        Isometry::from_parts(self.translation, self.z_rotation * self.x_rotation)
    }

    pub fn end_isometry(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        self.isometry()
            * self.arm_pivot
            * Isometry::from_parts(
                Translation::from(Vector3::new(0.0, 0.0, self.length)),
                self.arm_pivot.rotation.inverse(),
            )
    }

    pub fn lerp_mut(&mut self, other: &Self, t: f32) {
        self.length = lerp(self.length(), other.length(), t);
        self.arm_pivot = self.arm_pivot.lerp_slerp(&other.arm_pivot, t);
    }
}
