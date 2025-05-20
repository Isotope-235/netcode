use std::{error::Error, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{
    Command, FRAME_TIME, Game, HOST, LOGICAL_HEIGHT, LOGICAL_WIDTH, PORT, Player, SERVER_HOST,
    SERVER_PORT, Vec2, player_movement, render, send, sys,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = Game {
        player_idx: Some(0),
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
        render(&state, &mut sdl.canvas);
        sdl.canvas.present();

        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }

    Ok(())
}
