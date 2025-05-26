//! Implementations of netcode features.

use std::time::{Duration, Instant};

use crate::{client, model::*};

/// Initialize the `Netcode`, which implements netcode features.
pub fn init() -> Netcode {
    Netcode {
        movement_history: Vec::new(),
        players_prev: Vec::new(),
        players_current: Vec::new(),
        server_tick_time: Duration::from_millis(100), // initial guess, this gets changed
        server_timestamp: Instant::now(),
    }
}

/// Keeps track of state required to implement netcode features.
///
/// Use `apply` to apply prediction, reconciliation and interpolation.
/// Use `push_movement` and `update` to update the state each frame.
pub struct Netcode {
    movement_history: Vec<Movement>,
    players_prev: Vec<Player>,
    players_current: Vec<Player>,
    server_tick_time: Duration,
    server_timestamp: Instant,
}

impl Netcode {
    /// Adds another movement to the movement history, which is needed for reconciliation.
    ///
    /// Returns the ID/sequence number of the movement, used for acknowledgment.
    pub fn push_movement(&mut self, movement: (i8, i8)) -> usize {
        let id = self.movement_history.last().map(|l| l.id + 1).unwrap_or(1);
        self.movement_history.push(Movement { id, dir: movement });
        id
    }

    /// Update the current player state, which is used for interpolation.
    pub fn update(&mut self, players_current: Vec<Player>) {
        std::mem::swap(&mut self.players_prev, &mut self.players_current);
        self.server_tick_time = self.server_timestamp.elapsed();
        self.server_timestamp = Instant::now();
        self.players_current = players_current;
    }

    /// Apply netcode features on the client side. Use the boolean arguments to specify which features to enable.
    ///
    /// `move_ack_id` is the id of the last movement acknowledged by the server.
    pub fn apply(
        &mut self,
        state: &mut client::State,
        move_ack_id: usize,
        movement: (i8, i8),
        prediction: bool,
        reconciliation: bool,
        interpolation: bool,
    ) {
        if move_ack_id != 0 {
            self.movement_history.retain(|m| m.id > move_ack_id);
            if reconciliation {
                reconcile(state, &self.movement_history)
            };
        }

        if prediction {
            predict(state, movement)
        };

        if interpolation {
            let elapsed = self.server_timestamp.elapsed().as_secs_f64();
            let tick_time = self.server_tick_time.as_secs_f64();
            let factor = (elapsed / tick_time).min(1.);
            interpolate(state, &self.players_prev, &self.players_current, factor);
        }
    }
}

fn predict(state: &mut client::State, movement: (i8, i8)) {
    if let Some(player_idx) = state.player_idx {
        state
            .shared
            .player_physics(player_idx, movement, client::DELTA_TIME);
    }
}

fn reconcile(state: &mut client::State, movement_history: &[Movement]) {
    for movement in movement_history {
        if let Some(player_idx) = state.player_idx {
            state
                .shared
                .player_physics(player_idx, movement.dir, client::DELTA_TIME);
        }
    }
}

fn interpolate(
    state: &mut client::State,
    players_prev: &[Player],
    players_current: &[Player],
    interpolation_factor: f64,
) {
    let player_idx = state.player_idx.unwrap_or(players_prev.len());
    for (i, player) in players_prev.iter().enumerate() {
        if i == player_idx {
            continue;
        }

        let pos_diff = players_current[i].pos - player.pos;
        state.shared.players[i].pos = player.pos + (pos_diff * interpolation_factor);
    }
}
