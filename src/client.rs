use std::error::Error;

use sdl3::keyboard::KeyboardState;

use crate::{FRAME_TIME, Game, Platform, Player, Vec2, render, server, sys};

const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 0;

const PLAYER_TOP_SPEED: f32 = 100.;
const PLAYER_ACCELERATION: f32 = PLAYER_TOP_SPEED * 5.;
const GRAVITY: Vec2 = Vec2 { x: 0., y: 9.81 };
const DELTA_TIME: f32 = FRAME_TIME.as_secs_f32();

pub fn run(mut sdl: sys::SdlContext, shared: Game) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: 0,
        shared,
    };

    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((server::HOST, server::PORT))?;
    client.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut running = true;
    while running {
        let tick = ticker.start();

        running = sdl.user_has_not_quit();

        let movement = get_input(sdl.events.keyboard_state());

        let mut buf = [0; 2048];
        if let Ok(read) = client.recv(&mut buf) {
            println!("got data: {:?}", &buf[..read]);
            state.shared = serde_json::from_slice(&buf[..read]).unwrap();
        }

        send(&client, movement);

        player_input(&mut state.shared, state.player_idx, movement);
        player_movement(&mut state);
        render(&state.shared, &mut sdl.canvas);

        tick.wait();
    }

    Ok(())
}

fn get_input(keyboard: KeyboardState<'_>) -> (i8, i8) {
    use sdl3::keyboard::Scancode as Sc;

    let (left, right) = (
        keyboard.is_scancode_pressed(Sc::A),
        keyboard.is_scancode_pressed(Sc::D),
    );
    let (up, down) = (
        keyboard.is_scancode_pressed(Sc::W),
        keyboard.is_scancode_pressed(Sc::S),
    );
    let x = right as i8 - left as i8;
    let y = down as i8 - up as i8;

    (x, y)
}

fn send(socket: &std::net::UdpSocket, movement: (i8, i8)) {
    let message = crate::Message {
        x: movement.0,
        y: movement.1,
    };
    let _ = socket.send(&serde_json::to_vec(&message).unwrap());
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}

fn player_input(game: &mut Game, player_idx: usize, movement: (i8, i8)) {
    let current_velocity = game.players[player_idx].velocity;
    let target_velocity =
        Vec2::new(movement.0 as f32, movement.1 as f32).normalize() * PLAYER_TOP_SPEED;
    let velocity_diff = target_velocity - current_velocity;
    let acceleration = if PLAYER_ACCELERATION * DELTA_TIME < velocity_diff.len() {
        PLAYER_ACCELERATION * DELTA_TIME
    } else {
        velocity_diff.len()
    };

    game.players[player_idx].velocity =
        current_velocity + (velocity_diff.normalize() * acceleration);
}

fn player_movement(game: &mut State) {
    for player in &mut game.shared.players {
        player.velocity += GRAVITY;
        player.pos += player.velocity * DELTA_TIME;

        collide(player, &game.shared.platforms);
    }
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
