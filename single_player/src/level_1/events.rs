use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Level1Event {
    LevelCompleted,
    PlayerReset,
    CheckpointReached,
}

impl AsRef<str> for Level1Event {
    fn as_ref(&self) -> &str {
        match self {
            Self::LevelCompleted => "LEVEL_COMPLETED",
            Self::PlayerReset => "PLAYER_RESET",
            Self::CheckpointReached => "CHECKPOINT_REACHED",
        }
    }
}
