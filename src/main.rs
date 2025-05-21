use std::{
    error::Error,
    net::UdpSocket,
    ops::{Add, AddAssign, Mul, Sub},
    time::Duration,
};

use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

mod client;
mod networking;
mod server;
mod sys;

const LOGICAL_WIDTH: u32 = 160;
const LOGICAL_HEIGHT: u32 = 120;
const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    let mode = args.nth(1).unwrap_or_default();

    dbg!(&mode);

    let sdl = sys::init_sdl()?;
    let shared_state = Game::new();

    #[allow(clippy::wildcard_in_or_patterns)]
    match &mode[..] {
        "server" | "--server" => server::run(sdl, shared_state),
        "client" | "--client" | _ => client::run(sdl, shared_state),
    }
}

fn render(game: &Game, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();

    canvas.set_draw_color(Color::BLACK);
    for platform in &game.platforms {
        let r = Rect::from_center(
            Point::new(platform.pos.x as _, platform.pos.y as _),
            platform.size.0 as _,
            platform.size.1 as _,
        );
        let _ = canvas.fill_rect(r);
    }

    for player in &game.players {
        canvas.set_draw_color(Color::RED);
        let r = Rect::from_center(
            Point::new(player.pos.x as _, player.pos.y as _),
            player.size as _,
            player.size as _,
        );
        let _ = canvas.fill_rect(r);
    }

    canvas.present();
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Message {
    x: i8,
    y: i8,
}

#[derive(serde::Serialize, serde::Deserialize)]
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
                size: 10.0,
            }],
            platforms: vec![Platform {
                size: (60., 20.),
                pos: Vec2::new((LOGICAL_WIDTH / 2) as _, (LOGICAL_HEIGHT / 2 + 30) as _),
            }],
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Platform {
    size: (f32, f32),
    pos: Vec2,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Player {
    pos: Vec2,
    velocity: Vec2,
    size: f32,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
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

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
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

impl Mul for Vec2 {
    type Output = f32;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    fn normalize(self) -> Self {
        let len = self.len();
        if len.abs() > 1e-10 {
            self * (1. / len)
        } else {
            self
        }
    }

    fn len(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
