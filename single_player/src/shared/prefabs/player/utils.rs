use crate::shared::controllers::character::utils::CrouchState;
use crate::shared::vectors::*;
use perigee::prelude::*;
use serde::{Deserialize, Serialize};

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

impl AsRef<str> for MovementState {
    fn as_ref(&self) -> &str {
        match self {
            Self::Stationary(crouch_state) => match crouch_state {
                CrouchState::Upright => "IDLE",
                CrouchState::Crouched => "CROUCHED",
            },
            Self::Creeping => "CREEPING",
            Self::Walking(walk_dir) => match walk_dir {
                WalkDirection::Forward => "WALK_FORWARD",
                WalkDirection::RightForward => "WALK_RIGHT_FORWARD",
                WalkDirection::Right => "WALK_RIGHT",
                WalkDirection::RightBack => "WALK_RIGHT_BACK",
                WalkDirection::Back => "WALK_BACKWARD",
                WalkDirection::LeftBack => "WALK_LEFT_BACK",
                WalkDirection::Left => "WALK_LEFT",
                WalkDirection::LeftForward => "WALK_LEFT_FORWARD",
            },
            Self::Running => "RUN_FORWARD",
            Self::Sprinting => "SPRINT_FORWARD",
            Self::InAir => "IN_AIR",
        }
    }
}
