use std::{error::Error, io, net::UdpSocket, time::Duration};

use sdl2::EventPump;

use crate::{model::*, render, sys};

pub const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
pub const PORT: u16 = 7878;

const DEFAULT_TICKRATE: usize = 4;
const FRAME_TIME: Duration = Duration::from_millis(200);

pub fn run(
    mut sdl: sys::SdlContext,
    font: &sdl2::ttf::Font,
    shared: Game,
) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        last_ack: Vec::new(),
        clients: Vec::new(),
        shared,
    };

    let mut tickrate = DEFAULT_TICKRATE;

    let server = std::net::UdpSocket::bind((HOST, PORT))?;
    server.set_nonblocking(true)?;

    let mut ticker = sys::ticker(FRAME_TIME);

    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_server_inputs(&mut sdl.events, &mut running, &mut tickrate);
        let frame_time = Duration::from_secs_f64((tickrate as f64).recip());
        ticker = sys::ticker(frame_time);

        let mut buf = [0; 64];
        while let Ok((read, origin)) = server.recv_from(&mut buf) {
            let mut player_idx = state.clients.len();
            for (i, client) in state.clients.iter().enumerate() {
                if *client == origin {
                    player_idx = i;
                }
            }

            if player_idx >= state.clients.len() {
                state.clients.push(origin);
                state.last_ack.push(0);
                state.shared.players.push(Player::new());
            }

            let message: Message = serde_json::from_slice(&buf[..read]).unwrap();
            let movement = (message.x, message.y);

            state.last_ack[player_idx] = message.id;
            state
                .shared
                .player_physics(player_idx, movement, crate::client::DELTA_TIME);
        }

        broadcast(&state, &server)?;
        render::game(&state.shared, &mut sdl.canvas);
        let text = format!("Server ticks per second: {}", tickrate);
        render::settings(&mut sdl, font, Some(text.as_str()));
        sdl.canvas.present();

        tick.wait();
    }

    Ok(())
}

struct State {
    last_ack: Vec<usize>,
    clients: Vec<std::net::SocketAddr>,
    shared: Game,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AckMessage {
    last_ack: usize,
    game: Game,
}

#[allow(dead_code)]
fn broadcast(state: &State, socket: &UdpSocket) -> io::Result<()> {
    for (i, addr) in state.clients.iter().enumerate() {
        let response = ServerResponse {
            game: state.shared.clone(),
            ack_id: state.last_ack[i],
            player_idx: i,
        };
        let serialized_state = serde_json::to_vec(&response).unwrap();
        if let Err(e) = socket.send_to(&serialized_state, addr) {
            println!("{e}");
        };
    }

    Ok(())
}

fn handle_server_inputs(events: &mut EventPump, running: &mut bool, tickrate: &mut usize) {
    for event in events.poll_iter() {
        use sdl2::{event::Event as Ev, keyboard::Keycode as Kc};

        match event {
            Ev::Quit { .. } => *running = false,
            Ev::KeyDown {
                keycode: Some(kc), ..
            } => match kc {
                Kc::Plus => *tickrate += 1,
                Kc::Minus => *tickrate = (*tickrate - 1).max(1),
                _ => (),
            },
            _ => (),
        }
    }
}
