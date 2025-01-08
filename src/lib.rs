use std::ops::Mul;

use bevy::{prelude::*, window::PrimaryWindow};
use bitflags::bitflags;

// Todo: Add more methods for InputFlags maybe

bitflags! {
    #[derive(Clone,Copy)]
    /// Flags that keep track of relevant inputs.
    pub struct InputFlags: u8 {
        const LeftClick = 0b00000001;
        const RightClick = 0b00000010;
        const MiddleClick = 0b00000100;
        const Shift = 0b00001000;
        const Ctrl = 0b00010000;
        const Alt = 0b00100000;
        const Clicks = 0b00000111;
        const Modifiers = 0b00111000;
    }
}

impl Mul<u8> for InputFlags {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        Self::from_bits_truncate(self.bits() * rhs)
    }
}

/// Event that is sent when an entity is released
#[derive(Event)]
pub struct Dropped {
    /// Entity that was dropped
    pub dropped: Entity,
    /// Entity that received the dropped entity if any.
    pub received: Option<Entity>,
    /// Inputs at the time of the event being sent
    pub inputs: InputFlags,
}

/// Event that is sent when an entity has just begun being dragged
#[derive(Event)]
pub struct Dragged {
    /// Entity that is being dragged
    pub dragged: Entity,
    /// Inputs at the time of the event being sent
    pub inputs: InputFlags,
}

/// Event that is sent when an entity is waiting for a minimum time to elapse to initiate dragging
#[derive(Event)]
pub struct DragAwait {
    /// Entity that is awaiting to be dragged
    pub awaiting: Entity,
    /// Inputs at the time of the event being sent
    pub inputs: InputFlags,
}

/// Event that is sent when an entity is hovered over a new receiver, and when it is dropped.
#[derive(Event)]
pub struct HoveredChange {
    /// The entity that is being dragged
    pub hovered: Entity,
    /// The entity that is now being hovered over, None if no receivers are being hovered over or if it has been dropped.
    pub receiver: Option<Entity>,
    /// The last entity that was being hovered over if any
    pub prevreceiver: Option<Entity>,
    /// Inputs at the time of the event being sent
    pub inputs: InputFlags,
}

/// Component that may be attached to anything with a transform and GlobalTransform component to give it draggable functionality.
#[derive(Component)]
pub struct Draggable {
    /// All of these inputs must be pressed down for dragging to initiate.
    pub required: InputFlags,
    /// Dragging will not initiate if any of these are held down.
    pub disallowed: InputFlags,
    /// Minimum amount of time for buttons to be held before dragging initiates in seconds.
    pub minimum_held: Option<f64>,
}

impl Default for Draggable {
    fn default() -> Self {
        Draggable {
            required: InputFlags::LeftClick,
            disallowed: InputFlags::RightClick | InputFlags::MiddleClick,
            minimum_held: None,
        }
    }
}

/// Component used to designate when an object is actively being dragged.
#[derive(Component)]
pub struct Dragging {
    pub hovering: Option<Entity>,
}

/// Component used to designate when an object is waiting to be able to be dragged.
#[derive(Component)]
pub struct AwaitingDrag {
    pub ends: f64,
}

/// Component that may be attached to anything with a transform and GlobalTransform component to allow it to be detected when a draggable is dropped over it.
#[derive(Component)]
pub struct Receiver;

/// Plugin that contains systems and events for dragging and dropping.
pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                startdrag,
                dragging.before(drop),
                drop.after(dragging),
                awaitdrag,
            ),
        )
        .add_event::<Dropped>()
        .add_event::<Dragged>()
        .add_event::<DragAwait>()
        .add_event::<HoveredChange>();
    }
}

fn startdrag(
    mut commands: Commands,
    q_draggable: Query<(
        &GlobalTransform,
        Option<&Handle<Image>>,
        Entity,
        Option<&Node>,
        &Draggable,
    )>,
    dragging: Query<&Dragging>,
    awaiting: Query<&AwaitingDrag>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
    mut ew_dragged: EventWriter<Dragged>,
    mut ew_await: EventWriter<DragAwait>,
    time: Res<Time<Real>>,
) {
    let inputs = get_inputs(&keys, &buttons);
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    let mut candidates: Vec<(Entity, f32, &Draggable)> = Vec::new();

    if inputs.intersects(InputFlags::Clicks) && dragging.is_empty() && awaiting.is_empty() {
        if let Some(logical_position) = window.cursor_position() {
            let world_position = camera
                .viewport_to_world(camera_transform, logical_position)
                .map(|ray| ray.origin.truncate())
                .unwrap();
            for (gtransform, image_handle, entity, node, draggable) in q_draggable.iter() {
                if is_in_bounds(
                    gtransform,
                    image_handle,
                    node,
                    &assets,
                    logical_position,
                    world_position,
                ) && inputs.contains(draggable.required)
                    && !(inputs.intersects(draggable.disallowed))
                {
                    candidates.push((entity, gtransform.translation().z, draggable));
                }
            }
        }
        if !candidates.is_empty() {
            //Get the candidate with the highest Z
            let mut final_candidate = candidates[0];
            for candidate in candidates {
                if candidate.1 > final_candidate.1 {
                    final_candidate = candidate;
                }
            }
            if let Some(x) = final_candidate.2.minimum_held {
                ew_await.send(DragAwait {
                    awaiting: final_candidate.0,
                    inputs,
                });
                commands.entity(final_candidate.0).insert(AwaitingDrag {
                    ends: time.elapsed_seconds_f64() + x,
                });
                return;
            }
            ew_dragged.send(Dragged {
                dragged: final_candidate.0,
                inputs,
            });
            commands
                .entity(final_candidate.0)
                .insert(Dragging { hovering: None });
        }
    }
}

fn awaitdrag(
    mut commands: Commands,
    q_draggable: Query<(Entity, &Draggable, &AwaitingDrag)>,
    mut ew_dragged: EventWriter<Dragged>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
) {
    let inputs = get_inputs(&keys, &buttons);

    for (entity, draggable, awaiting) in q_draggable.iter() {
        if inputs.contains(draggable.required) && !(inputs.intersects(draggable.disallowed)) {
            if time.elapsed_seconds_f64() > awaiting.ends {
                ew_dragged.send(Dragged {
                    dragged: entity,
                    inputs,
                });
                commands
                    .entity(entity)
                    .insert(Dragging { hovering: None })
                    .remove::<AwaitingDrag>();
            }
            return;
        }
        commands.entity(entity).remove::<AwaitingDrag>();
    }
}

fn dragging(
    q_parent: Query<&GlobalTransform>,
    mut q_dragging: Query<(
        &Parent,
        &mut Transform,
        Option<&mut Style>,
        &mut Dragging,
        Entity,
    )>,
    q_receivers: Query<
        (
            &GlobalTransform,
            Option<&Handle<Image>>,
            Entity,
            Option<&Node>,
        ),
        With<Receiver>,
    >,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
    mut ew_hover: EventWriter<HoveredChange>,
) {
    let inputs = get_inputs(&keys, &buttons);
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    for (parent, mut transform, style, mut dragging, entity) in q_dragging.iter_mut() {
        let gtransform = q_parent.get(parent.get()).unwrap();
        if let Some(logical_position) = window.cursor_position() {
            let world_position = camera
                .viewport_to_world(camera_transform, logical_position)
                .map(|ray| ray.origin.truncate())
                .unwrap();
            let mut mat = gtransform.compute_matrix();
            mat = mat.inverse();

            if let Some(mut style) = style {
                let local_point4 =
                    mat.mul_vec4(Vec4::new(logical_position.x, logical_position.y, 0.0, 1.0));
                let local_point =
                    Vec3::new(local_point4.x, local_point4.y, transform.translation.z);

                style.left = Val::Px(local_point.x);
                style.top = Val::Px(local_point.y);
            } else {
                let local_point4 =
                    mat.mul_vec4(Vec4::new(world_position.x, world_position.y, 0.0, 1.0));
                let local_point =
                    Vec3::new(local_point4.x, local_point4.y, transform.translation.z);

                transform.translation = local_point;
            }

            for (gtransform, image_handle, receiver, node) in q_receivers.iter() {
                if is_in_bounds(
                    gtransform,
                    image_handle,
                    node,
                    &assets,
                    logical_position,
                    world_position,
                ) {
                    if let Some(hovered) = dragging.hovering {
                        if hovered == receiver {
                            return;
                        }
                    }
                    ew_hover.send(HoveredChange {
                        hovered: entity,
                        prevreceiver: dragging.hovering,
                        receiver: Some(receiver),
                        inputs,
                    });
                    dragging.hovering = Some(receiver);
                    return;
                }
            }
            if dragging.hovering.is_some() {
                ew_hover.send(HoveredChange {
                    hovered: entity,
                    prevreceiver: dragging.hovering,
                    receiver: None,
                    inputs,
                });
                dragging.hovering = None;
            }
        }
    }
}

fn drop(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_receivers: Query<
        (
            &GlobalTransform,
            Option<&Handle<Image>>,
            Entity,
            Option<&Node>,
        ),
        With<Receiver>,
    >,
    q_dragging: Query<(Entity, &Draggable, &Dragging)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut ew_dropped: EventWriter<Dropped>,
    mut ew_hover: EventWriter<HoveredChange>,
    assets: Res<Assets<Image>>,
) {
    let inputs = get_inputs(&keys, &buttons);
    if q_dragging.is_empty() {
        return;
    }
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    if let Some(logical_position) = window.cursor_position() {
        let world_position = camera
            .viewport_to_world(camera_transform, logical_position)
            .map(|ray| ray.origin.truncate())
            .unwrap();
        for (gtransform, image_handle, entity, node) in q_receivers.iter() {
            if is_in_bounds(
                gtransform,
                image_handle,
                node,
                &assets,
                logical_position,
                world_position,
            ) {
                for (drag_entity, draggable, dragging) in q_dragging.iter() {
                    if !inputs.intersects(draggable.required & InputFlags::Clicks) {
                        ew_hover.send(HoveredChange {
                            hovered: drag_entity,
                            receiver: None,
                            prevreceiver: dragging.hovering,
                            inputs,
                        });
                        ew_dropped.send(Dropped {
                            dropped: drag_entity,
                            received: Some(entity),
                            inputs,
                        });
                        commands.entity(drag_entity).remove::<Dragging>();
                    }
                }
                return;
            }
        }
        for (entity, draggable, dragging) in q_dragging.iter() {
            if !inputs.intersects(draggable.required & InputFlags::Clicks) {
                ew_hover.send(HoveredChange {
                    hovered: entity,
                    receiver: None,
                    prevreceiver: dragging.hovering,
                    inputs,
                });
                ew_dropped.send(Dropped {
                    dropped: entity,
                    received: None,
                    inputs,
                });
                commands.entity(entity).remove::<Dragging>();
            }
        }
    }
}

fn is_in_bounds(
    gtransform: &GlobalTransform,
    image_handle: Option<&Handle<Image>>,
    node: Option<&Node>,
    assets: &Res<Assets<Image>>,
    logical_position: Vec2,
    world_position: Vec2,
) -> bool {
    if let Some(node) = node {
        let bounding_box = node.logical_rect(gtransform);
        bounding_box.contains(logical_position)
    } else {
        let transform = gtransform.compute_transform();
        let mut scaled_image_dimension = transform.scale.truncate();

        //Need to account for sprite size if it is a sprite.
        if let Some(img) = image_handle {
            scaled_image_dimension *= assets.get(img).unwrap().size().as_vec2();
        }

        let bounding_box =
            Rect::from_center_size(gtransform.translation().truncate(), scaled_image_dimension);
        bounding_box.contains(world_position)
    }
}

fn get_inputs(keys: &Res<ButtonInput<KeyCode>>, buttons: &Res<ButtonInput<MouseButton>>) -> InputFlags {
    (InputFlags::LeftClick * (buttons.pressed(MouseButton::Left) as u8))
        | (InputFlags::RightClick * (buttons.pressed(MouseButton::Right) as u8))
        | (InputFlags::MiddleClick * (buttons.pressed(MouseButton::Middle) as u8))
        | (InputFlags::Shift
            * ((keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)) as u8))
        | (InputFlags::Ctrl
            * ((keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)) as u8))
        | (InputFlags::Alt
            * ((keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)) as u8))
}
