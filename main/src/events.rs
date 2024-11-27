use std::{collections::HashSet, sync::{Arc, RwLock}};

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use gloo_events::{EventListener, EventListenerOptions};
use dominator::clone;

use shared::{Event, BevyClient};

pub fn register(
    canvas: &HtmlCanvasElement,
    bevy_client: &BevyClient
) -> Vec<EventListener> {
    
    /* track which buttons were pressed over the canvas */
    let pressed_buttons: Arc<RwLock<HashSet<i16>>> = Default::default();
    
    /* this event clears any pointer down events when moving beyond the body */
    let global_pointer_out = EventListener::new(
        &dominator::body(),
        "pointerout",
        clone!(bevy_client, pressed_buttons => move |_| {
            for button in pressed_buttons.write().unwrap().drain() {
                bevy_client.process_event(
                    Event::MouseButton { pressed: false, button }
                );
            }
        }
    ));

    let global_pointer_up = EventListener::new(
        &dominator::body(),
        "pointerup",
        clone!(bevy_client, pressed_buttons => move |event| {
            let button = event.unchecked_ref::<web_sys::PointerEvent>().button();
            if pressed_buttons.write().unwrap().remove(&button) {
                bevy_client.process_event(
                    Event::MouseButton { pressed: false, button }
                );
            }
        }
    ));
    
    let mut global_last_position = None;
    let global_pointer_move = EventListener::new(
        &dominator::body(),
        "pointermove",
        clone!(bevy_client => move |event| {
            let event = event.unchecked_ref::<web_sys::PointerEvent>();
            let position = (event.offset_x(), event.offset_y());
            let delta = global_last_position
                .map(|(last_x, last_y)| ((position.0 - last_x), (position.1 - last_y)));
            global_last_position = Some(position);
            if let Some(delta) = delta {
                bevy_client.process_event(
                    Event::MouseMotion {
                        delta,
                    }
                );
            }
        }
    ));

    let mut last_position = None;
    let pointer_move = EventListener::new(
        canvas,
        "pointermove",
        clone!(bevy_client => move |event| {
            let event = event.unchecked_ref::<web_sys::PointerEvent>();
            let position = (event.offset_x(), event.offset_y());
            let delta = last_position
                .map(|(last_x, last_y)| ((position.0 - last_x), (position.1 - last_y)));
            last_position = Some(position);
            bevy_client.process_event(
                Event::CursorMoved {
                    delta,
                    position
                }
            );
        }
    ));

    let pointer_down = EventListener::new_with_options(
        canvas,
        "pointerdown",
        EventListenerOptions::enable_prevent_default(),
        clone!(bevy_client => move |event| {
            event.prevent_default();
            let button = event.unchecked_ref::<web_sys::PointerEvent>().button();
            pressed_buttons.write().unwrap().insert(button);
            bevy_client.process_event(
                Event::MouseButton { pressed: true, button }
            );
        }
    ));

    let pointer_over = EventListener::new(
        canvas,
        "pointerover",
        clone!(bevy_client => move |_| {
            bevy_client.process_event(Event::CursorEntered);
        }
    ));

    let pointer_out = EventListener::new(
        canvas,
        "pointerout",
        clone!(bevy_client => move |_| {
            bevy_client.process_event(Event::CursorLeft);
        }
    ));

    let wheel = EventListener::new(
        canvas,
        "wheel",
        clone!(bevy_client => move |event| {
            let event = event.unchecked_ref::<web_sys::WheelEvent>();
            bevy_client.process_event(
                Event::MouseWheel {
                    delta: (event.delta_x(), event.delta_y()),
                    unit: event.delta_mode()
                }
            );
        }
    ));

    /* disable the context menu over the canvas */
    let context_menu = EventListener::new_with_options(
        canvas,
        "contextmenu",
        EventListenerOptions::enable_prevent_default(),
        |event| {
        event.prevent_default();
    });

    vec![
        global_pointer_up,
        global_pointer_out,
        global_pointer_move,
        pointer_move,
        pointer_down,
        pointer_over,
        pointer_out,
        wheel,
        context_menu
    ]
}