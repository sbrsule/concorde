use bevy::{sprite::SpriteSheetBundle, prelude::{Component, Bundle}};
use bevy_ecs_ldtk::LdtkEntity;

use crate::{player::{Orientation, Direction, AnimationTimer, FrameTimer}, level::Collider};

use super::Patrol;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Knight;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct KnightBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub collider: Collider,
    pub knight: Knight,
    pub direction: Direction,
    pub orientation: Orientation,
    pub animation_timer: AnimationTimer,
    pub frame_timer: FrameTimer,
    #[ldtk_entity]
    pub patrol: Patrol,
}