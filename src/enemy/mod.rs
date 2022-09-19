use std::time::Duration;

use bevy::{prelude::{Component, Vec2, Handle, Image, AssetServer, Assets, IVec2, Transform, Query, EventWriter, Plugin, App, CoreStage, SystemStage}, sprite::TextureAtlas};
use bevy_ecs_ldtk::{prelude::{LdtkEntity, LayerInstance, TilesetDefinition, FieldValue}, EntityInstance, utils::ldtk_pixel_coords_to_translation_pivoted};
use iyes_loopless::{prelude::FixedTimestepStage, condition::ConditionSet};
use crate::{player::{Direction, FrameTimer}, misc::state::GameState};

pub mod knight;

const ENEMY_SPEED: f32 = 0.1;

pub struct PatrolPointReached(Patrol);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(next_patrol)
                    .with_system(enemy_movement)
                    .into()
            );
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub index: usize,
}

impl LdtkEntity for Patrol {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Patrol {
        let mut points = Vec::new();
        points.push(ldtk_pixel_coords_to_translation_pivoted(
            entity_instance.px,
            layer_instance.c_hei * layer_instance.grid_size,
            IVec2::new(entity_instance.width, entity_instance.height),
            entity_instance.pivot,
        ));

        let ldtk_patrol = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == *"Patrol")
            .unwrap();
        if let FieldValue::Points(ldtk_points) = &ldtk_patrol.value {
            for ldtk_point in ldtk_points {
                if let Some(ldtk_point) = ldtk_point {
                    // The +1 is necessary here due to the pivot of the entities in the sample
                    // file.
                    // The patrols set up in the file look flat and grounded,
                    // but technically they're not if you consider the pivot,
                    // which is at the bottom-center for the skulls.
                    let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 1.))
                        * Vec2::splat(layer_instance.grid_size as f32);

                    points.push(ldtk_pixel_coords_to_translation_pivoted(
                        pixel_coords.as_ivec2(),
                        layer_instance.c_hei * layer_instance.grid_size,
                        IVec2::new(entity_instance.width, entity_instance.height),
                        entity_instance.pivot,
                    ));
                }
            }
        }

        Patrol {
            points,
            index: 1,
        }
    }
}

fn next_patrol(
    mut query: Query<(&mut Patrol, &mut Direction, &mut Transform)>,
) {
    for (mut patrol, mut direction, transform) in query.iter_mut() {
        if patrol.points.len() > 1 {
            if patrol.points[patrol.index] == transform.translation.truncate() {
                if patrol.index == patrol.points.len() - 1 {
                    patrol.index = 0;
                } else {
                    patrol.index += 1;
                }
            }

            if patrol.points[patrol.index][0] > transform.translation.x + 1.0 {
                *direction = Direction::Right;
            } else if patrol.points[patrol.index][0] < transform.translation.x - 1.0 {
                *direction = Direction::Left;
            } else if patrol.points[patrol.index][1] > transform.translation.y  + 1.0 {
                *direction = Direction::Up;
            } else if patrol.points[patrol.index][1] < transform.translation.y - 1.0 {
                *direction = Direction::Down;
            }
        }
    }
}

fn enemy_movement(
    mut enemy: Query<(&Direction, &mut Transform)>
) {
    for (direction, mut transform) in enemy.iter_mut() {
        match direction {
            Direction::Right => transform.translation.x += ENEMY_SPEED,
            Direction::Left => transform.translation.x -= ENEMY_SPEED,
            Direction::Up => transform.translation.y += ENEMY_SPEED,
            Direction::Down => transform.translation.y -= ENEMY_SPEED,
            _ => (),
        }
    }
}