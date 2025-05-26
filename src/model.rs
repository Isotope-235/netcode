//! Items implementing game logic and communication between server and client.

use crate::math::Vec2;

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

/// One movement input.
///
/// Contains an id/sequence number, which is used to implement reconciliation.
pub struct Movement {
    /// The sequence number.
    pub id: usize,
    /// The directionality of the movement input.
    pub dir: (i8, i8),
}

/// A message DTO, sent from the client to the server.
///
/// Contains information about the client's inputs.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Message {
    /// The sequence number.
    pub id: usize,
    /// The x value of the directionality of the movement input.
    pub x: i8,
    /// The y value of the directionality of the movement input.
    pub y: i8,
}

/// A server response DTO, sent from the server to the client each tick.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerResponse {
    /// The last client message that was acknowledged before the server sent this response.
    pub ack_id: usize,
    /// The player ID of the client receiving the message.
    pub player_idx: usize,
    /// The current game state on the server, as of this response being sent.
    pub game: Game,
}

/// A game state. Includes the level layout and the current player data.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Game {
    /// All platforms composing the level layout.
    pub platforms: Vec<Platform>,
    /// The states of the players. An player ID is an index into this vector.
    pub players: Vec<Player>,
}

impl Game {
    /// Initialize the game state, including the arrangement of the platforms.
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
                    pos: Vec2::new(HALF_WIDTH, -15.),
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

    /// Apply physics calculations to players, using the time delta specified.
    /// Physics are applied per new movement, even if that movement is (0, 0).
    pub fn player_physics(&mut self, player_idx: usize, movement: (i8, i8), dt: f64) {
        let player = &mut self.players[player_idx];
        let current_velocity = player.velocity.x;
        let target_velocity = movement.0 as f64 * PLAYER_TOP_SPEED;
        let velocity_diff = target_velocity - current_velocity;

        let acc = match player.state {
            PlayerState::Grounded => {
                let direction = if velocity_diff < 0. { -1. } else { 1. };
                PLAYER_ACCELERATION * direction * dt
            }
            _ => PLAYER_ACCELERATION * movement.0 as f64 * dt,
        };

        let delta_v = if acc.abs() < velocity_diff.abs() {
            (acc * velocity_diff).signum() * acc
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

/// A rectangle-shaped platform, which has collision with players.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Platform {
    /// The (x, y) width and height.
    pub size: (f64, f64),
    /// The position of the platform, from the middle.
    pub pos: Vec2,
}

impl Platform {
    fn bounds(&self) -> (Vec2, Vec2) {
        let dims = Vec2::new(self.size.0 * 0.50, self.size.1 * 0.50);
        (self.pos - dims, self.pos + dims)
    }
}

/// A player's current state, including position and velocity.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Player {
    /// The current position (center) of the player.
    pub pos: Vec2,
    /// The current velocity of the player.
    pub velocity: Vec2,
    /// The size of the player (width and height).
    pub size: f64,
    /// The current airtime state of the player,
    /// which tells the physics whether to apply friction,
    /// whether the player is allowed to jump, and whether the player can wall-jump.
    pub state: PlayerState,
}

impl Player {
    /// Create a new player with default settings.
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

    fn radius(&self) -> f64 {
        self.size * 0.50
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        let rad = self.radius();
        let dims = Vec2::new(rad, rad);
        (self.pos - dims, self.pos + dims)
    }
}

/// A type used to tell the physics what effect an attempt to jump will have.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum PlayerState {
    /// The player is on a wall. The number is the direction of the hit wall. The player can wall-jump.
    WallBound(i8),
    /// The player is on the grounds, and is therefore allowed to jump.
    Grounded,
    /// The player is airborne. They cannot jump.
    Airborne,
}

fn collide(player: &mut Player, platforms: &[Platform]) {
    let mut collided = true;
    player.state = PlayerState::Airborne;
    while collided {
        collided = false;
        for platform in platforms {
            let (topleft, botright) = player.bounds();
            let (platform_topleft, platform_botright) = platform.bounds();

            if platform_botright.x > topleft.x
                && platform_topleft.x < botright.x
                && platform_botright.y > topleft.y
                && platform_topleft.y < botright.y
            {
                collided = true;
                fix_position(player, platform);
            }
        }
    }
}

fn fix_position(player: &mut Player, platform: &Platform) {
    let player_relative_position = player.pos - platform.pos;

    let rad = player.radius();

    // These corrected positions are the possible positions to push the
    // player out of the platform they are currently colliding with
    let x_direction = player_relative_position.x.signum();
    let x_corrected = Vec2::new(
        platform.pos.x + x_direction * (platform.size.0 / 2. + rad),
        platform.pos.y + player_relative_position.y,
    );

    let y_direction = player_relative_position.y.signum();
    let y_corrected = Vec2::new(
        platform.pos.x + player_relative_position.x,
        platform.pos.y + y_direction * (platform.size.1 / 2. + rad),
    );

    // Check which position is closest to the players actual location and use that one
    player.pos = if player.pos.dist(y_corrected) < player.pos.dist(x_corrected) {
        if player_relative_position.y < 0. {
            player.state = PlayerState::Grounded
        }
        player.velocity.y = 0.;
        y_corrected
    } else {
        let wall_direction = if player_relative_position.x < 0. {
            1
        } else {
            -1
        };
        player.state = PlayerState::WallBound(wall_direction);
        player.velocity.x = 0.;
        x_corrected
    };
}
