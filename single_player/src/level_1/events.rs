use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Level1Event {
    LevelCompleted,
    LevelRestarted,
    LevelFailed,
}

impl AsRef<str> for Level1Event {
    fn as_ref(&self) -> &str {
        match self {
            Self::LevelCompleted => "LEVEL_COMPLETED",
            Self::LevelRestarted => "LEVEL_RESTARTED",
            Self::LevelFailed => "LEVEL_FAILED",
        }
    }
}
