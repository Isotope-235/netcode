use std::{error::Error, thread, time::Instant};

use sdl3::{keyboard::Keycode, pixels::Color};

use crate::{Command, FRAME_TIME, Game, Vec2, render, send, server, sys};

const HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 0;
use crate::{
    Command, FRAME_TIME, Game, Platform, HOST, LOGICAL_HEIGHT, LOGICAL_WIDTH, PORT, Player, SERVER_HOST,
    SERVER_PORT, Vec2, render, send, sys,
};

const PLAYER_SPEED: f32 = 35.;
const JUMP_VELOCITY: f32 = 20.;
const GRAVITY: Vec2 = Vec2 { x: 0., y: 9.81 };
const DELTA_TIME: f32 = FRAME_TIME.as_secs_f32();

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut state = State {
        player_idx: 0,
        shared: Game::new(),
    };

    let mut sdl = sys::init_sdl()?;

    let mut movement = (0, 0);
    let client = std::net::UdpSocket::bind((HOST, PORT))?;
    client.connect((server::HOST, server::PORT))?;
    client.set_nonblocking(true)?;

    'game: loop {
        let start = Instant::now();

        for event in sdl.events.poll_iter() {
            use sdl3::event::Event as Ev;

            match event {
                Ev::Quit { .. } => break 'game,
                Ev::KeyDown {
                    keycode: Some(kc),
                    repeat: false,
                    ..
                } => match kc {
                    Keycode::A => movement.0 -= 1,
                    Keycode::D => movement.0 += 1,
                    Keycode::W => movement.1 -= 1,
                    Keycode::S => movement.1 += 1,
                    _ => (),
                },
                Ev::KeyUp {
                    keycode: Some(kc),
                    repeat: false,
                    ..
                } => match kc {
                    Keycode::A => movement.0 += 1,
                    Keycode::D => movement.0 -= 1,
                    Keycode::W => movement.1 += 1,
                    Keycode::S => movement.1 -= 1,
                    _ => (),
                },
                _ => (),
            }
        }

        let mut buf = [0; 64];
        if let Ok(read) = client.recv(&mut buf) {
            println!("got data: {:?}", &buf[..read]);
        }

        send(
            &client,
            Command {
                x: movement.0,
                y: movement.1,
            },
        );
        
        player_input(&mut state.shared, state.player_idx, movement);
        player_movement(&mut state);
        render(&state.shared, &mut sdl.canvas);
        sdl.canvas.present();

        sys::tick(start, FRAME_TIME);
    }

    Ok(())
}

struct State {
    player_idx: usize,
    shared: crate::Game,
}

fn player_input(game: &mut Game, player_idx: usize, movement: (i8, i8)) {
    game.players[player_idx].velocity =
        Vec2::new(movement.0 as f32, movement.1 as f32).normalize() * (PLAYER_SPEED * DELTA_TIME);
}

fn player_movement(game: &mut State) {
    for player in &mut game.shared.players {
        player.velocity += GRAVITY * DELTA_TIME;
        player.pos += player.velocity;
        
        dbg!(player.pos);
    }
}

fn collide(player: &mut Player, platforms: &Vec<Platform>) {
    for platform in platforms {
        if platform.pos.x + platform.size.0 + (player.size / 2.)
    }
}

fn fix_position(player: &mut Player, platform: &Platform) {
    let player_relative_posistion = platform.pos - player.pos;
}
