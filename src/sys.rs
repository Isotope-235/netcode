use std::{
    ops::Not,
    time::{Duration, Instant},
};

use sdl2::render::BlendMode;

const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const SCALE: u32 = 8;

#[allow(dead_code)]
pub struct SdlContext {
    sdl: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    pub events: sdl2::EventPump,
    pub canvas: sdl2::render::WindowCanvas,
}

pub fn init_sdl() -> Result<SdlContext, Box<dyn std::error::Error>> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let events = sdl.event_pump()?;
    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    canvas.set_logical_size(
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
    )?;
    canvas.set_integer_scale(true)?;
    canvas.set_blend_mode(BlendMode::None);

    Ok(SdlContext {
        sdl,
        video,
        events,
        canvas,
    })
}

#[derive(Clone, Copy)]
pub struct Ticker {
    frame_time: Duration,
}

pub fn ticker(frame_time: Duration) -> Ticker {
    Ticker { frame_time }
}

impl Ticker {
    pub fn start(self) -> Tick {
        Tick {
            frame_time: self.frame_time,
            start: Instant::now(),
        }
    }
}

pub struct Tick {
    frame_time: Duration,
    start: Instant,
}

impl Tick {
    pub fn wait(self) {
        let elapsed = self.start.elapsed();
        let wait = self.frame_time.checked_sub(elapsed);
        std::thread::sleep(wait.unwrap_or_default());
    }
}
