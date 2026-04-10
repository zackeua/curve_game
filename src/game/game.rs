use macroquad::prelude::*;

use crate::config::{SCREEN_W, SCREEN_H, UI_WIDTH, COLLISION_RADIUS, SELF_GRACE_POINTS, GameConfig};
use super::player::Player;
use super::powerup::{Powerup, PowerupType, apply_powerup};

pub struct PlayerInput {
    pub left: KeyCode,
    pub right: KeyCode,
}

#[derive(PartialEq, Clone, Debug)]
pub enum RoundState {
    Countdown { timer: f32 },
    Playing,
    RoundOver { winner: Option<usize> },
    MatchOver { winner: Option<usize> },
}

pub struct Game {
    pub players: Vec<Player>,
    pub inputs: Vec<PlayerInput>,
    pub colors: Vec<Color>,
    pub death_orders: Vec<Option<usize>>,
    pub scores: Vec<u32>,
    pub round_state: RoundState,

    pub config: GameConfig,

    pub powerups: Vec<Powerup>,
    pub spawn_timer: f32,
}

fn draw_border() {
    let thickness = 4.0;

    draw_rectangle_lines(0.0, 0.0, SCREEN_W, SCREEN_H, thickness, WHITE);
}

fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    if ab.length_squared() == 0.0 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance(closest)
}

impl Game {
    pub fn is_player_alive(&self, player_idx: usize) -> bool {
        self.death_orders[player_idx].is_none()
    }

    pub fn kill_player(&mut self, player_idx: usize) {
        if self.is_player_alive(player_idx) {
            let death_count = self.death_orders.iter().filter(|d| d.is_some()).count();
            self.death_orders[player_idx] = Some(death_count + 1);
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let RoundState::Countdown { timer } = &mut self.round_state {
            *timer -= dt;
            if *timer <= 0.0 {
                self.round_state = RoundState::Playing;
            }
            return;
        }

        if let RoundState::Playing = self.round_state {
            if self.config.powerups_enabled {
                self.spawn_timer += dt;
                if self.spawn_timer > 5.0 {
                    self.spawn_timer = 0.0;

                    use macroquad::rand::gen_range;
                    let pos = vec2(
                        gen_range(50.0, SCREEN_W - 50.0),
                        gen_range(50.0, SCREEN_H - 50.0),
                    );
                    let kind = match gen_range(0, 4) {
                        0 => PowerupType::SpeedSelf,
                        1 => PowerupType::SpeedOthers,
                        2 => PowerupType::SlowSelf,
                        3 => PowerupType::SlowOthers,
                        _ => PowerupType::ThickenTrail,
                    };

                    self.powerups.push(Powerup { pos, kind });
                }
            }

            for (p, input) in self.players.iter_mut().zip(self.inputs.iter()) {
                let mut turn = 0.0;

                if is_key_down(input.left) {
                    turn -= 1.0;
                }
                if is_key_down(input.right) {
                    turn += 1.0;
                }

                p.update(dt, turn, &self.config);
            }

            self.check_collision();

            for i in 0..self.players.len() {
                if !self.is_player_alive(i) {
                    continue;
                }

                let player_pos = self.players[i].pos;

                self.powerups.retain(|p| {
                    if player_pos.distance(p.pos) < 10.0 {
                        apply_powerup(i, p.kind, &mut self.players, &mut self.config);
                        false // remove powerup
                    } else {
                        true
                    }
                });
            }

            // Count alive players
            let alive: Vec<usize> = (0..self.players.len())
                .filter(|&i| self.is_player_alive(i))
                .collect();

            if alive.len() <= 1 {
                let winner = alive.first().cloned();

                // Award points based on death order
                for player_idx in 0..self.players.len() {
                    if let Some(rank) = self.death_orders[player_idx] {
                        // Points = rank minus 1 (first to die gets 1 point, etc.)
                        let points: usize = rank.saturating_sub(1);
                        self.scores[player_idx] += points as u32;
                    } else if alive.contains(&player_idx) {
                        // Last player alive gets max points
                        let points = self.players.len() - 1;
                        self.scores[player_idx] += points as u32;
                    }
                }

                // Check if any player has reached the target score
                let mut match_winner = None;
                for (i, &score) in self.scores.iter().enumerate() {
                    if score >= self.config.target_score {
                        match_winner = Some(i);
                        break;
                    }
                }

                if let Some(w) = match_winner {
                    self.round_state = RoundState::MatchOver { winner: Some(w) };
                    return;
                }

                self.round_state = RoundState::RoundOver { winner };
                self.powerups.clear();
            }
        }
    }

    fn draw_player(&self, player_idx: usize, show_direction: bool) {
        let player = &self.players[player_idx];
        let color = self.colors[player_idx];
        for i in 1..player.trail.len() {
            if let (Some(a), Some(b)) = (player.trail[i - 1], player.trail[i]) {
                draw_line(a.x, a.y, b.x, b.y, player.trail_thickness, color);
            }
        }
        if show_direction {
            let dir_vec = player.get_direction_vector() * 10.0;
            draw_line(
                player.pos.x,
                player.pos.y,
                player.pos.x + dir_vec.x,
                player.pos.y + dir_vec.y,
                2.0,
                YELLOW,
            );
        }

        draw_circle(player.pos.x, player.pos.y, 4.0, color);

        // Draw powerup effect duration indicator as arc beneath head
        if player.effect_timer > 0.0 {
            let max_duration = 5.0;
            let progress = (player.effect_timer / max_duration).clamp(0.0, 1.0);
            let arc_radius = 5.0;
            let num_segments = 30;
            let filled_segments = ((num_segments as f32) * progress).ceil() as i32;

            // Draw arc starting from top, going clockwise
            for i in 0..=filled_segments {
                let angle1 = -std::f32::consts::PI / 2.0
                    + (i as f32 / num_segments as f32) * std::f32::consts::PI * 2.0;
                let angle2 = -std::f32::consts::PI / 2.0
                    + ((i as f32 + 1.0) / num_segments as f32) * std::f32::consts::PI * 2.0;

                let x1 = player.pos.x + arc_radius * angle1.cos();
                let y1 = player.pos.y + arc_radius * angle1.sin();
                let x2 = player.pos.x + arc_radius * angle2.cos();
                let y2 = player.pos.y + arc_radius * angle2.sin();

                draw_line(x1, y1, x2, y2, 2.0, WHITE);
            }
        }
    }

    pub fn draw(&self) {
        draw_border();

        let show_direction = matches!(self.round_state, RoundState::Countdown { .. });

        for player_idx in 0..self.players.len() {
            self.draw_player(player_idx, show_direction);
        }
        // Powerups
        for p in &self.powerups {
            let color = match p.kind {
                PowerupType::SpeedSelf => YELLOW,
                PowerupType::SpeedOthers => PINK,
                PowerupType::SlowSelf => ORANGE,
                PowerupType::SlowOthers => RED,
                PowerupType::ThickenTrail => BLUE,
            };
            draw_circle(p.pos.x, p.pos.y, 6.0, color);
        }

        // Scores
        let panel_x = SCREEN_W + 20.0;
        draw_rectangle(
            SCREEN_W,
            0.0,
            UI_WIDTH,
            SCREEN_H,
            Color::from_rgba(30, 30, 30, 255),
        );

        draw_text("SCORES", panel_x, 40.0, 30.0, WHITE);

        for (i, score) in self.scores.iter().enumerate() {
            draw_text(
                &format!("P{}: {}", i, score),
                panel_x,
                80.0 + i as f32 * 30.0,
                25.0,
                self.colors[i],
            );
        }
        // Powerup info
        if self.config.powerups_enabled {
            let y_offset = 80.0 + self.players.len() as f32 * 30.0 + 40.0;
            draw_text("POWERUPS:", panel_x, y_offset, 30.0, WHITE);
            draw_text(
                "Yellow: Speed Self",
                panel_x,
                y_offset + 40.0,
                20.0,
                YELLOW,
            );
            draw_text(
                "Pink: Speed Others",
                panel_x,
                y_offset + 70.0,
                20.0,
                PINK,
            );
            draw_text(
                "Orange: Slow Self",
                panel_x,
                y_offset + 100.0,
                20.0,
                ORANGE,
            );
            draw_text(
                "Red: Slow Others",
                panel_x,
                y_offset + 130.0,
                20.0,
                RED,
            );
            //draw_text("Blue: Thicken Trail", panel_x, y_offset + 160.0, 20.0, BLUE);
        }

        // Countdown display
        if let RoundState::Countdown { timer } = self.round_state {
            let countdown = (timer.ceil() as i32).max(0);
            draw_text(
                &countdown.to_string(),
                SCREEN_W / 2.0 - 30.0,
                SCREEN_H / 2.0,
                120.0,
                if countdown > 1 { YELLOW } else { YELLOW },
            );
        }

        // results
        match self.round_state {
            RoundState::RoundOver { winner } => {
                let text = match winner {
                    Some(i) => format!("Player {} wins! Press SPACE to continue", i),
                    None => "It's a tie! Press SPACE to continue".to_string(),
                };

                draw_text(&text, 200.0, 50.0, 30.0, YELLOW);
            }
            RoundState::MatchOver { winner } => {
                let text = match winner {
                    Some(i) => format!("Player {} wins the match!", i),
                    None => "It's a tie! Press SPACE to restart".to_string(),
                };

                draw_text(&text, 180.0, 50.0, 40.0, YELLOW);
                draw_text("R = Replay | ENTER = Menu", 220.0, 90.0, 25.0, YELLOW);
            }
            _ => {}
        }
    }

    pub fn restart_round(&mut self) {
        use macroquad::rand::gen_range;

        let margin = 50.0;
        let min_distance = 80.0;

        for p in &mut self.players {
            p.pos = Vec2::ZERO; // Mark all players as unpositioned
        }

        for i in 0..self.players.len() {
            let mut pos;

            // try until we find a non-colliding position
            loop {
                pos = vec2(
                    gen_range(margin, SCREEN_W - margin),
                    gen_range(margin, SCREEN_H - margin),
                );

                if self.players.iter().all(|other| other.pos.distance(pos) > min_distance) {
                    break;
                }
            }

            self.players[i].reset(pos, gen_range(0.0, std::f32::consts::PI * 2.0));
        }

        self.powerups.clear();
        self.death_orders = vec![None; self.players.len()];

        self.round_state = RoundState::Countdown { timer: 3.0 };
    }

    pub fn restart_match(&mut self) {
        self.scores = vec![0; self.players.len()];
        self.death_orders = vec![None; self.players.len()];
        self.restart_round();
    }

    pub fn check_collision(&mut self) {
        for i in 0..self.players.len() {
            if self.death_orders[i].is_some() {
                continue;
            }

            let p = self.players[i].pos;
            if p.x < 0.0 || p.x > SCREEN_W || p.y < 0.0 || p.y > SCREEN_H {
                self.kill_player(i);
                continue;
            }

            for j in 0..self.players.len() {
                let trail = &self.players[j].trail;
                let len = trail.len();

                for k in 1..len {
                    if i == j && k > len.saturating_sub(SELF_GRACE_POINTS) {
                        continue;
                    }

                    if let (Some(a), Some(b)) = (trail[k - 1], trail[k]) {
                        if distance_to_segment(self.players[i].pos, a, b) < COLLISION_RADIUS {
                            self.kill_player(i);
                            break;
                        }
                    }
                }

                if self.death_orders[i].is_some() {
                    break;
                }
            }
        }
    }
}
