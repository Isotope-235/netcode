use std::{error::Error, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{
    Command, FRAME_TIME, Game, HOST, LOGICAL_HEIGHT, LOGICAL_WIDTH, PORT, Player, SERVER_HOST,
    SERVER_PORT, Vec2, render, send, sys,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: 0,
        shared: Game::new(),
    };

    let mut sdl = sys::init_sdl()?;

    let mut movement = (0, 0);
    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((SERVER_HOST, SERVER_PORT))?;

    'game: loop {
        let start = Instant::now();

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

        send(
            &client,
            Command {
                x: movement.0,
                y: movement.1,
            },
        );
        player_movement(&mut state, movement);
        render(&state.shared, &mut sdl.canvas);
        sdl.canvas.present();

        sys::tick(start, FRAME_TIME);
    }

    Ok(())
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}

fn player_movement(game: &mut State, movement: (i8, i8)) {
    const PLAYER_SPEED: f32 = 10.;
    const JUMP_VELOCITY: f32 = 20.;
    const GRAVITY: f32 = 9.81;

    game.shared.players[game.player_idx].velocity =
        Vec2::new(movement.0 as f32, movement.1 as f32).normalize() * PLAYER_SPEED;

    for player in &mut game.shared.players {
        player.pos += player.velocity;
    }
}
