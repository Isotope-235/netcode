use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
};

use crate::Game;

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

pub fn game(game: &Game, canvas: &mut sdl2::render::WindowCanvas) {
    canvas.set_draw_color(BG);
    canvas.clear();

    canvas.set_draw_color(PLATFORM);
    for platform in &game.platforms {
        let r = Rect::from_center(
            Point::new(platform.pos.x as _, platform.pos.y as _),
            platform.size.0 as _,
            platform.size.1 as _,
        );
        let _ = canvas.fill_rect(r);
    }

    for (i, player) in game.players.iter().enumerate() {
        assert!(i < PLAYER_COLORS.len(), "Not enough colors :(");

        canvas.set_draw_color(PLAYER_COLORS[i]);
        let r = Rect::from_center(
            Point::new(player.pos.x as _, player.pos.y as _),
            player.size as _,
            player.size as _,
        );
        let _ = canvas.fill_rect(r);
    }
}
