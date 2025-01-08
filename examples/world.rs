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

    let mut rng = thread_rng();

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.40, 0.40, 0.40),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(1280.0, 720.0, 1.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.10, 0.10, 0.10),
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(0.5625, 1.0, 1.0),
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for x in 0..5 {
                        for y in 0..5 {
                            parent
                                .spawn(SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.75, 0.75, 0.75),
                                        ..default()
                                    },
                                    transform: Transform {
                                        scale: Vec3::new(0.2, 0.2, 1.0),
                                        translation: Vec3::new(
                                            -0.4 + (x as f32 * 0.2),
                                            -0.4 + (y as f32 * 0.2),
                                            1.0,
                                        ),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            SpriteBundle {
                                                sprite: Sprite {
                                                    color: Color::srgb(0.10, 0.10, 0.10),
                                                    ..default()
                                                },
                                                transform: Transform {
                                                    scale: Vec3::new(0.96, 0.96, 1.0),
                                                    translation: Vec3::new(0.0, 0.0, 1.0),
                                                    ..default()
                                                },
                                                ..default()
                                            },
                                            Receiver,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                SpriteBundle {
                                                    sprite: Sprite {
                                                        color: Color::hsl(
                                                            rng.gen::<f32>() * 360.0,
                                                            1.0,
                                                            0.5,
                                                        ),
                                                        ..default()
                                                    },
                                                    transform: Transform {
                                                        scale: Vec3::new(
                                                            0.00390625, 0.00390625, 0.00390625,
                                                        ),
                                                        translation: Vec3::new(0.0, 0.0, 1.0),
                                                        ..default()
                                                    },
                                                    texture: icon.clone(),
                                                    ..default()
                                                },
                                                Draggable {
                                                    required: InputFlags::LeftClick,
                                                    disallowed: InputFlags::RightClick
                                                        | InputFlags::MiddleClick,
                                                    minimum_held: Some(0.15),
                                                },
                                            ));
                                        });
                                });
                        }
                    }
                });
        });
}

fn on_dropped(
    mut commands: Commands,
    mut er_drop: EventReader<Dropped>,
    mut q_draggable: Query<&mut Transform, With<Draggable>>,
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
        let mut transform = q_draggable.get_mut(event.dropped).unwrap();
        transform.translation = Vec3::new(0.0, 0.0, 1.0);
    }
}

fn on_dragged(
    mut er_drag: EventReader<Dragged>,
    mut q_draggable: Query<&mut Transform, With<Draggable>>,
) {
    for event in er_drag.read() {
        let mut transform = q_draggable.get_mut(event.dragged).unwrap();
        transform.translation.z = 15.0;
    }
}

fn on_hovered(
    mut er_hovered: EventReader<HoveredChange>,
    mut q_receiver: Query<&mut Sprite, With<Receiver>>,
) {
    for event in er_hovered.read() {
        if let Some(receiver) = event.receiver {
            let mut sprite = q_receiver.get_mut(receiver).unwrap();
            sprite.color = Color::srgb(0.3, 0.3, 0.3);
        }
        if let Some(receiver) = event.prevreceiver {
            let mut sprite = q_receiver.get_mut(receiver).unwrap();
            sprite.color = Color::srgb(0.1, 0.1, 0.1);
        }
    }
}
