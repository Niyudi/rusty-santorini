use bevy::prelude::*;

use bevy::app::AppExit;

use crate::{
    AppState,
    controller::{Controllers, Controller}
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Menu), setup)
            .add_systems(Update, buttons_system.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup);
    }
}

// Constants

const HOVERED_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.65);
const NORMAL_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.25);

// Components

#[derive(Component)]
enum MainMenuButton {
    Play,
    Quit,
}

#[derive(Component)]
struct MainMenuCamera;

#[derive(Component)]
struct MainMenuMarker;

// Systems

fn buttons_system(
    mut commands: Commands,
    mut exit: EventWriter<AppExit>,
    mut interaction_query: Query<
        (&Interaction, &MainMenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                match *button {
                    MainMenuButton::Play => {
                        commands.insert_resource(Controllers {
                            p1: Controller::Human,
                            p2: Controller::Human,
                        });
                        next_state.set(AppState::InGame);
                    }
                    MainMenuButton::Quit => exit.send(AppExit),
                }
                continue;
            }
            Interaction::Hovered => HOVERED_BUTTON_COLOR.into(),
            Interaction::None => NORMAL_BUTTON_COLOR.into(),
        };
    }
}

fn cleanup(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuMarker>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((MainMenuCamera, MainMenuMarker, Camera2dBundle::default()));

    const BASE_COLOR: Color = Color::rgb(0.97, 0.97, 1.00);

    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
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
    let title_style = TextStyle {
        font_size: 80.0,
        color: Color::rgb(0.05, 0.05, 0.65),
        ..default()
    };

    commands
        .spawn((
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
            MainMenuMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BASE_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(
                            TextBundle::from_section(
                                "Santorini",
                                title_style,
                            )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(15.0)),
                                    ..default()
                                }),
                        );

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON_COLOR.into(),
                                ..default()
                            },
                            MainMenuButton::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON_COLOR.into(),
                                ..default()
                            },
                            MainMenuButton::Quit,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Quit",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}