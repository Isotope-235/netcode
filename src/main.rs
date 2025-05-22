use std::{error::Error};

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use math::Vec2;

mod client;
mod math;
mod networking;
mod server;
mod sys;

const LOGICAL_WIDTH: u32 = 320;
const LOGICAL_HEIGHT: u32 = 240;

const PLAYER_TOP_SPEED: f32 = 100.;
const PLAYER_ACCELERATION: f32 = PLAYER_TOP_SPEED * 5.;
const GRAVITY: Vec2 = Vec2 { x: 0., y: 9.81 };

const PLAYER_COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::MAGENTA, Color::GREEN, Color::YELLOW, Color::CYAN];

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

    for (i, player) in game.players.iter().enumerate() {
        if i >= PLAYER_COLORS.len() { panic!("Not enough colors :("); }
        canvas.set_draw_color(PLAYER_COLORS[i]);
        let r = Rect::from_center(
            Point::new(player.pos.x as _, player.pos.y as _),
            player.size as _,
            player.size as _,
        );
        let _ = canvas.fill_rect(r);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Message {
    id: usize,
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
            players: vec![],
            platforms: vec![Platform {
                size: (60., 20.),
                pos: Vec2::new((LOGICAL_WIDTH / 2) as _, (LOGICAL_HEIGHT / 2 + 30) as _),
            }],
        }
    }

    fn player_movement(&mut self, dt: f32) {
        for player in &mut self.players {
            // player.velocity += GRAVITY * dt;
            // player.pos += player.velocity * dt;

            collide(player, &self.platforms);
        }
    }

    fn simple_player_input(&mut self, player_idx: usize, movement: (i8, i8), dt: f32) {
        let target_velocity =
            Vec2::new(movement.0 as _, movement.1 as _).normalize() * PLAYER_TOP_SPEED;
        self.players[player_idx].pos += target_velocity * dt;
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

fn player_input(game: &mut Game, player_idx: usize, movement: (i8, i8), dt: f32) {
    let current_velocity = game.players[player_idx].velocity;
    let target_velocity =
        Vec2::new(movement.0 as f32, movement.1 as f32).normalize() * PLAYER_TOP_SPEED;
    let velocity_diff = target_velocity - current_velocity;
    let delta_v = (PLAYER_ACCELERATION * dt).min(velocity_diff.len());

    game.players[player_idx].velocity = current_velocity + (velocity_diff.normalize() * delta_v);
}

fn collide(player: &mut Player, platforms: &Vec<Platform>) {
    let mut collided = true;
    while collided {
        collided = false;
        for platform in platforms {
            if platform.pos.x + platform.size.0 / 2. > player.pos.x - (player.size / 2.)
                && platform.pos.x - platform.size.0 / 2. < player.pos.x + (player.size / 2.)
                && platform.pos.y + platform.size.1 / 2. > player.pos.y - (player.size / 2.)
                && platform.pos.y - platform.size.1 / 2. < player.pos.y + (player.size / 2.)
            {
                collided = true;
                fix_position(player, platform);
            }
        }
    }
}

fn fix_position(player: &mut Player, platform: &Platform) {
    let player_relative_posistion = player.pos - platform.pos;

    // These corrected positions are the possible positions to push the
    // player out of the platform they are currently colliding with
    let x_direction = if player_relative_posistion.x < 0. {
        -1.
    } else {
        1.
    };
    let x_corrected = Vec2::new(
        platform.pos.x + x_direction * (platform.size.0 / 2. + (player.size / 2.)),
        platform.pos.y + player_relative_posistion.y,
    );

    let y_direction = if player_relative_posistion.y < 0. {
        -1.
    } else {
        1.
    };
    let y_corrected = Vec2::new(
        platform.pos.x + player_relative_posistion.x,
        platform.pos.y + y_direction * (platform.size.1 / 2. + (player.size / 2.)),
    );

    // Check which position is closest to the players actual location and use that one
    player.pos = if (player.pos - y_corrected).len() < (player.pos - x_corrected).len() {
        player.velocity.y = 0.;
        y_corrected
    } else {
        player.velocity.x = 0.;
        x_corrected
    };
}
