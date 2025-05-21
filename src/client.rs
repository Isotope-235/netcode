use std::{error::Error, time::Duration};

use sdl3::{
    EventPump,
    keyboard::{KeyboardState, Keycode, Scancode as Sc},
    sys::events,
};

use crate::{Game, player_input, player_movement, render, server, simple_player_input, sys};

const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 0;

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
        ping: Duration::from_millis(100),
    };

    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((server::HOST, server::PORT))?;
    client.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut movement: (i8, i8) = (0, 0);
    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_client_inputs(&mut sdl.events, &mut settings, &mut movement, &mut running);

        send(client.try_clone()?, movement);

        let mut buf = [0; 2048];
        while let Ok(read) = client.recv(&mut buf) {
            println!("got data: {:?}", &buf[..read]);
            state.shared = serde_json::from_slice(&buf[..read]).unwrap();
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
        use sdl3::event::Event as Ev;

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
                Keycode::Minus => {
                    settings.ping -= Duration::from_millis(50).min(Duration::from_millis(0))
                }
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
    simple_player_input(&mut state.shared, state.player_idx, movement, DELTA_TIME);
    player_movement(&mut state.shared, DELTA_TIME);
}

fn send(socket: std::net::UdpSocket, movement: (i8, i8)) {
    let message = crate::Message {
        x: movement.0,
        y: movement.1,
    };
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        let _ = socket.send(&serde_json::to_vec(&message).unwrap());
    });
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
