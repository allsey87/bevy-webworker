
use dominator::{clone, html};
use futures_signals::signal::SignalExt;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod events;

#[allow(non_snake_case)]
#[wasm_bindgen(inline_js = "
export function newWorker() {
    return new Worker(new URL('../../worker.js', import.meta.url), {
        type: 'module'
    });
}
"
)]
extern "C" {
    #[wasm_bindgen(js_name = newWorker, catch)]
    pub fn bevy() -> Result<web_sys::Worker, JsValue>;
}

#[wasm_bindgen(main)]
pub async fn main() {
    /* start the Bevy web worker */
    let bevy_worker = bevy().expect("could not create bevy worker");
    let bevy_interface = web_rpc::Interface::new(bevy_worker).await;
    let bevy_client = web_rpc::Builder::new(bevy_interface)
        .with_client::<shared::BevyClient>()
        .build();

    /* create a canvas for Bevy to render onto */
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    /* create the offscreen canvas and start Bevy */
    let offscreen_canvas = canvas.transfer_control_to_offscreen()
        .expect("could not transfer control to offscreen");
    bevy_client.init(offscreen_canvas).await
        .expect("could not initialize bevy");
    bevy_client.start(std::time::Duration::from_millis(25));

    let canvas_size = dominator::window_size()
        .map(|size| ((size.width - 200.0).max(0.0) as u32, (size.height - 200.0).max(0.0) as u32))
        .broadcast();

    dominator::append_dom(&dominator::body(), html!("div", {
        .style("padding", "100px")
        .style("width", "100%")
        .style("height", "100%")
        .future(canvas_size.signal().for_each(clone!(bevy_client =>
            move |(width, height)| clone!(bevy_client =>
                async move {
                    bevy_client.process_event(shared::Event::Resize { width, height });
                }
            )
        )))
        .after_inserted(clone!(canvas => move |node| {
            node.append_child(&canvas).unwrap();
        }))
    }));
    
    /* register event handlers against the canvas */
    let handlers = events::register(&canvas, &bevy_client);
    /* do not drop the event handlers since this would cause them to be unregistered */
    std::mem::forget(handlers);
}
