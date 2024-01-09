use bevy::prelude::*;

use bevy::app::AppExit;

use crate::AppState;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(Update, buttons_system.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), despawn_menu);
    }
}

// Constants

const HOVERED_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.65);
const NORMAL_BUTTON_COLOR: Color = Color::rgb(0.05, 0.05, 0.25);

// Components

#[derive(Component)]
enum MenuButton {
    Play,
    Quit,
}
#[derive(Component)]
struct MainMenuMarker;

// Systems

fn buttons_system(
    mut exit: EventWriter<AppExit>,
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                match *button {
                    MenuButton::Play => next_state.set(AppState::InGame),
                    MenuButton::Quit => exit.send(AppExit),
                }
                continue;
            }
            Interaction::Hovered => HOVERED_BUTTON_COLOR.into(),
            Interaction::None => NORMAL_BUTTON_COLOR.into(),
        };
    }
}
fn despawn_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuMarker>>,
) {
    let entity = menu_query.single();
    commands.entity(entity).despawn_recursive();
}
fn spawn_menu(
    mut commands: Commands,
) {
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
                            MenuButton::Play
                    ,
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
                            MenuButton::Quit,
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