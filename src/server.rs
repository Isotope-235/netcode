use std::{collections::HashMap, error::Error, io, net::UdpSocket, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{FRAME_TIME, Game, render, sys};

pub const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
pub const PORT: u16 = 7878;

pub fn run(mut sdl: sys::SdlContext, shared: Game) -> Result<(), Box<dyn Error>> {
    let mut state = State {
        clients: Vec::new(),
        shared,
    };

    let server = std::net::UdpSocket::bind((HOST, PORT))?;
    server.set_nonblocking(true)?;

    let ticker = sys::ticker(FRAME_TIME);

    let mut running = true;
    while running {
        let tick = ticker.start();

        running = sdl.should_run();

        let mut buf = [0; 64];
        while let Ok((read, origin)) = server.recv_from(&mut buf) {
            if !state.clients.contains(&origin) {
                state.clients.push(origin);
            }

            println!("server got data: {:?}", &buf[..read]);
        }

        render(&state.shared, &mut sdl.canvas);

        tick.wait();
    }

    Ok(())
}

struct State {
    clients: Vec<std::net::SocketAddr>,
    shared: crate::Game,
}

fn broadcast(state: &State, socket: &UdpSocket) -> io::Result<()> {
    for addr in &state.clients {
        socket.send_to(&[69], addr)?;
    }

    Ok(())
}
