use std::{error::Error, time::Duration};

use sdl3::keyboard::KeyboardState;

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

    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((server::HOST, server::PORT))?;
    client.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut running = true;
    while running {
        let tick = ticker.start();

        running = sdl.user_has_not_quit();

        let movement = get_input(sdl.events.keyboard_state());
        send(&client, movement);

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

fn predict(state: &mut State, movement: (i8, i8)) {
    simple_player_input(&mut state.shared, state.player_idx, movement, DELTA_TIME);
    player_movement(&mut state.shared, DELTA_TIME);
}

fn get_input(keyboard: KeyboardState<'_>) -> (i8, i8) {
    use sdl3::keyboard::Scancode as Sc;

    let (left, right) = (
        keyboard.is_scancode_pressed(Sc::A),
        keyboard.is_scancode_pressed(Sc::D),
    );
    let (up, down) = (
        keyboard.is_scancode_pressed(Sc::W),
        keyboard.is_scancode_pressed(Sc::S),
    );
    let x = right as i8 - left as i8;
    let y = down as i8 - up as i8;

    (x, y)
}

fn send(socket: &std::net::UdpSocket, movement: (i8, i8)) {
    let message = crate::Message {
        x: movement.0,
        y: movement.1,
    };
    let _ = socket.send(&serde_json::to_vec(&message).unwrap());
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}
