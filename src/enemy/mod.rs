use std::time::Duration;

use bevy::{prelude::{Component, Vec2, Handle, Image, AssetServer, Assets, IVec2, Transform, Query, EventWriter, Plugin, App, CoreStage, SystemStage, Res}, sprite::TextureAtlas, time::Time};
use bevy_ecs_ldtk::{prelude::{LdtkEntity, LayerInstance, TilesetDefinition, FieldValue}, EntityInstance, utils::ldtk_pixel_coords_to_translation_pivoted};
use iyes_loopless::{prelude::FixedTimestepStage, condition::{ConditionSet, IntoConditionalSystem}};
use crate::{player::{Direction, FrameTimer}, misc::state::GameState};

pub mod knight;

const ENEMY_SPEED: f32 = 1.0/3.0;

pub struct PatrolPointReached(Patrol);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(
                next_patrol
                    .run_in_state(GameState::InGame)
                    .label("patrol")
            )
            .add_system(
                enemy_movement
                    .run_in_state(GameState::InGame)
                    .after("patrol")
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
    pub forward: bool,
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
                    let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 0.5))
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
            forward: true,
        }
    }
}

fn next_patrol(
    mut query: Query<(&mut Patrol, &mut Direction, &mut Transform)>,
) {
    for (mut patrol, mut direction, transform) in query.iter_mut() {
        if patrol.points.len() > 1 {
            // try to do something similar to future collisions, but check patrol within margin of error
            let mut new_vec: Vec2;
            match *direction {
                Direction::Right => new_vec = transform.translation.truncate() + Vec2::new(ENEMY_SPEED, 0.0),
                Direction::Left => new_vec = transform.translation.truncate() + Vec2::new(-ENEMY_SPEED, 0.0),
                Direction::Up => new_vec = transform.translation.truncate() + Vec2::new(0.0, ENEMY_SPEED),
                Direction::Right => new_vec = transform.translation.truncate() + Vec2::new(0.0, -ENEMY_SPEED),
                _ => ()
            }
            if patrol.points[patrol.index] == transform.translation.truncate() {
                println!("reset patrol");
                if patrol.index == patrol.points.len() - 1 {
                    patrol.index -= 1;
                    patrol.forward = false;
                } else if patrol.index == 0 {
                    patrol.index += 1;
                    patrol.forward = true;
                } else if patrol.forward {
                    patrol.index += 1;
                } else {
                    patrol.index -= 1;
                }
            } else {
                println!("patrol point: {:?}, sprite point: {:?}", patrol.points[patrol.index], transform.translation.truncate())
            }

            if patrol.points[patrol.index][0] > transform.translation.x {
                *direction = Direction::Right;
            } else if patrol.points[patrol.index][0] < transform.translation.x {
                *direction = Direction::Left;
            } else if patrol.points[patrol.index][1] > transform.translation.y {
                *direction = Direction::Up;
            } else if patrol.points[patrol.index][1] < transform.translation.y {
                *direction = Direction::Down;
            }
        }
    }
}

fn enemy_movement(
    mut enemy: Query<(&Direction, &mut Transform, &mut FrameTimer, &Patrol)>,
    time: Res<Time>,
) {
    for (direction, mut transform, mut frame_timer, patrol) in enemy.iter_mut() {
        if frame_timer.tick(time.delta()).just_finished() {
            match direction {
                Direction::Right => {
                    if patrol.points[patrol.index][0] < transform.translation.x + ENEMY_SPEED {
                        transform.translation.x = patrol.points[patrol.index][0];
                    } else {
                        transform.translation.x += ENEMY_SPEED
                    }
                },
                Direction::Left => {
                    if patrol.points[patrol.index][0] > transform.translation.x - ENEMY_SPEED {
                        transform.translation.x = patrol.points[patrol.index][0];
                    } else {
                        transform.translation.x -= ENEMY_SPEED
                    }
                },
                Direction::Up => {
                    if patrol.points[patrol.index][1] < transform.translation.y + ENEMY_SPEED {
                        transform.translation.y = patrol.points[patrol.index][1];
                    } else {
                        transform.translation.y += ENEMY_SPEED
                    }
                },
                Direction::Down => {
                    if patrol.points[patrol.index][1] > transform.translation.y - ENEMY_SPEED {
                        transform.translation.y = patrol.points[patrol.index][1];
                    } else {
                        transform.translation.y -= ENEMY_SPEED
                    }
                },
                _ => (),
            }
        }
    }
}