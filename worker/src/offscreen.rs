use bevy_app::{App, Plugin};
use bevy_window::{   
    CursorEntered, CursorLeft, CursorMoved, 
    PrimaryWindow, RawHandleWrapper, Window, WindowResolution,
    WindowClosed, WindowScaleFactorChanged, WindowCreated, WindowResized,
};
use thread_safe::ThreadSafe;
use web_sys::OffscreenCanvas;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WebDisplayHandle, WebOffscreenCanvasWindowHandle};

pub struct OffscreenPlugin {
    canvas: ThreadSafe<OffscreenCanvas>,
}

impl OffscreenPlugin {
    pub fn new(canvas: OffscreenCanvas) -> OffscreenPlugin {
        OffscreenPlugin {
            canvas: ThreadSafe::new(canvas)
        }
    }
}

impl Plugin for OffscreenPlugin {
    fn build(&self, simulator: &mut App) {
        let window_handle =
            WebOffscreenCanvasWindowHandle::from_wasm_bindgen_0_2(self.canvas.get_ref());
        simulator
            .add_event::<WindowResized>()
            .add_event::<WindowCreated>()
            .add_event::<WindowClosed>()
            .add_event::<WindowScaleFactorChanged>()
            .add_event::<CursorEntered>()
            .add_event::<CursorLeft>()
            .add_event::<CursorMoved>();
        simulator.world
            .spawn(Window {
                resolution: WindowResolution::new(500.0, 500.0),
                ..Default::default()
            })
            .insert(PrimaryWindow)
            .insert(RawHandleWrapper {
                window_handle: RawWindowHandle::WebOffscreenCanvas(window_handle),
                display_handle: RawDisplayHandle::Web(WebDisplayHandle::new()),
            });
    }
}

