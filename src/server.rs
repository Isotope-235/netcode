use std::{error::Error, io, net::UdpSocket, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{FRAME_TIME, Game, render, sys};

pub const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
pub const PORT: u16 = 7878;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = State {
        clients: Vec::new(),
        shared: Game::new(),
    };

    let mut sdl = sys::init_sdl()?;

    let mut movement = (0, 0);
    let server = std::net::UdpSocket::bind((HOST, PORT))?;

    'game: loop {
        let start = Instant::now();

        for event in sdl.events.poll_iter() {
            use sdl3::event::Event as Ev;

            match event {
                Ev::Quit { .. } => break 'game,
                _ => (),
            }
        }

        render(&state.shared, &mut sdl.canvas);
        sdl.canvas.present();

        sys::tick(start, FRAME_TIME)
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
