use std::{
    error::Error, net::{Ipv4Addr, UdpSocket}, thread, time::{Duration, Instant}
};

use sdl3::{
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas, FRect},
    video::Window,
};

mod cfg;
mod networking;

const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 56665;
const SERVER_HOST: Ipv4Addr = HOST;
const SERVER_PORT: u16 = 7878;
const TITLE: &str = "netcode";
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const SCALE: u32 = 8;
const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);

fn main() -> Result<(), Box<dyn Error>> {
    let state = Game {
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

    let sdl = sdl3::init()?;

    let video = sdl.video()?;

    let mut event_pump = sdl.event_pump()?;

    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()?;
    let mut canvas = window.into_canvas();
    canvas
        .set_logical_size(
            LOGICAL_WIDTH,
            LOGICAL_HEIGHT,
            sdl3::sys::render::SDL_RendererLogicalPresentation::INTEGER_SCALE,
        )?;
    canvas.set_blend_mode(BlendMode::None);

    let mut movement = (0, 0);
    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((SERVER_HOST, SERVER_PORT))?;

    'game: loop {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            use sdl3::event::Event as Ev;

            match event {
                Ev::Quit { .. } => break 'game,
                Ev::KeyDown {
                    keycode: Some(kc),
                    repeat: false,
                    ..
                } => match kc {
                    Keycode::A => movement.0 -= 1,
                    Keycode::D => movement.0 += 1,
                    Keycode::W => movement.1 -= 1,
                    Keycode::S => movement.1 += 1,
                    _ => (),
                },
                Ev::KeyUp {
                    keycode: Some(kc),
                    repeat: false,
                    ..
                } => match kc {
                    Keycode::A => movement.0 += 1,
                    Keycode::D => movement.0 -= 1,
                    Keycode::W => movement.1 += 1,
                    Keycode::S => movement.1 -= 1,
                    _ => (),
                },
                _ => ()
            }
        }

        send(&client, Command { x: movement.0, y: movement.1 });
        render(&state, &mut canvas);
        canvas.present();

        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }
    
    Ok(())
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

#[derive(Debug)]
struct Command {
    x: i8,
    y: i8
}

fn send(socket: &UdpSocket, moved: Command) {
    println!("sent movement: {:?}", &moved);
    let payload: [u8; 2] = unsafe { std::mem::transmute(moved) };
    socket.send(&payload).unwrap();
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
