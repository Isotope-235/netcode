use std::error::Error;

use math::Vec2;

mod client;
mod math;
mod networking;
mod render;
mod server;
mod sys;

const LOGICAL_WIDTH: u32 = 320;
const LOGICAL_HEIGHT: u32 = 240;
const WIDTH: f64 = LOGICAL_WIDTH as _;
const HEIGHT: f64 = LOGICAL_HEIGHT as _;
const HALF_WIDTH: f64 = WIDTH / 2.;
const HALF_HEIGHT: f64 = HEIGHT / 2.;

const PLAYER_TOP_SPEED: f64 = 100.;
const PLAYER_ACCELERATION: f64 = PLAYER_TOP_SPEED * 10.;
const JUMP_SPEED: f64 = 100.;
const GRAVITY: Vec2 = Vec2 {
    x: 0.,
    y: 9.81 * 20.,
};

const FONT_PATH: &str = "assets/minecraft.otf";
const FONT_SIZE: u16 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    let mode = args.nth(1).unwrap_or_default();

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let ctx = sys::init_sdl_systems(&sdl, &video)?;
    let ttf = sdl2::ttf::init()?;
    let font = ttf.load_font(FONT_PATH, FONT_SIZE)?;

    let shared_state = Game::new();

    #[allow(clippy::wildcard_in_or_patterns)]
    match &mode[..] {
        "server" | "--server" => server::run(ctx, &font, shared_state),
        "client" | "--client" | _ => client::run(ctx, &font, shared_state),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Message {
    id: usize,
    x: i8,
    y: i8,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ServerResponse {
    ack_id: usize,
    player_idx: usize,
    game: Game,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Game {
    platforms: Vec<Platform>,
    players: Vec<Player>,
}

impl Game {
    fn new() -> Self {
        Self {
            players: Vec::new(),
            platforms: vec![
                Platform {
                    size: (120., 30.),
                    pos: Vec2::new(HALF_WIDTH, 50. + HALF_HEIGHT),
                },
                Platform {
                    size: (WIDTH, 30.),
                    pos: Vec2::new(HALF_WIDTH, HEIGHT),
                },
                Platform {
                    size: (30., HEIGHT),
                    pos: Vec2::new(0., HALF_HEIGHT),
                },
                Platform {
                    size: (30., HEIGHT),
                    pos: Vec2::new(WIDTH, HALF_HEIGHT),
                },
            ],
        }
    }

    fn player_physics(&mut self, player_idx: usize, movement: (i8, i8), dt: f64) {
        let player = &mut self.players[player_idx];
        let current_velocity = player.velocity.x;
        let target_velocity = movement.0 as f64 * PLAYER_TOP_SPEED;
        let velocity_diff = target_velocity - current_velocity;

        let acc = match player.state {
            PlayerState::Grounded => {
                let direction = if velocity_diff < 0. { -1. } else { 1. };
                PLAYER_ACCELERATION * direction * dt / 3.
            }
            _ => PLAYER_ACCELERATION * movement.0 as f64 * dt,
        };

        let delta_v = if acc.abs() < velocity_diff.abs() {
            acc
        } else {
            velocity_diff
        };

        player.velocity.x = current_velocity + delta_v;

        if movement.1 == -1 {
            match player.state {
                PlayerState::Grounded => {
                    player.velocity.y -= JUMP_SPEED;
                }
                PlayerState::WallBound(direction) => {
                    player.velocity.y = -JUMP_SPEED;
                    player.velocity.x -= direction as f64 * JUMP_SPEED * 1.5;
                }
                PlayerState::Airborne => (),
            }
        }

        player.velocity += GRAVITY * dt;
        player.pos += player.velocity * dt - GRAVITY * dt.powi(2) * 0.5;

        collide(player, &self.platforms);
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Platform {
    size: (f64, f64),
    pos: Vec2,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Player {
    pos: Vec2,
    velocity: Vec2,
    size: f64,
    state: PlayerState,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
enum PlayerState {
    // Direction of hit wall
    WallBound(i8),
    Grounded,
    Airborne,
}

fn collide(player: &mut Player, platforms: &Vec<Platform>) {
    let mut collided = true;
    player.state = PlayerState::Airborne;
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
    player.pos = if player.pos.dist(y_corrected) < player.pos.dist(x_corrected) {
        if player_relative_posistion.y < 0. {
            player.state = PlayerState::Grounded
        }
        player.velocity.y = 0.;
        y_corrected
    } else {
        let wall_direction = if player_relative_posistion.x < 0. {
            1
        } else {
            -1
        };
        player.state = PlayerState::WallBound(wall_direction);
        player.velocity.x = 0.;
        x_corrected
    };
}
