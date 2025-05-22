use std::{error::Error, time::Duration};

use sdl2::{EventPump, keyboard::Keycode};

use crate::{Game, networking, player_input, render, server, sys};

const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);
pub const DELTA_TIME: f32 = FRAME_TIME.as_secs_f32();

pub fn run(mut sdl: sys::SdlContext, shared: Game) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: 0,
        shared,
    };

    let mut settings = Settings {
        reconciliation: true,
        interpolation: true,
        prediction: true,
        ping: Duration::from_millis(250),
    };

    let client = networking::Client::connect((server::HOST, server::PORT), settings.ping)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut movement: (i8, i8) = (0, 0);
    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_client_inputs(&mut sdl.events, &mut settings, &mut movement, &mut running);

        let message = crate::Message {
            x: movement.0,
            y: movement.1,
        };
        client.send(&message);

        for bytes in client.recv() {
            state.shared = serde_json::from_slice(&bytes).unwrap();
        }

        predict(&mut state, movement);
        render(&state.shared, &mut sdl.canvas);

        tick.wait();
    }

    Ok(())
}

fn handle_client_inputs(
    events: &mut EventPump,
    settings: &mut Settings,
    movement: &mut (i8, i8),
    running: &mut bool,
) {
    for event in events.poll_iter() {
        use sdl2::event::Event as Ev;

        match event {
            Ev::Quit { .. } => *running = false,
            Ev::KeyDown {
                keycode: Some(kc),
                repeat: false,
                ..
            } => match kc {
                Keycode::W => movement.1 -= 1,
                Keycode::S => movement.1 += 1,
                Keycode::A => movement.0 -= 1,
                Keycode::D => movement.0 += 1,
                Keycode::I => settings.interpolation = !settings.interpolation,
                Keycode::P => settings.prediction = !settings.prediction,
                Keycode::R => settings.reconciliation = !settings.reconciliation,
                Keycode::Plus => settings.ping += Duration::from_millis(50),
                Keycode::Minus => settings.ping -= Duration::from_millis(50).min(settings.ping),
                _ => (),
            },
            Ev::KeyUp {
                keycode: Some(kc),
                repeat: false,
                ..
            } => match kc {
                Keycode::W => movement.1 += 1,
                Keycode::S => movement.1 -= 1,
                Keycode::A => movement.0 += 1,
                Keycode::D => movement.0 -= 1,
                _ => (),
            },
            _ => (),
        }
    }
}

fn predict(state: &mut State, movement: (i8, i8)) {
    state
        .shared
        .simple_player_input(state.player_idx, movement, DELTA_TIME);
    state.shared.player_movement(DELTA_TIME);
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}

struct Settings {
    reconciliation: bool,
    prediction: bool,
    interpolation: bool,
    ping: Duration,
}
