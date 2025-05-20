use std::{error::Error, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{
    Command, FRAME_TIME, Game, HOST, LOGICAL_HEIGHT, LOGICAL_WIDTH, PORT, Player, SERVER_HOST,
    SERVER_PORT, Vec2, render, send, sys,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = Game::new();

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
        render(&state, &mut sdl.canvas);
        sdl.canvas.present();

        let elapsed = start.elapsed();
        let wait = FRAME_TIME.checked_sub(elapsed);
        thread::sleep(wait.unwrap_or_default());
    }

    Ok(())
}
