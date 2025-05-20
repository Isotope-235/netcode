use std::{
    error::Error,
    net::{Ipv4Addr, UdpSocket},
    ops::{Add, AddAssign, Mul},
    time::Duration,
};

use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::{Canvas, FRect},
    video::Window,
};

mod client;
mod networking;
mod server;
mod sys;

const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 56665;
const SERVER_HOST: Ipv4Addr = HOST;
const SERVER_PORT: u16 = 7878;
const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);

fn main() -> Result<(), Box<dyn Error>> {
    client::run()
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
    y: i8,
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

impl Game {
    fn new() -> Self {
        Self {
            players: vec![Player {
                pos: Vec2 {
                    x: (LOGICAL_WIDTH / 2) as _,
                    y: (LOGICAL_HEIGHT / 2) as _,
                },
                velocity: Vec2::new(0., 0.),
                color: Color::RED,
                size: 10.0,
            }],
            platforms: Vec::new(),
        }
    }
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
    velocity: Vec2,
    size: f32,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    fn normalize(self) -> Self {
        self * (1. / self.len())
    }

    fn len(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
