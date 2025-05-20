use std::{error::Error, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{Command, FRAME_TIME, Game, Platform, Player, Vec2, render, send, server, sys};

const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 0;

const PLAYER_SPEED: f32 = 35.;
const JUMP_VELOCITY: f32 = 20.;
const GRAVITY: Vec2 = Vec2 { x: 0., y: 9.81 };
const DELTA_TIME: f32 = FRAME_TIME.as_secs_f32();

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: 0,
        shared: Game::new(),
    };

    let mut sdl = sys::init_sdl()?;

    let mut movement = (0, 0);
    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((server::HOST, server::PORT))?;
    client.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    'game: loop {
        let tick = ticker.start();

        for event in sdl.events.poll_iter() {
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
                _ => (),
            }
        }

        let mut buf = [0; 64];
        if let Ok(read) = client.recv(&mut buf) {
            println!("got data: {:?}", &buf[..read]);
        }

        send(
            &client,
            Command {
                x: movement.0,
                y: movement.1,
            },
        );

        player_input(&mut state.shared, state.player_idx, movement);
        player_movement(&mut state);
        render(&state.shared, &mut sdl.canvas);
        sdl.canvas.present();

        tick.wait();
    }

    Ok(())
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}

fn player_input(game: &mut Game, player_idx: usize, movement: (i8, i8)) {
    game.players[player_idx].velocity =
        Vec2::new(movement.0 as f32, movement.1 as f32).normalize() * (PLAYER_SPEED * DELTA_TIME);
}

fn player_movement(game: &mut State) {
    for player in &mut game.shared.players {
        player.velocity += GRAVITY * DELTA_TIME;
        player.pos += player.velocity;

        dbg!(player.pos);
    }
}

fn collide(player: &mut Player, platforms: &Vec<Platform>) {
    let mut collided = true;
    while !collided {
        collided = false;
        for platform in platforms {
            if platform.pos.x + platform.size.0 + (player.size / 2.) > player.pos.x
                && platform.pos.x - platform.size.0 - (player.size / 2.) < player.pos.x
                && platform.pos.y + platform.size.1 + (player.size / 2.) > player.pos.y
                && platform.pos.y - platform.size.1 - (player.size / 2.) < player.pos.y
            {
                collided = true;
                fix_position(player, platform);
            }
        }
    }
}

fn fix_position(player: &mut Player, platform: &Platform) {
    let player_relative_posistion = platform.pos - player.pos;

    // These corrected positions are the possible positions to push the
    // player out of the platform they are currently colliding with
    let x_direction = if player_relative_posistion.x < 0. {
        -1.
    } else {
        1.
    };
    let x_corrected = Vec2::new(
        platform.pos.x + x_direction * (platform.size.0 + (player.size / 2.)),
        platform.pos.y + player_relative_posistion.y,
    );

    let y_direction = if player_relative_posistion.y < 0. {
        -1.
    } else {
        1.
    };
    let y_corrected = Vec2::new(
        platform.pos.x + player_relative_posistion.x,
        platform.pos.y + x_direction * (platform.size.0 + (player.size / 2.)),
    );

    // Check which position is closest to the players actual location and use that one
}
