/// Types of objects that can interact with each other. Meant to be used
/// in conjunction with [Rapier Interaction Groups](rapier3d::prelude::InteractionGroups).
#[derive(Debug, Clone, Copy)]
pub enum InteractionGroup {
    All,
    StaticLevelObjects,
    DynamicLevelObjects,
    CharacterController,
}

impl From<InteractionGroup> for u32 {
    fn from(group: InteractionGroup) -> u32 {
        // Each group gets a power of 2 u32 value so that only
        // one of its 32 bits is a 1. This lets us use bitwise operations
        // to control which groups can interact with others.
        //
        // For example, if an object should collide with more than one interaction group,
        // you can define that as StaticLevelObjects | DynamicLevelObjects | CharacterController. Conversely,
        // If it should interact with all groups but a few, you can define that as DynamicLevelObjects ^ CharacterController
        match group {
            InteractionGroup::All => u32::MAX,
            InteractionGroup::StaticLevelObjects => 2u32.pow(0),
            InteractionGroup::DynamicLevelObjects => 2u32.pow(1),
            InteractionGroup::CharacterController => 2u32.pow(2),
        }
    }
}
