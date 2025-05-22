use std::{error::Error, io, net::UdpSocket, time::Duration};

use sdl2::EventPump;

use crate::{Game, render, sys};

pub const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
pub const PORT: u16 = 7878;

const FRAME_TIME: Duration = Duration::from_millis(200);
const DELTA_TIME: f32 = FRAME_TIME.as_secs_f32();

pub fn run(mut sdl: sys::SdlContext, shared: Game) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        last_acc: vec![0],
        clients: Vec::new(),
        shared,
    };

    let server = std::net::UdpSocket::bind((HOST, PORT))?;
    server.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut running = true;
    while running {
        let tick = ticker.start();

        handle_server_inputs(&mut sdl.events, &mut running);

        let mut buf = [0; 64];
        while let Ok((read, origin)) = server.recv_from(&mut buf) {
            if !state.clients.contains(&origin) {
                state.clients.push(origin);
            }

            let message = serde_json::from_slice::<crate::Message>(&buf[..read]).unwrap();
            let movement = (message.x, message.y);

            // TODO make player_idx not hardcoded
            state.last_acc[0] = message.id;
            state
                .shared
                .simple_player_input(0, movement, crate::client::DELTA_TIME);
        }

        state.shared.player_movement(DELTA_TIME);
        broadcast(&state, &server)?;
        render(&state.shared, &mut sdl.canvas);
        sdl.canvas.present();

        tick.wait();
    }

    Ok(())
}

struct State {
    last_acc: Vec<usize>,
    clients: Vec<std::net::SocketAddr>,
    shared: crate::Game,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AckMessage {
    last_acc: usize,
    game: crate::Game,
}

#[allow(dead_code)]
fn broadcast(state: &State, socket: &UdpSocket) -> io::Result<()> {
    let serialized_state = serde_json::to_vec(&(&state.shared, state.last_acc[0])).unwrap();
    for addr in &state.clients {
        if let Err(e) = socket.send_to(&serialized_state, addr) {
            println!("{}", e);
        };
    }

    Ok(())
}

fn handle_server_inputs(events: &mut EventPump, running: &mut bool) {
    for event in events.poll_iter() {
        use sdl2::event::Event as Ev;

        if let Ev::Quit { .. } = event {
            *running = false
        }
    }
}
