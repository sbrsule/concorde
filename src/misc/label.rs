use bevy::prelude::SystemLabel;

#[derive(SystemLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerLabel {
    Input,
    Collision,
    Movement,
}

