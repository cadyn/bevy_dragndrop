use bevy::prelude::*;
use bevy_dragndrop::DragPlugin;
use bevy_dragndrop::*;
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DragPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (on_dropped, on_dragged, on_hovered))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon: Handle<Image> = asset_server.load("textures/icon.png");
    // Camera
    commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            background_color: Color::srgb(0.40, 0.40, 0.40).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        width: Val::Auto,
                        height: Val::Percent(90.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_self: AlignSelf::Center,
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.10, 0.10, 0.10).into(),
                    ..default()
                })
                .with_children(|parent| {
                    for x in 1..6 {
                        for y in 1..6 {
                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            display: Display::Flex,
                                            width: Val::Auto,
                                            height: Val::Auto,
                                            justify_content: JustifyContent::SpaceAround,
                                            align_self: AlignSelf::Center,
                                            aspect_ratio: Some(1.0),
                                            border: UiRect::all(Val::Percent(0.75)),
                                            grid_row: GridPlacement::start(x),
                                            grid_column: GridPlacement::start(y),
                                            align_content: AlignContent::Center,
                                            ..default()
                                        },
                                        background_color: Color::srgb(0.30, 0.30, 0.30).into(),
                                        border_color: Color::srgb(0.75, 0.75, 0.75).into(),
                                        ..default()
                                    },
                                    Receiver,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        NodeBundle {
                                            style: Style {
                                                width: Val::Percent(75.0),
                                                height: Val::Percent(75.0),
                                                align_self: AlignSelf::Center,
                                                ..default()
                                            },
                                            background_color: Color::hsl(
                                                rng.gen::<f32>() * 360.0,
                                                1.0,
                                                0.5,
                                            )
                                            .into(),
                                            ..default()
                                        },
                                        UiImage::new(icon.clone()),
                                        Draggable::default(),
                                    ));
                                });
                        }
                    }
                });
        });
}

fn on_dropped(
    mut commands: Commands,
    mut er_drop: EventReader<Dropped>,
    mut q_draggable: Query<(&mut Style, &mut ZIndex), With<Draggable>>,
    parent: Query<&Parent, With<Draggable>>,
    children: Query<&Children, With<Receiver>>,
) {
    for event in er_drop.read() {
        if let Some(received) = event.received {
            let ent_parent = parent.get(event.dropped).unwrap().get();
            commands.entity(event.dropped).remove_parent();

            let child = *children.get(received).unwrap().iter().next().unwrap();
            commands
                .entity(received)
                .remove_children(&[child])
                .add_child(event.dropped);
            commands.entity(ent_parent).add_child(child);
        }
        let (mut style, mut zindex) = q_draggable.get_mut(event.dropped).unwrap();
        style.left = Val::Auto;
        style.top = Val::Auto;
        *zindex = ZIndex::Local(0);
    }
}

fn on_dragged(
    mut er_drag: EventReader<Dragged>,
    mut q_draggable: Query<&mut ZIndex, With<Draggable>>,
) {
    for event in er_drag.read() {
        let mut zindex = q_draggable.get_mut(event.dragged).unwrap();
        *zindex = ZIndex::Global(15);
    }
}

fn on_hovered(
    mut er_hovered: EventReader<HoveredChange>,
    mut q_receiver: Query<&mut BackgroundColor, With<Receiver>>,
) {
    for event in er_hovered.read() {
        if let Some(receiver) = event.receiver {
            let mut color = q_receiver.get_mut(receiver).unwrap();
            *color = Color::srgb(0.45, 0.45, 0.45).into();
        }
        if let Some(receiver) = event.prevreceiver {
            let mut color = q_receiver.get_mut(receiver).unwrap();
            *color = Color::srgb(0.3, 0.3, 0.3).into();
        }
    }
}
