use macroquad::prelude::*;
use macroquad::rand::srand;

use crate::config::{WINDOW_W, WINDOW_H};
use crate::game::{Game, RoundState};
use crate::menu::Menu;

mod config;
mod game;
mod menu;


enum AppState {
    Menu(Menu),
    Playing(Game),
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
                game.draw();


                match game.round_state {
                    RoundState::RoundOver { .. } => {
                        if is_key_pressed(KeyCode::Space) {
                            game.restart_round();
                        }
                    }
                    RoundState::MatchOver { .. } => {
                        if is_key_pressed(KeyCode::R) {
                            game.restart_match();
                        }

                        if is_key_pressed(KeyCode::Enter) {
                            state = AppState::Menu(Menu::new());
                        }
                    }
                    _ => {}
                }
                
                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::Menu(Menu::new());
                }
            }
        }

        next_frame().await;
    }
}