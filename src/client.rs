use std::{error::Error, fmt::Display, time::Duration};

use sdl2::{
    EventPump, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, ttf, video::Window,
};

use crate::{Game, networking, render, server, sys};

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
    let mut movement_history: Vec<((i8, i8), usize)> = vec![(movement, 1)];
    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_client_inputs(&mut sdl.events, &mut settings, &mut movement, &mut running);

        // Handling of movement history for reconciliation
        let movement_id = movement_history.last().unwrap_or(&((0, 0), 0)).1 + 1;
        movement_history.push((movement, movement_id));

        let message = crate::Message {
            id: movement_id,
            x: movement.0,
            y: movement.1,
        };

        client.send(&message)?;

        let mut move_ack_id: usize = 0;
        for bytes in client.recv() {
            (state.shared, move_ack_id) = serde_json::from_slice(&bytes).unwrap();
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
        render(&state.shared, &mut sdl.canvas);
        render_settings(&settings, &mut sdl.canvas);
        sdl.canvas.present();

        tick.wait();
    }

    Ok(())
}

fn reconcile(state: &mut State, movement_history: &Vec<((i8, i8), usize)>) {
    for movement in movement_history {
        // TODO make player_idx not hardcoded
        state
            .shared
            .simple_player_input(state.player_idx, movement.0, DELTA_TIME);
        state.shared.player_movement(DELTA_TIME);
    }
}

fn render_settings(settings: &Settings, canvas: &mut Canvas<Window>) {
    let ttf_context = ttf::init().unwrap();
    let font = ttf_context
        .load_font("assets/MinecraftRegular-Bmg3.otf", 10)
        .unwrap();

    for (i, text_chunk) in format!("{settings}").split("\n").enumerate() {
        let surface = font.render(text_chunk).blended(Color::BLACK).unwrap();

        // Create texture from surface
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        // Get text dimensions
        let sdl2::render::TextureQuery { width, height, .. } = texture.query();

        // Render the texture
        let target = Rect::new(4, 4 + (height * i as u32) as i32, width, height);
        canvas.copy(&texture, None, Some(target)).unwrap();
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

#[derive(Debug)]
struct Settings {
    reconciliation: bool,
    prediction: bool,
    interpolation: bool,
    ping: Duration,
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Reconciliation: {}\nInterpolation: {}\nPrediction: {}\nPing: {:?}",
            self.reconciliation, self.interpolation, self.prediction, self.ping
        )
    }
}
