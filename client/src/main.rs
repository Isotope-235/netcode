use std::{
    thread,
    time::{Duration, Instant},
};

use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::BlendMode,
};

const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const SCALE: u32 = 8;
const FRAME_TIME: Duration = Duration::from_nanos(166_666_666);

fn main() {
    let sdl = sdl3::init().unwrap();

    let video = sdl.video().unwrap();

    let mut event_pump = sdl.event_pump().unwrap();

    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas();
    canvas
        .set_logical_size(
            LOGICAL_WIDTH,
            LOGICAL_HEIGHT,
            sdl3::sys::render::SDL_RendererLogicalPresentation::INTEGER_SCALE,
        )
        .unwrap();
    canvas.set_blend_mode(BlendMode::None);

    'game: loop {
        let start = Instant::now();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        event_pump.pump_events();
        for event in event_pump.poll_iter() {
            use sdl3::event::Event as Ev;

            match event {
                Ev::Quit { .. } => break 'game,
                Ev::KeyDown { .. } => {}
                Ev::MouseButtonDown { x, y, .. } => {
                    canvas.set_draw_color(Color::WHITE);
                    canvas
                        .fill_rect(Rect::from_center(
                            Point::new((x as u32 / SCALE) as _, (y as u32 / SCALE) as _),
                            20,
                            20,
                        ))
                        .unwrap();
                }
                _ => {}
            }
        }

        canvas.present();

        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }
}
