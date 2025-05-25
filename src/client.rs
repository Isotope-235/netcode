use std::{
    error::Error,
    fmt::Display,
    time::{Duration, Instant},
};

use sdl2::{EventPump, keyboard::Keycode};

use crate::{model::*, networking, render, server, sys};

const FRAME_TIME: Duration = Duration::from_nanos(16_666_666);
pub const DELTA_TIME: f64 = FRAME_TIME.as_secs_f64();

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

    let mut movement: (i8, i8) = (0, 0);
    let mut movement_history: Vec<((i8, i8), usize)> = vec![(movement, 1)];

    let mut players_prev = Vec::new();
    let mut players_current = Vec::new();
    let mut server_frame_time = Duration::from_millis(100); // initial guess, this gets changed
    let mut server_timestamp = Instant::now();

    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_client_inputs(&mut sdl.events, &mut settings, &mut movement, &mut running);
        client.set_ping(settings.ping_ms);

        // Handling of movement history for reconciliation
        let movement_id = movement_history.last().unwrap_or(&((0, 0), 0)).1 + 1;
        movement_history.push((movement, movement_id));

        let message = Message {
            id: movement_id,
            x: movement.0,
            y: movement.1,
        };

        client.send(&message)?;

        let mut move_ack_id = 0;
        for bytes in client.recv() {
            let server_response: ServerResponse = serde_json::from_slice(&bytes).unwrap();
            state.player_idx = Some(server_response.player_idx);
            players_prev = players_current;
            state.shared = server_response.game;
            players_current = state.shared.players.clone();
            move_ack_id = server_response.ack_id;
            server_frame_time = server_timestamp.elapsed();
            server_timestamp = Instant::now();
        }

        if move_ack_id != 0 {
            movement_history.retain(|m| m.1 > move_ack_id);
            if settings.reconciliation {
                reconcile(&mut state, &movement_history)
            };
        }

        if settings.prediction {
            predict(&mut state, movement)
        };

        if settings.interpolation {
            let interpolation_float = (server_timestamp.elapsed().as_secs_f64()
                / server_frame_time.as_secs_f64())
            .min(1.);
            interpolate(
                &mut state,
                &players_prev,
                &players_current,
                interpolation_float,
            );
        }

        render::game(&state.shared, &mut sdl.canvas);
        render::settings(&mut sdl, font, settings.to_string().lines());
        sdl.canvas.present();

        tick.wait();
    }

    Ok(())
}

fn interpolate(
    state: &mut State,
    players_prev: &[Player],
    players_current: &[Player],
    interpolation_float: f64,
) {
    let player_idx = state.player_idx.unwrap_or(players_prev.len());
    for (i, player) in players_prev.iter().enumerate() {
        if i == player_idx {
            continue;
        }

        let pos_diff = players_current[i].pos - player.pos;
        state.shared.players[i].pos = player.pos + (pos_diff * interpolation_float);
    }
}

fn reconcile(state: &mut State, movement_history: &[((i8, i8), usize)]) {
    for movement in movement_history {
        if let Some(player_idx) = state.player_idx {
            state
                .shared
                .player_physics(player_idx, movement.0, DELTA_TIME);
        }
    }
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

fn predict(state: &mut State, movement: (i8, i8)) {
    if let Some(player_idx) = state.player_idx {
        state
            .shared
            .player_physics(player_idx, movement, DELTA_TIME);
    }
}

struct State {
    player_idx: Option<usize>,
    shared: Game,
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

    pub fn increment_ping(&mut self) {
        self.ping_ms += Self::PING_INTERVAL;
    }

    pub fn decrement_ping(&mut self) {
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
