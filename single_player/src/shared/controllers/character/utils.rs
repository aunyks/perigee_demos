use perigee::prelude::*;
use serde::{Deserialize, Serialize};

pub const COLLIDER_RAYCAST_OFFSET: f32 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlidingState {
    None,
    Normal,
    Downhill,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WallRunning {
    OnRight(Vector3<f32>),
    OnLeft(Vector3<f32>),
    None,
}

impl PartialEq for WallRunning {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (WallRunning::OnLeft(_), WallRunning::OnLeft(_))
                | (WallRunning::OnRight(_), WallRunning::OnRight(_))
                | (WallRunning::None, WallRunning::None)
        )
    }
}

impl Eq for WallRunning {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrouchState {
    Upright,
    Crouched,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CharacterPerspectiveMode {
    #[serde(rename = "first-person")]
    FirstPerson,
    #[serde(rename = "third-person-basic")]
    ThirdPersonBasic,
    #[serde(rename = "third-person-combat")]
    ThirdPersonCombat,
}

impl CharacterPerspectiveMode {
    pub fn is_third_person(&self) -> bool {
        self == &CharacterPerspectiveMode::ThirdPersonBasic
            || self == &CharacterPerspectiveMode::ThirdPersonCombat
    }
}
