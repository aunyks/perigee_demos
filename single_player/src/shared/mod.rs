pub mod boom;
pub mod controllers;
pub mod events;
pub mod input;
pub mod interactions;
pub mod prefabs;
pub mod settings;

pub mod vectors {
    use perigee::rapier3d::na::Vector3;

    pub static INVALID_VECTOR: Vector3<f32> = Vector3::new(f32::NAN, f32::NAN, f32::NAN);
    pub static UP_VECTOR: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    pub static DOWN_VECTOR: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);
    pub static RIGHT_VECTOR: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
    pub static LEFT_VECTOR: Vector3<f32> = Vector3::new(-1.0, 0.0, 0.0);
    pub static FORWARD_VECTOR: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
    pub static BACK_VECTOR: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
}
