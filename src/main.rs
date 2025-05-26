use std::error::Error;

mod client;
mod math;
mod model;
mod netcode;
mod networking;
mod render;
mod server;
mod sys;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    let mode = args.nth(1).unwrap_or_default();

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let ctx = sys::init_sdl_systems(&sdl, &video)?;
    let ttf = sdl2::ttf::init()?;
    let font = sys::load_font(&ttf)?;

    let shared_state = model::Game::new();

    match &mode[..] {
        "server" | "--server" => server::run(ctx, &font, shared_state),
        _ => client::run(ctx, &font, shared_state),
    }
}
