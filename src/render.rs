//! Shared utilities for rendering the game through SDL2

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
};

use crate::{math::Vec2, model::*, sys};

const BG: Color = Color::WHITE;
const PLATFORM: Color = Color::BLACK;

const PLAYER_COLORS: &[Color] = &[
    Color::RED,
    Color::BLUE,
    Color::MAGENTA,
    Color::GREEN,
    Color::YELLOW,
    Color::CYAN,
];

fn centered_rect(pos: Vec2, size: (f64, f64)) -> Rect {
    Rect::from_center(Point::new(pos.x as _, pos.y as _), size.0 as _, size.1 as _)
}

/// Render the current game state.
pub fn game(game: &Game, canvas: &mut sdl2::render::WindowCanvas) {
    canvas.set_draw_color(BG);
    canvas.clear();

    canvas.set_draw_color(PLATFORM);
    for platform in &game.platforms {
        let r = centered_rect(platform.pos, platform.size);
        let _ = canvas.fill_rect(r);
    }

    for (i, player) in game.players.iter().enumerate() {
        assert!(i < PLAYER_COLORS.len(), "Not enough colors :(");

        canvas.set_draw_color(PLAYER_COLORS[i]);
        let r = centered_rect(player.pos, (player.size, player.size));
        let _ = canvas.fill_rect(r);
    }
}

/// Render settings.
///
/// Each setting is one line from the iterator.
/// Each setting will get its own line, with appropriate spacing between the settings.
pub fn settings<'a, L: IntoIterator<Item = &'a str>>(
    sdl: &mut sys::SdlContext,
    font: &sdl2::ttf::Font,
    lines: L,
) {
    for (i, line) in lines.into_iter().enumerate() {
        let surface = font.render(line).blended(Color::BLACK).unwrap();

        let texture = sdl
            .texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        let sdl2::render::TextureQuery { width, height, .. } = texture.query();

        let target = Rect::new(20, 4 + (height * i as u32) as i32, width, height);
        let _ = sdl.canvas.copy(&texture, None, Some(target));
    }
}
