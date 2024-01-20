use bevy::prelude::*;

use crate::AppState;

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, (
                pause_button,
                pause_menu,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), cleanup);
    }
}


// Constants

const HOVERED_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.65);
const NORMAL_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.25);

// Resources

#[derive(Resource)]
pub struct Paused {
    pub value: bool,
}

// Components

#[derive(Component)]
struct PauseButtonMarker;

#[derive(Component)]
enum PauseMenuButton {
    Resume,
    MainMenu,
}

#[derive(Component)]
struct PauseMenuMarker;

// Systems

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<PauseButtonMarker>, With<PauseMenuButton>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<Paused>();
}

fn pause_button(
    mut commands: Commands,
    mut paused: ResMut<Paused>,
    button_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<PauseButtonMarker>)>,
) {
    const BASE_COLOR: Color = Color::rgba(0.97, 0.97, 1.00, 0.8);

    for (entity, interaction) in button_query.iter() {
            if *interaction == Interaction::Pressed {
                paused.value = true;
                commands.entity(entity).despawn();

                let button_style = Style {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                };
                let button_text_style = TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.95, 0.95, 0.95),
                    ..default()
                };

                commands.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    },
                    PauseMenuMarker,
                )).with_children(|parent| {
                    parent.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BASE_COLOR.into(),
                        ..default()
                    }).with_children(|parent| {
                        parent.spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON_COLOR.into(),
                                ..default()
                            },
                            PauseMenuButton::Resume,
                        )).with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Resume",
                                button_text_style.clone(),
                            ));
                        });

                        parent.spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON_COLOR.into(),
                                ..default()
                            },
                            PauseMenuButton::MainMenu,
                        )).with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Main Menu",
                                button_text_style.clone(),
                            ));
                        });
                    });
                });
            }
    }
}

fn pause_menu(
    mut buttons_query: Query<(&Interaction, &PauseMenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut paused: ResMut<Paused>,
    asset_server: Res<AssetServer>,
    pause_menu_query: Query<Entity, With<PauseMenuMarker>>,
) {
    for (interaction, button, mut color) in buttons_query.iter_mut() {
        match (interaction, button) {
            (Interaction::Pressed, PauseMenuButton::Resume) => {
                paused.value = false;
                commands.entity(pause_menu_query.single()).despawn_recursive();
                spawn_pause_button(&mut commands, Res::clone(&asset_server));
            }
            (Interaction::Pressed, PauseMenuButton::MainMenu) => {
                next_state.set(AppState::Menu);
            }
            (Interaction::Hovered, _) => *color = HOVERED_BUTTON_COLOR.into(),
            (Interaction::None, _) => *color = NORMAL_BUTTON_COLOR.into(),
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Paused { value: false });
    spawn_pause_button(&mut commands, asset_server);
}

// Functions

fn spawn_pause_button(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: asset_server.load("pause_icon.png").into(),
            ..default()
        },
        PauseButtonMarker,
    ));
}