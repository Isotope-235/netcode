use std::time::{Duration, Instant};

const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 320;
const LOGICAL_HEIGHT: u32 = 240;
const SCALE: u32 = 4;
const INT_SCALE: bool = true;
const BLEND_MODE: sdl2::render::BlendMode = sdl2::render::BlendMode::None;

pub struct SdlContext {
    pub events: sdl2::EventPump,
    pub canvas: sdl2::render::WindowCanvas,
    pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

pub fn init_sdl_systems(
    sdl: &sdl2::Sdl,
    video: &sdl2::VideoSubsystem,
) -> Result<SdlContext, Box<dyn std::error::Error>> {
    let events = sdl.event_pump()?;
    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    canvas.set_logical_size(LOGICAL_WIDTH, LOGICAL_HEIGHT)?;
    canvas.set_integer_scale(INT_SCALE)?;
    canvas.set_blend_mode(BLEND_MODE);

    let texture_creator = canvas.texture_creator();

    Ok(SdlContext {
        events,
        canvas,
        texture_creator,
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
