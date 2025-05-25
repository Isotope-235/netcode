use crate::math::Vec2;

pub const LOGICAL_WIDTH: u32 = 320;
pub const LOGICAL_HEIGHT: u32 = 240;
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Message {
    pub id: usize,
    pub x: i8,
    pub y: i8,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerResponse {
    pub ack_id: usize,
    pub player_idx: usize,
    pub game: Game,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Game {
    pub platforms: Vec<Platform>,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new() -> Self {
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

    pub fn player_physics(&mut self, player_idx: usize, movement: (i8, i8), dt: f64) {
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
pub struct Platform {
    pub size: (f64, f64),
    pub pos: Vec2,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Player {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub size: f64,
    pub state: PlayerState,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2 {
                x: HALF_WIDTH,
                y: HALF_HEIGHT,
            },
            velocity: Vec2::NULL,
            size: 10.0,
            state: PlayerState::Airborne,
        }
    }

    fn dimensions(&self) -> Vec2 {
        let rad = self.size * 0.5;
        Vec2::new(rad, rad)
    }

    fn top_left(&self) -> Vec2 {
        self.pos - self.dimensions()
    }

    fn bottom_right(&self) -> Vec2 {
        self.pos + self.dimensions()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum PlayerState {
    // Direction of hit wall
    WallBound(i8),
    Grounded,
    Airborne,
}

fn collide(player: &mut Player, platforms: &[Platform]) {
    let mut collided = true;
    player.state = PlayerState::Airborne;
    while collided {
        collided = false;
        for platform in platforms {
            let topleft = player.top_left();
            let botright = player.bottom_right();

            if platform.pos.x + platform.size.0 / 2. > topleft.x
                && platform.pos.x - platform.size.0 / 2. < botright.x
                && platform.pos.y + platform.size.1 / 2. > topleft.y
                && platform.pos.y - platform.size.1 / 2. < botright.y
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
