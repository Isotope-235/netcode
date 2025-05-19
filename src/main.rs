use std::{
    thread,
    time::{Duration, Instant},
};

use sdl3::{
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas, FRect},
    video::Window,
};

mod networking;

const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const SCALE: u32 = 8;
const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);

fn main() {
    let mut state = Game {
        players: vec![Player {
            pos: Vec2 {
                x: (LOGICAL_WIDTH / 2) as _,
                y: (LOGICAL_HEIGHT / 2) as _,
            },
            color: Color::RED,
            size: 10.0,
        }],
        platforms: Vec::new(),
    };

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

    let mut movement = Vec2 { x: 0.00, y: 0.00 };

    'game: loop {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            use sdl3::event::Event as Ev;

            match event {
                Ev::Quit { .. } => break 'game,
                Ev::KeyDown {
                    keycode,
                    repeat: false,
                    ..
                } => match keycode {
                    Some(kc) => match kc {
                        Keycode::A => movement.x -= 1.00,
                        Keycode::D => movement.x += 1.00,
                        Keycode::W => movement.y -= 1.00,
                        Keycode::S => movement.y += 1.00,
                        _ => (),
                    },
                    None => {}
                },
                Ev::KeyUp {
                    keycode,
                    repeat: false,
                    ..
                } => match keycode {
                    Some(kc) => match kc {
                        Keycode::A => movement.x += 1.00,
                        Keycode::D => movement.x -= 1.00,
                        Keycode::W => movement.y += 1.00,
                        Keycode::S => movement.y -= 1.00,
                        _ => (),
                    },
                    None => {}
                },
                _ => {}
            }
        }

        send(movement);
        render(&state, &mut canvas);
        canvas.present();

        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }
}

fn render(game: &Game, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();

    canvas.set_draw_color(Color::BLACK);
    for platform in &game.platforms {
        let _ = canvas.draw_rect(FRect::new(
            platform.pos.x,
            platform.pos.y,
            platform.size.0,
            platform.size.1,
        ));
    }

    for player in &game.players {
        canvas.set_draw_color(player.color);
        let r = Rect::from_center(
            Point::new(player.pos.x as _, player.pos.y as _),
            player.size as _,
            player.size as _,
        );
        let _ = canvas.fill_rect(r);
    }
}

fn send(moved: Vec2) {
    println!("sent movement: {:?}", moved);
}

struct Game {
    platforms: Vec<Platform>,
    players: Vec<Player>,
}

#[derive(Clone, Copy)]
struct Platform {
    size: (f32, f32),
    pos: Vec2,
}

impl Platform {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Platform {
            size: (width, height),
            pos: Vec2::new(x, y),
        }
    }
}

struct Player {
    pos: Vec2,
    size: f32,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
}
