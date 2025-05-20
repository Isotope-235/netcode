use std::time::{Duration, Instant};

use sdl3::render::BlendMode;

const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const SCALE: u32 = 8;

#[allow(dead_code)]
pub struct SdlContext {
    sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    pub events: sdl3::EventPump,
    pub canvas: sdl3::render::WindowCanvas,
}

pub fn init_sdl() -> Result<SdlContext, Box<dyn std::error::Error>> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let events = sdl.event_pump()?;
    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()?;
    let mut canvas = window.into_canvas();
    canvas.set_logical_size(
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        sdl3::sys::render::SDL_RendererLogicalPresentation::INTEGER_SCALE,
    )?;
    canvas.set_blend_mode(BlendMode::None);

    Ok(SdlContext {
        sdl,
        video,
        events,
        canvas,
    })
}

pub fn tick(start: Instant, frame_time: Duration) {
    let elapsed = start.elapsed();
    let wait = frame_time.checked_sub(elapsed);
    std::thread::sleep(wait.unwrap_or_default());
}
