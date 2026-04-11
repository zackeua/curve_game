use macroquad::prelude::*;
use macroquad::rand::srand;

use crate::config::{WINDOW_W, WINDOW_H};
use crate::game::{Game, RoundEndAction};
use crate::menu::Menu;

mod config;
mod game;
mod menu;


enum AppState {
    Menu(Menu),
    Playing(Game),
}

struct Assets {
    speed_self: Texture2D,
    speed_others: Texture2D,
    slow_self: Texture2D,
    slow_others: Texture2D,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Zachtung!".to_string(),
        window_width: WINDOW_W as i32,
        window_height: WINDOW_H as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    srand(miniquad::date::now() as u64); // Seed random to get different maps each run

    let mut state = AppState::Menu(Menu::new());

    let speed_self_texture = load_texture("assets/powerups/speed_self.png").await.unwrap();
    let speed_others_texture = load_texture("assets/powerups/speed_others.png").await.unwrap();
    let slow_self_texture = load_texture("assets/powerups/slow_self.png").await.unwrap();
    let slow_others_texture = load_texture("assets/powerups/slow_others.png").await.unwrap();
    let assets = Assets {
        speed_self: speed_self_texture,
        speed_others: speed_others_texture,
        slow_self: slow_self_texture,
        slow_others: slow_others_texture,
    };

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        match &mut state {
            AppState::Menu(menu) => {
                menu.update();
                menu.draw();

                if (is_key_pressed(KeyCode::Enter) || menu.should_start_game()) && menu.is_ready() {
                    state = AppState::Playing(menu.build_game());
                }
            }

            AppState::Playing(game) => {
                game.update(dt);
                game.draw(&assets);

                let action = game.handle_round_end_input();
                if action == RoundEndAction::ReturnToMenu {
                    state = AppState::Menu(Menu::new());
                }
                
                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::Menu(Menu::new());
                }
            }
        }

        next_frame().await;
    }
}