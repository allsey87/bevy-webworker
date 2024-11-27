use std::time::Duration;
use bevy_app::{App, PluginsState, Startup, Update};
use bevy_asset::Assets;
use bevy_ecs::{entity::Entity, query::With, system::{Commands, ResMut}};
use bevy_input::{mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel}, ButtonState};
use bevy_math::Vec2;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_pbr::{AmbientLight, StandardMaterial};
use bevy_rapier3d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_render::{camera::ClearColor, color::Color, mesh::Mesh};
use bevy_window::{CursorEntered, CursorLeft, CursorMoved, PrimaryWindow, Window, WindowResized};
use futures::{lock::Mutex, FutureExt, StreamExt};
use gloo_timers::future::{self, IntervalStream};
use tracing_subscriber::{prelude::*, EnvFilter};
use wasm_bindgen::prelude::*;

mod world;
mod drag;
mod offscreen;
mod camera;

#[wasm_bindgen(main)]
pub fn main() {
    /* configure panic hook for debugging */
    console_error_panic_hook::set_once();
    /* configure logging */
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .without_time()
            .with_writer(tracing_web::MakeConsoleWriter))
            .with(EnvFilter::from("worker=trace,shared=trace"))
        .init();
    /* start Bevy */
    bevy_app::App::new()
        .set_runner(|app| wasm_bindgen_futures::spawn_local({
            let scope = js_sys::global().dyn_into::<web_sys::DedicatedWorkerGlobalScope>().unwrap();
            /* create the RPC interface, this block evaluates to a server future which is passed to
               wasm_bindgen_futures */
            web_rpc::Interface::new(scope)
                .then(|interface| web_rpc::Builder::new(interface)
                    .with_service::<shared::BevyService<_>>(BevyServerImpl(Mutex::new(app)))
                    .build())
        }))
        .run();
}

struct BevyServerImpl(Mutex<App>);

impl shared::Bevy for BevyServerImpl {
    async fn init(
        &self,
        canvas: web_sys::OffscreenCanvas,
    ) -> Result<(), ()> {
        let mut app_locked = self.0.lock().await;
        app_locked
            /* configure simulator */
            .add_plugins(bevy_core::TaskPoolPlugin::default())
            .add_plugins(bevy_core::TypeRegistrationPlugin)
            .add_plugins(bevy_core::FrameCountPlugin)
            .add_plugins(bevy_time::TimePlugin)
            .add_plugins(bevy_transform::TransformPlugin)
            .add_plugins(bevy_hierarchy::HierarchyPlugin)
            .add_plugins(bevy_diagnostic::DiagnosticsPlugin)
            .add_plugins(bevy_input::InputPlugin)
            .add_plugins(offscreen::OffscreenPlugin::new(canvas))
            .add_plugins(bevy_asset::AssetPlugin::default())
            .add_plugins(bevy_scene::ScenePlugin)
            .add_plugins(bevy_render::RenderPlugin::default())
            .add_plugins(bevy_render::texture::ImagePlugin::default())
            .add_plugins(bevy_core_pipeline::CorePipelinePlugin)
            .add_plugins(bevy_pbr::PbrPlugin::default())
            .add_plugins(bevy_gltf::GltfPlugin::default())
            .add_plugins(bevy_gizmos::GizmoPlugin)
            /* simulation configuration */
            .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 500.0,
            })
            .insert_resource(RapierConfiguration {
                timestep_mode: TimestepMode::Fixed { dt: 0.05, substeps: 20 }, // 20 fps
                physics_pipeline_active: true,
                query_pipeline_active: true,
                ..Default::default()
            })
            .add_plugins(DefaultPickingPlugins)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            //.add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, move |
                commands: Commands,
                meshes: ResMut<Assets<Mesh>>,
                materials: ResMut<Assets<StandardMaterial>> | {
                    world::setup(commands, meshes, materials)
                })
            // custom systems for controlling the camera and dragging entities
            .add_systems(Update, (camera::update_camera_system, camera::accumulate_mouse_events_system))
            .add_systems(Update, drag::drag_system);
        /* wait until initialisation is complete before releasing the app lock */
        while app_locked.plugins_state() != PluginsState::Ready {
            future::sleep(Duration::default()).await;
        }
        app_locked.finish();
        app_locked.cleanup();

        Ok(())
    }
    
    async fn start(&self, update_interval: Duration) {
        let mut update = IntervalStream::new(update_interval.as_millis() as u32);
        loop {
            /* suspend for update_interval milliseconds */
            update.next().await;
            /* lock the app and update bevy's world */
            let mut app_locked = self.0.lock().await;
            app_locked.update();
            /* at this point the lock is dropped so that events can be sent to
               the bevy world while we wait to do the next update */
        }
    }

    async fn stop(&self) {
        // does this event have to be picked up by an event reader?
        self.0.lock().await.world.send_event(bevy_app::AppExit);
    }

    async fn process_event(&self, event: shared::Event) {
        /* wait for the world to be ready */
        let world = &mut self.0.lock().await.world;
        /* get the primary window */
        let (window_id, mut window) = world
            .query_filtered::<(Entity, &mut Window), With<PrimaryWindow>>()
            .single_mut(world);
        /* convert and send event */
        match event {
            shared::Event::Resize { width, height } => {
                window.resolution.set(width as f32, height as f32);
                world.send_event(WindowResized {
                    window: window_id,
                    width: width as f32,
                    height: height as f32,
                });
            },
            shared::Event::CursorEntered => {
                world.send_event(CursorEntered { window: window_id });
            }
            shared::Event::CursorLeft => {
                world.send_event(CursorLeft { window: window_id });
            },
            shared::Event::CursorMoved { delta, position: (x, y) } => {
                world.send_event(CursorMoved {
                    window: window_id,
                    position: Vec2::new(x as f32, y as f32),
                    delta: delta.map(|(x, y)| Vec2::new(x as f32, y as f32))
                });
            }
            shared::Event::MouseMotion { delta: (delta_x, delta_y) } => {
                world.send_event(MouseMotion {
                    delta: Vec2::new(delta_x as f32, delta_y as f32)
                });
            },
            shared::Event::MouseWheel { delta: (delta_x, delta_y), unit } => {
                world.send_event(MouseWheel {
                    window: window_id,
                    unit: match unit {
                        0 => MouseScrollUnit::Pixel,
                        1 => MouseScrollUnit::Line,
                        _ => unreachable!("invalid scroll unit")
                    },
                    x: delta_x as f32,
                    y: delta_y as f32,
                });
            }
            shared::Event::MouseButton { pressed, button } => {
                world.send_event(MouseButtonInput {
                    state: match pressed {
                        true => ButtonState::Pressed,
                        false => ButtonState::Released,
                    },
                    button: match button {
                        0 => MouseButton::Left,
                        1 => MouseButton::Middle,
                        2 => MouseButton::Right,
                        3 => MouseButton::Back,
                        4 => MouseButton::Forward,
                        other => MouseButton::Other(other as u16)
                    },
                    window: window_id,
                });
            }
        }
    }
}
