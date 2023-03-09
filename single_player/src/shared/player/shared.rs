use crate::shared::interactions::InteractionGroup;
use crate::shared::vectors::*;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};

pub const COLLIDER_RAYCAST_OFFSET: f32 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrouchState {
    Upright,
    Crouched,
}

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

pub fn query_filter_excluding_player() -> QueryFilter<'static> {
    QueryFilter {
        groups: Some(InteractionGroups::all().with_filter(
            Group::from_bits_truncate(u32::from(InteractionGroup::Player)).complement(),
        )),
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PerspectiveMode {
    FirstPerson,
    ThirdPersonBasic,
    ThirdPersonCombat,
}

impl Default for PerspectiveMode {
    fn default() -> Self {
        Self::ThirdPersonBasic
    }
}

impl PerspectiveMode {
    pub fn is_third_person(&self) -> bool {
        self == &PerspectiveMode::ThirdPersonBasic || self == &PerspectiveMode::ThirdPersonCombat
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    Discrete,
    Continuous,
}

impl Default for MovementMode {
    fn default() -> Self {
        Self::Discrete
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    Forward,
    RightForward,
    Right,
    RightBack,
    Back,
    LeftBack,
    Left,
    LeftForward,
}

impl Default for WalkDirection {
    fn default() -> Self {
        Self::Forward
    }
}

impl ToString for WalkDirection {
    fn to_string(&self) -> String {
        match self {
            Self::Forward => "FORWARD",
            Self::RightForward => "RIGHT_FORWARD",
            Self::Right => "RIGHT",
            Self::RightBack => "RIGHT_BACK",
            Self::Back => "BACKWARD",
            Self::LeftBack => "LEFT_BACK",
            Self::Left => "LEFT",
            Self::LeftForward => "LEFT_FORWARD",
        }
        .to_string()
    }
}

impl WalkDirection {
    pub fn from_movement_vector(movement_vector: &Vector3<f32>) -> Option<Self> {
        if movement_vector.magnitude() <= 0.0 {
            return None;
        }
        return Some(
            if movement_vector.angle(&FORWARD_VECTOR).to_degrees() < 22.5 {
                WalkDirection::Forward
            } else if movement_vector
                .angle(&(RIGHT_VECTOR + FORWARD_VECTOR))
                .to_degrees()
                <= 22.5
            {
                WalkDirection::RightForward
            } else if movement_vector.angle(&RIGHT_VECTOR).to_degrees() < 22.5 {
                WalkDirection::Right
            } else if movement_vector
                .angle(&(RIGHT_VECTOR + BACK_VECTOR))
                .to_degrees()
                <= 22.5
            {
                WalkDirection::RightBack
            } else if movement_vector.angle(&BACK_VECTOR).to_degrees() < 22.5 {
                WalkDirection::Back
            } else if movement_vector
                .angle(&(LEFT_VECTOR + BACK_VECTOR))
                .to_degrees()
                <= 22.5
            {
                WalkDirection::LeftBack
            } else if movement_vector.angle(&LEFT_VECTOR).to_degrees() < 22.5 {
                WalkDirection::Left
            } else {
                WalkDirection::LeftForward
            },
        );
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, Debug)]
pub enum MovementState {
    Stationary(CrouchState),
    Creeping,
    Walking(WalkDirection),
    Running,
    Sprinting,
    InAir,
}

impl PartialEq for MovementState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Sprinting, Self::Sprinting) => true,
            (Self::Running, Self::Running) => true,
            (Self::Creeping, Self::Creeping) => true,
            (Self::Stationary(self_crouch_state), Self::Stationary(other_crouch_state)) => {
                self_crouch_state == other_crouch_state
            }
            (Self::Walking(self_walk_dir), Self::Walking(other_walk_dir)) => {
                self_walk_dir == other_walk_dir
            }
            (Self::InAir, Self::InAir) => true,
            _ => false,
        }
    }
}

impl Default for MovementState {
    fn default() -> Self {
        Self::Stationary(CrouchState::Upright)
    }
}

impl ToString for MovementState {
    fn to_string(&self) -> String {
        match self {
            Self::Stationary(crouch_state) => match crouch_state {
                CrouchState::Upright => "IDLE".to_string(),
                CrouchState::Crouched => "CROUCHED".to_string(),
            },
            Self::Creeping => "CREEPING".to_string(),
            Self::Walking(walk_dir) => format!("WALK_{}", walk_dir.to_string()),
            Self::Running => "RUN_FORWARD".to_string(),
            Self::Sprinting => "SPRINT_FORWARD".to_string(),
            Self::InAir => "IN_AIR".to_string(),
        }
    }
}
