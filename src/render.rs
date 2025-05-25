use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
};

use crate::{Game, math::Vec2};

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
