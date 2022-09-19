use std::time::Duration;
use iyes_loopless::prelude::*;
use bevy::{prelude::*, sprite::collide_aabb::{collide, Collision}};
use crate::{SPRITE_SCALE, TILE_OFFSET, PLAYER_SPEED, PlayerCamera, misc::{label::PlayerLabel, state::GameState}, level::Collider, TILE_SIZE};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(FrameTimer::default())
            .insert_resource(AnimationTimer::default())
            .add_system(
                movement_input
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::Input)
            )
            .add_system(
                collision_check
                    .run_in_state(GameState::InGame)
                    .after(PlayerLabel::Input)
                    .label(PlayerLabel::Collision)
            )
            .add_system(
                player_movement
                    .run_in_state(GameState::InGame)
                    .run_if(frame_tick)
                    .after(PlayerLabel::Collision)
                    .label(PlayerLabel::Movement)
            )
            .add_system(
                animate_player_movement
                    .run_in_state(GameState::InGame)
                    .run_if(animation_tick)
                    .before(PlayerLabel::Movement)
                    .after(PlayerLabel::Collision)
            )
            .add_system(
                camera_on_player
                    .run_in_state(GameState::InGame)
                    .after(PlayerLabel::Movement)
            );
    }
}

#[derive(Component, Default, Clone)]
pub struct Player;

#[derive(PartialEq, Clone, Default, Component)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    #[default] None
}

#[derive(PartialEq, Clone, Component)]
pub struct Orientation(Direction); 

impl Default for Orientation {
    fn default() -> Self {
        Orientation(Direction::Down)
    }
}

#[derive(Deref, DerefMut, Clone, Component)]
pub struct AnimationTimer(Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::new(Duration::from_millis(190), true))
    }
}

#[derive(Deref, DerefMut, Clone, Component)]
pub struct FrameTimer(Timer);

impl Default for FrameTimer {
    fn default() -> Self {
        FrameTimer(Timer::new(Duration::from_millis(17), true))
    }
}

fn camera_on_player(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let mut camera = camera.single_mut();

    for transform in player.iter() {
        camera.translation = transform.translation;
        camera.translation.z = 999.9;
    }
}

fn animate_player_movement(
    mut player: Query<(&Direction, &Orientation, &mut TextureAtlasSprite)>,
) {
    player.for_each_mut(|(direction, orientation, mut sprite)| {
        match direction {
            Direction::Left => {
                match sprite.index {
                    8..=10 => sprite.index += 1,
                    _ => sprite.index = 8,
                }
            },
            Direction::Right => {
                match sprite.index {
                    12..=14 => {
                        sprite.index += 1;
                    }
                    _ => {
                        sprite.index = 12;
                    }
                }
            },
            Direction::Up => {
                match sprite.index {
                    4..=6 => sprite.index += 1,
                    _ => sprite.index = 4
                }
            },
            Direction::Down => {
                println!("Down animation detected");
                match sprite.index {
                    0..=2 => {sprite.index += 1; println!("Sprite Increased");},
                    _ => {sprite.index = 0; println!("Sprite reset to default");}
                }
            },
            Direction::None => match orientation.0 {
                Direction::Left => sprite.index = 8,
                Direction::Right => sprite.index = 12,
                Direction::Up => sprite.index = 4,
                Direction::Down => sprite.index = 0,
                Direction::None => (),
            },
        }

        if *direction == Direction::None {
            sprite.index = match sprite.index {
                0..=3 => 0,
                4..=7 => 4,
                8..=11 => 8,
                12..=15 => 12,
                _ => sprite.index 
            }
        }
    })
}

fn animation_tick(
    mut timer: ResMut<AnimationTimer>,
    time: Res<Time>,
) -> bool {
    timer.tick(time.delta());
    timer.just_finished()
}

fn movement_input(
    mut query: Query<&mut Direction, With<Player>>,
    keys: Res<Input<KeyCode>>,
) {
    for mut direction in query.iter_mut() {
        if keys.pressed(KeyCode::W) {
            *direction = Direction::Up;
        }
        if keys.pressed(KeyCode::S) {
            *direction = Direction::Down;
        }
        if keys.pressed(KeyCode::D) {
            *direction = Direction::Right;
        }
        if keys.pressed(KeyCode::A) {
            *direction = Direction::Left;
        }
    }
}

fn collision_check(
    mut player: Query<(&Transform, &mut Direction), With<Player>>,
    collider : Query<&Transform, (With<Collider>, Without<Player>)>
) {
    for (transform, mut direction) in player.iter_mut() {    
        let new_translation = match *direction {
            Direction::Left => Vec3::new(transform.translation.x - PLAYER_SPEED, transform.translation.y, transform.translation.z),
            Direction::Right => Vec3::new(transform.translation.x + PLAYER_SPEED, transform.translation.y, transform.translation.z),
            Direction::Up => Vec3::new(transform.translation.x, transform.translation.y + PLAYER_SPEED, transform.translation.z),
            Direction::Down => Vec3::new(transform.translation.x, transform.translation.y - PLAYER_SPEED, transform.translation.z),
            Direction::None => transform.translation,
        };
        collider.for_each(|(collider)| {
                let collision = collide(
                    new_translation,
                    Vec2::splat(TILE_SIZE - 1.0),
                    collider.translation,
                    collider.scale.truncate(),
                );
            
                match collision {
                    Some(collision) => {
                        match collision {
                        Collision::Left => {
                            if *direction == Direction::Right {
                                *direction = Direction::None;
                            }
                        },
                        Collision::Right => {
                            if *direction == Direction::Left {
                                *direction = Direction::None;
                            }
                        },
                        Collision::Top => {
                            if *direction == Direction::Down {
                                *direction = Direction::None;
                            }
                        },
                        Collision::Bottom => {
                            if *direction == Direction::Up {
                                *direction = Direction::None;
                            }
                        },
                        Collision::Inside => (),
                    }
                },
                None => (),
            }
        })
    }
}

fn player_movement(
    mut query: Query<(&mut Transform, &mut Direction, &mut Orientation), With<Player>>,
) {
    query.for_each_mut(|(mut transform, mut direction, mut orientation)| {
        match *direction {
            Direction::Left => transform.translation.x -= PLAYER_SPEED,
            Direction::Right => transform.translation.x += PLAYER_SPEED,
            Direction::Up => transform.translation.y += PLAYER_SPEED,
            Direction::Down => transform.translation.y -= PLAYER_SPEED,
            _ => (),
        };
            
        orientation.0 = direction.clone();
        *direction = Direction::None;
    })
}

fn frame_tick(
    mut frame_timer: ResMut<FrameTimer>,
    time: Res<Time>
) -> bool {
    frame_timer.tick(time.delta()); 
    frame_timer.just_finished()
}
