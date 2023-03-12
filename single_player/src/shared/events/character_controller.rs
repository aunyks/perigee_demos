use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterControllerEvent {
    /// CharacterController just jumped
    Jump,
    /// CharacterController just landed on a surface
    Landed,
    Moving,
    Stopped,
    Crouched,
    StoodUpright,
    /// CharacterController just took a footstep
    Stepped,
    StartedWallRunning,
    StoppedWallRunning,
    /// CharacterController started sliding on the ground
    StartedSliding,
    StoppedSliding,
}
