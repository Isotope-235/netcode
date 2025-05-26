//! The client game loop.

use std::{error::Error, fmt::Display, time::Duration};

use sdl2::{EventPump, keyboard::Keycode};

use crate::{model::*, netcode, networking, render, server, sys};

const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);

/// Client-observed time delta.
pub const DELTA_TIME: f64 = FRAME_TIME.as_secs_f64();

/// Run the client.
pub fn run(
    mut sdl: sys::SdlContext,
    font: &sdl2::ttf::Font,
    shared: Game,
) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: None,
        shared,
    };

    let mut settings = Settings {
        reconciliation: false,
        interpolation: false,
        prediction: false,
        ping_ms: 250,
    };

    let client = networking::Client::connect((server::HOST, server::PORT), settings.ping_ms)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut movement = (0, 0);

    let mut netcode = netcode::init();

    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_client_inputs(&mut sdl.events, &mut settings, &mut movement, &mut running);
        client.set_ping(settings.ping_ms);

        // Handling of movement history for reconciliation
        let id = netcode.push_movement(movement);

        let message = Message {
            id,
            x: movement.0,
            y: movement.1,
        };

        client.send(&message)?;

        let mut move_ack_id = 0;
        for bytes in client.recv() {
            let server_response: ServerResponse = serde_json::from_slice(&bytes).unwrap();
            state.player_idx = Some(server_response.player_idx);
            state.shared = server_response.game;
            move_ack_id = server_response.ack_id;
            netcode.update(state.shared.players.clone());
        }

        // apply the enabled netcode features
        netcode.apply(
            &mut state,
            move_ack_id,
            movement,
            settings.prediction,
            settings.reconciliation,
            settings.interpolation,
        );

        render::game(&state.shared, &mut sdl.canvas);
        render::settings(&mut sdl, font, settings.to_string().lines());
        sdl.canvas.present();

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
                Keycode::Plus => settings.increment_ping(),
                Keycode::Minus => settings.decrement_ping(),
                _ => (),
            },
            Ev::KeyDown {
                keycode: Some(kc),
                repeat: true,
                ..
            } => match kc {
                Keycode::Plus => settings.increment_ping(),
                Keycode::Minus => settings.decrement_ping(),
                _ => {}
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

/// Client-observed game state.
///
/// Keeps track of which player the client is, and contains their local version of the game state.
pub struct State {
    /// The index of the local player.
    /// Used to index the player array in the shared game state.
    pub player_idx: Option<usize>,
    /// Shared game state, i.e. a struct shared by the client and the server, representing the whole game.
    pub shared: Game,
}

#[derive(Debug)]
struct Settings {
    reconciliation: bool,
    prediction: bool,
    interpolation: bool,
    ping_ms: u64,
}

impl Settings {
    const PING_INTERVAL: u64 = 50;

    fn increment_ping(&mut self) {
        self.ping_ms += Self::PING_INTERVAL;
    }

    fn decrement_ping(&mut self) {
        self.ping_ms -= Self::PING_INTERVAL.min(self.ping_ms);
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Reconciliation: {}\nInterpolation: {}\nPrediction: {}\nPing: {:?}",
            self.reconciliation,
            self.interpolation,
            self.prediction,
            Duration::from_millis(self.ping_ms)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_increment() {
        let mut settings = Settings {
            reconciliation: false,
            prediction: false,
            interpolation: false,
            ping_ms: 250,
        };
        settings.increment_ping();
        assert_eq!(settings.ping_ms, 300);
    }

    #[test]
    fn ping_decrement() {
        let mut settings = Settings {
            reconciliation: false,
            prediction: false,
            interpolation: false,
            ping_ms: 250,
        };
        settings.decrement_ping();
        assert_eq!(settings.ping_ms, 200);
    }

    #[test]
    fn ping_decrement_zero() {
        let mut settings = Settings {
            reconciliation: false,
            prediction: false,
            interpolation: false,
            ping_ms: 0,
        };
        settings.decrement_ping();
        assert_eq!(settings.ping_ms, 0);
    }
}
