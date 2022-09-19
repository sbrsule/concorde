use bevy::prelude::*;
use bevy_asset_loader::prelude::{LoadingStateAppExt, LoadingState};
use iyes_loopless::{prelude::{AppLooplessStateExt, IntoConditionalSystem}, state::NextState};
use crate::misc::state::GameState;
use super::menu_assets::MenuAssets;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(GameState::LoadMenu)
                    .continue_to_state(GameState::MainMenu)
                    .with_collection::<MenuAssets>(),
            )
            .add_enter_system(GameState::MainMenu, spawn_menu)
            .add_system(
                start_button
                    .run_in_state(GameState::MainMenu)
            )
            .add_exit_system(GameState::MainMenu, despawn_menu);
    }
}

#[derive(Component)]
struct MainMenuComponent;

#[derive(Component)]
struct StartButton;

fn spawn_menu(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainMenuComponent);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(MainMenuComponent)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    image: menu_assets.button.clone().into(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(StartButton);
        });
}

fn start_button(
    mut commands: Commands,
    interaction: Query<&Interaction, With<StartButton>>,
) {
    interaction.for_each(|interaction| {
        if *interaction == Interaction::Clicked {
            commands.insert_resource(NextState(GameState::LoadGame));
        }
    }) 
}

fn despawn_menu(
    mut commands: Commands,
    menu: Query<Entity, With<MainMenuComponent>>
) {
    menu.for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    }) 
}