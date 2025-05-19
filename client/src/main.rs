use std::{thread, time::{Duration, Instant}};

use sdl3::render::BlendMode;

const FRAME_TIME: Duration = Duration::from_nanos(166_666_666);

fn main() {
    let sdl = sdl3::init().unwrap();
    
    let video = sdl.video().unwrap();
    
    let mut event_pump = sdl.event_pump().unwrap();
    
    let window = video.window("hello", 800, 600).build().unwrap();
    let mut canvas = window.into_canvas();
    canvas.set_logical_size(80, 60, sdl3::sys::render::SDL_RendererLogicalPresentation::INTEGER_SCALE).unwrap();
    canvas.set_blend_mode(BlendMode::None);
    
    loop {
        let start = Instant::now();
        
        canvas.present();
        
        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }
}
