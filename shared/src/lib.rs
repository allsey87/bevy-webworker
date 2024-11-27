use std::time::Duration;

#[web_rpc::service]
pub trait Bevy {
    #[post(transfer(canvas))]
    async fn init(
        canvas: web_sys::OffscreenCanvas,
    ) -> Result<(), ()>;

    async fn start(interval: Duration);
    
    async fn stop();

    async fn process_event(
        event: Event,
    );
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Resize {
        width: u32,
        height: u32
    },
    CursorEntered,
    CursorLeft,
    CursorMoved {
        delta: Option<(i32, i32)>,
        position: (i32, i32)
    },
    MouseMotion {
        delta: (i32, i32)
    },
    MouseWheel {
        delta: (f64, f64),
        unit: u32
    },
    MouseButton {
        pressed: bool,
        button: i16,
    },
}