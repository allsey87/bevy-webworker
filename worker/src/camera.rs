use bevy_ecs::{component::Component, event::EventReader, query::With, system::{Query, Res}};
use bevy_input::{mouse::{MouseButton, MouseMotion, MouseWheel}, ButtonInput};
use bevy_math::{Mat3, Quat, Vec2, Vec3};
use bevy_render::camera::Projection;
use bevy_transform::components::Transform;
use bevy_window::{PrimaryWindow, Window};

const LERP: f32 = 0.1;

#[derive(Component)]
pub struct PanOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    // These accumulate the events
    pub pan: Vec2,
    pub rotation_move: Vec2,
    pub scroll: f32,
    pub orbit_button_changed: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
            // These accumulate the events
            pan: Vec2::ZERO,
            rotation_move: Vec2::ZERO,
            scroll: 0.0,
            orbit_button_changed: false,
        }
    }
}

pub fn accumulate_mouse_events_system(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut PanOrbitCamera>,
) {
    // need to accumulate these and apply them to all cameras
    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;
    
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for mut camera in query.iter_mut() {
        camera.orbit_button_changed |= orbit_button_changed;
        camera.pan += 2.0 * pan;
        camera.rotation_move += 2.0 * rotation_move;
        camera.scroll += -0.1 * scroll;
    }

    ev_motion.clear();
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn update_camera_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    for (mut camera, mut transform, projection) in query.iter_mut() {
        if camera.orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            camera.upside_down = up.y <= 0.0;
            
            camera.orbit_button_changed = false;
        }

        let mut any = false;
        if camera.rotation_move.length_squared() > 0.5 {
            any = true;
            let rotation_move = camera.rotation_move * LERP;
            camera.rotation_move -= rotation_move;

            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if camera.upside_down { -delta } else { delta }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } 
        
        if camera.pan.length_squared() > 0.5 {
            any = true;
            let mut pan = camera.pan * LERP;
            camera.pan -= pan;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * camera.radius;
            camera.focus += translation;
        } 
        
        if camera.scroll.abs() > 0.5 {
            any = true;
            
            let scroll = camera.scroll * LERP;
            camera.scroll -= scroll;
            camera.radius -= scroll * camera.radius * 0.05;
            // dont allow zoom to reach zero or you get stuck
            camera.radius = f32::max(camera.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation = camera.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, camera.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    
}

fn get_primary_window_size(windows: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    let window = windows.get_single().unwrap();
    Vec2::new(window.width(), window.height())
}
