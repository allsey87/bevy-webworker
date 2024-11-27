use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    event::EventReader,
    query::With,
    system::{Commands, Query}
};
use bevy_eventlistener::{callbacks::Listener, event_listener::{EntityEvent, On}};
use bevy_math::{Vec2, Vec3};
use bevy_render::camera::Camera;
use bevy_transform::components::GlobalTransform;
use bevy_mod_picking::{
    events::{Drag, DragEnd, DragStart, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    pointer::PointerButton
};
use bevy_rapier3d::dynamics::ExternalImpulse;

#[derive(Component)]
pub struct Target {
    /// the camera on which this drag is occuring
    pub camera: Entity,

    /// allows calculating the drag target from the mouse
    pub origin: Vec3,

    /// the offset from the center of mass where the drag started
    pub offset: Vec3,

    /// distance of the drag (as last reported by events<pointer<drag>>)
    pub distance: Vec2,
}

#[derive(Bundle)]
pub struct DraggableBundle {
    drag_start: On::<Pointer<DragStart>>,
    drag_end: On::<Pointer<DragEnd>>,
    external_impulse: ExternalImpulse,
    pickable: Pickable,
    interaction: PickingInteraction,
}

impl Default for DraggableBundle {
    fn default() -> Self {
        Self {
            drag_start: On::<Pointer<DragStart>>::run(drag_start_system),
            drag_end: On::<Pointer<DragEnd>>::run(drag_end_system),
            external_impulse: Default::default(),
            pickable: Default::default(),
            interaction: Default::default()
        }
    }
}

fn drag_start_system(
    listener: Listener<Pointer<DragStart>>,
    target: Query<&GlobalTransform, With<ExternalImpulse>>,
    mut commands: Commands
) {
    if listener.button == PointerButton::Primary {
        if let Ok(target_transform) = target.get(listener.target()) {
            let position = listener.hit.position
                .expect("backend does not support `position`");
            commands.entity(listener.target()).insert(Target {
                camera: listener.hit.camera,
                origin: position,
                offset: target_transform.affine().inverse().transform_point3(position),
                distance: Default::default()
            });
        }   
    }
}

fn drag_end_system(
    listener: Listener<Pointer<DragEnd>>,
    mut commands: Commands
) {
    commands.entity(listener.target()).remove::<Target>();
}

pub fn drag_system(
    mut drag_events: EventReader<Pointer<Drag>>,
    mut target: Query<(&mut Target, &GlobalTransform, &mut ExternalImpulse)>,
    camera_transforms: Query<&GlobalTransform, With<Camera>>,
) {
    if let Ok((mut target, target_transform, mut target_force)) = target.get_single_mut() {
        /* update the cached target distance */
        if let Some(last_drag_event) = drag_events.read().last() {
            target.distance = last_drag_event.distance;
        }

        /* convert drag target distance  */
        let camera_transform = camera_transforms
            .get(target.camera)
            .unwrap();
        let mut drag_target_offset = camera_transform.translation() +
            target.distance.x * camera_transform.right() -
            target.distance.y * camera_transform.up();
        drag_target_offset.y = 0.0;

        // TODO: improve zoom factor for lower camera altitudes
        let zoom_factor = (camera_transform.translation() - target.origin).length() * 0.0011;
        let drag_target = target.origin + (drag_target_offset * zoom_factor);
        let drag_point = target_transform.transform_point(target.offset);

        // TODO: make gain a factor of object weight
        const GAIN: f32 = 1.5;
        // TODO: use PID control?
        let drag_impulse = (drag_target - drag_point)
            .clamp(Vec3::NEG_ONE, Vec3::ONE) * GAIN;
        target_force.impulse = drag_impulse;

        let mut drag_com_offset = drag_point - target_transform.translation();
        drag_com_offset.y = 0.0;

        let orthogonal_vector = (drag_com_offset) - (drag_com_offset).project_onto(drag_impulse);
        target_force.torque_impulse = orthogonal_vector.cross(drag_impulse);
    }
}