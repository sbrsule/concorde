use std::{fs, collections::{HashMap, HashSet}};
use bevy_ecs_ldtk::{LdtkWorldBundle, LdtkEntity, prelude::*};
use iyes_loopless::{prelude::{AppLooplessStateExt, ConditionSet}, condition::IntoConditionalExclusiveSystem};
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

use crate::{SPRITE_SCALE, TILE_OFFSET, Player, player::Direction, player::Orientation, misc::state::GameState, player::AnimationTimer};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::InGame, load_level)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_wall_collision)
                    .into(),
             );
    }
}

pub fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ldtk_handle = asset_server.load("levels/test2.ldtk");
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub spritesheet_bundle: SpriteSheetBundle,
    pub player: Player,
    pub direction: Direction,
    pub orientation: Orientation, 
    #[worldly]
    pub worldly: Worldly
}

fn test(
    wall: Query<&Wall>
) {
    for transform in wall.iter() {
        println!("wall exists");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Component, Debug)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    pub wall: Wall
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Component, Debug)]
pub struct Collider;

pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        println!("test2");
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_insert(HashSet::new())
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        println!("test3");
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right
                    // edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut wall_rects: Vec<Rect> = Vec::new();
                let mut previous_rects: HashMap<Plate, Rect> = HashMap::new();

                // an extra empty row so the algorithm "terminates" the rects that touch the top
                // edge
                plate_stack.push(Vec::new());

                for (y, row) in plate_stack.iter().enumerate() {
                    let mut current_rects: HashMap<Plate, Rect> = HashMap::new();
                    for plate in row {
                        if let Some(previous_rect) = previous_rects.remove(plate) {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    top: previous_rect.top + 1,
                                    ..previous_rect
                                },
                            );
                        } else {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    bottom: y as i32,
                                    top: y as i32,
                                    left: plate.left,
                                    right: plate.right,
                                },
                            );
                        }
                    }

                    // Any plates that weren't removed above have terminated
                    wall_rects.append(&mut previous_rects.values().copied().collect());
                    previous_rects = current_rects;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        println!("test");
                        level
                            .spawn()
                            .insert(GlobalTransform::default())
                            .insert(Transform {
                                translation: Vec3::new(
                                    (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                        / 2.0,
                                    (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                        / 2.0,
                                    0.0,
                                ),
                                scale: Vec3::new(
                                    (wall_rect.right as f32 - wall_rect.left as f32 + 1.0)
                                        * grid_size as f32,
                                    (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.0)
                                        * grid_size as f32,
                                    0.0,
                                ),
                                ..Default::default()
                            })
                            .insert(Collider);
                    }
                });
            }
        });
    }
}

fn test_plate(
    collider: Query<&Transform, With<Collider>>,
) {
    for collider in collider.iter() {
        println!("x: {}, y: {}, width: {}, height: {}", collider.translation.x, collider.translation.y, collider.scale.x, collider.scale.y);
    }
}