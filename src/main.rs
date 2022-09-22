use bevy_ecs_ldtk::{LdtkPlugin, LevelSelection, prelude::RegisterLdtkObjects};
use enemy::{knight::KnightBundle, EnemyPlugin};
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use level::{LevelPlugin, PlayerBundle, WallBundle};
use bevy::{prelude::*, render::texture::ImageSettings, time::FixedTimestep, sprite::collide_aabb::{collide, Collision}, window::WindowId, winit::WinitWindows};
use misc::state::GameState;
use player::{AnimationTimer, Player, Direction, PlayerPlugin};
use ui::main_menu::MainMenuPlugin;
use winit::window::Icon;

mod level;
mod player;
mod enemy;
mod misc;
mod ui;

const SPRITE_SCALE: f32 = 3.5;
const TILE_SIZE: f32 = 16.0;
const TILE_OFFSET: f32 = SPRITE_SCALE * TILE_SIZE;
const PLAYER_SPEED: f32 = 2.0/3.0;

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(WindowDescriptor {
            title: "Concord".to_string(),
            width: 1024.0,
            height: 768.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameState::LoadMenu)
        .add_plugin(LdtkPlugin)
        .add_startup_system(set_window_icon)
        .add_enter_system(GameState::LoadGame, setup_camera)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(EnemyPlugin)
        .insert_resource(LevelSelection::Index(0))
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<KnightBundle>("Knight")
        .register_ldtk_int_cell::<WallBundle>(1)
        .run();
}


#[derive(Component)]
struct PlayerCamera;

fn setup_camera(
    mut commands: Commands,
) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scale: 1.0/SPRITE_SCALE,
                ..Default::default()
            },
            ..Default::default()
        }) 
        .insert(PlayerCamera);

    commands.insert_resource(NextState(GameState::InGame));
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/sprites/icon.png")
            .expect("Failed to open icon")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    primary.set_window_icon(Some(icon));
}