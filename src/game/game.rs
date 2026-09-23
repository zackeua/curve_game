use macroquad::prelude::*;

use super::player::Player;
use super::powerup::{Powerup, PowerupType, apply_powerup};
use crate::Assets;
use crate::config::{
    COLLISION_RADIUS, GameConfig, SCREEN_H, SCREEN_W, SELF_GRACE_POINTS, UI_WIDTH,
};

pub struct PlayerInput {
    pub left: String,
    pub right: String,
    pub ai: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum RoundState {
    Countdown { timer: f32 },
    Playing,
    RoundOver { winner: Option<usize> },
    MatchOver { winner: Option<usize> },
}
#[derive(PartialEq, Clone, Debug)]
pub enum RoundEndAction {
    ContinuePlaying,
    RestartRound,
    RestartMatch,
    ReturnToMenu,
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
    pub paused: bool,
    pub campaign_level: Option<u32>,
    pub maze_walls: Vec<(Vec2, Vec2)>,
    pub campaign_goal: Option<Vec2>,
    pub(crate) campaign_base_speed: f32,
    pub(crate) campaign_base_turn_speed: f32,
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

fn campaign_random_index(seed: &mut u64, length: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed as usize) % length
}

impl Game {
    pub fn new_campaign(config: GameConfig) -> Self {
        let campaign_base_speed = config.speed;
        let campaign_base_turn_speed = config.turn_speed;
        let mut game = Self {
            players: vec![],
            inputs: vec![PlayerInput {
                left: "a".to_string(),
                right: "d".to_string(),
                ai: false,
            }],
            colors: vec![RED],
            death_orders: vec![None],
            scores: vec![0],
            round_state: RoundState::Countdown { timer: 3.0 },
            config,
            powerups: vec![],
            spawn_timer: 0.0,
            paused: false,
            campaign_level: Some(1),
            maze_walls: vec![],
            campaign_goal: None,
            campaign_base_speed,
            campaign_base_turn_speed,
        };
        game.setup_campaign_level();
        game
    }

    fn setup_campaign_level(&mut self) {
        let level = self.campaign_level.unwrap_or(1);
        let mut seed = 0xC0FFEE_u64.wrapping_add(level as u64 * 0x9E37_79B9);
        let level_index = level.saturating_sub(1) as f32;
        let columns = (10 + level as usize / 2).min(22);
        let rows = (7 + level as usize / 3).min(14);
        let cell_width = SCREEN_W / columns as f32;
        let cell_height = SCREEN_H / rows as f32;

        self.config.speed = (self.campaign_base_speed * (1.0 + level_index * 0.035)).min(260.0);
        self.config.turn_speed =
            (self.campaign_base_turn_speed * (1.0 + level_index * 0.015)).min(8.0);
        let cell_count = columns * rows;
        let mut visited = vec![false; cell_count];
        let mut vertical_walls = vec![true; rows * (columns - 1)];
        let mut horizontal_walls = vec![true; (rows - 1) * columns];
        let mut stack = vec![0usize];
        visited[0] = true;

        while let Some(&current) = stack.last() {
            let column = current % columns;
            let row = current / columns;
            let mut neighbors = Vec::new();

            if column > 0 && !visited[current - 1] {
                neighbors.push((current - 1, 0));
            }
            if column + 1 < columns && !visited[current + 1] {
                neighbors.push((current + 1, 1));
            }
            if row > 0 && !visited[current - columns] {
                neighbors.push((current - columns, 2));
            }
            if row + 1 < rows && !visited[current + columns] {
                neighbors.push((current + columns, 3));
            }

            if neighbors.is_empty() {
                stack.pop();
                continue;
            }

            let choice = campaign_random_index(&mut seed, neighbors.len());
            let (next, direction) = neighbors[choice];
            match direction {
                0 => vertical_walls[row * (columns - 1) + column - 1] = false,
                1 => vertical_walls[row * (columns - 1) + column] = false,
                2 => horizontal_walls[(row - 1) * columns + column] = false,
                3 => horizontal_walls[row * columns + column] = false,
                _ => {}
            }
            visited[next] = true;
            stack.push(next);
        }

        let mut walls = Vec::new();
        for row in 0..rows {
            for column in 0..columns - 1 {
                if vertical_walls[row * (columns - 1) + column] {
                    let x = (column + 1) as f32 * cell_width;
                    let y = row as f32 * cell_height;
                    walls.push((vec2(x, y), vec2(x, y + cell_height)));
                }
            }
        }
        for row in 0..rows - 1 {
            for column in 0..columns {
                if horizontal_walls[row * columns + column] {
                    let x = column as f32 * cell_width;
                    let y = (row + 1) as f32 * cell_height;
                    walls.push((vec2(x, y), vec2(x + cell_width, y)));
                }
            }
        }

        self.maze_walls = walls;
        self.campaign_goal = Some(vec2(
            SCREEN_W - cell_width / 2.0,
            SCREEN_H - cell_height / 2.0,
        ));
        let start_direction = if !vertical_walls[0] {
            0.0
        } else {
            std::f32::consts::FRAC_PI_2
        };
        self.players = vec![Player::new(
            vec2(cell_width / 2.0, cell_height / 2.0),
            start_direction,
        )];
        self.death_orders = vec![None];
        self.round_state = RoundState::Countdown { timer: 3.0 };
        self.powerups.clear();
    }

    pub fn is_player_alive(&self, player_idx: usize) -> bool {
        self.death_orders[player_idx].is_none()
    }

    pub fn kill_player(&mut self, player_idx: usize) {
        if self.is_player_alive(player_idx) {
            let death_count = self.death_orders.iter().filter(|d| d.is_some()).count();
            self.death_orders[player_idx] = Some(death_count + 1);
            self.players[player_idx].reset_modifiers();
        }
    }

    fn ai_turn(&self, player_idx: usize) -> f32 {
        let player = &self.players[player_idx];
        let directions = [-1.0, -0.5, 0.0, 0.5, 1.0];
        let lookahead = [35.0, 70.0, 105.0, 140.0];

        let mut best_turn = 0.0;
        let mut best_danger = f32::NEG_INFINITY;
        for turn in directions {
            let mut safety = 0.0;
            for (step, distance) in lookahead.iter().enumerate() {
                let direction = player.dir + turn * 0.55;
                let probe = player.pos + vec2(direction.cos(), direction.sin()) * *distance;
                let wall_margin = probe
                    .x
                    .min(SCREEN_W - probe.x)
                    .min(probe.y)
                    .min(SCREEN_H - probe.y);

                safety += wall_margin * (step as f32 + 1.0);
                if wall_margin < 24.0 {
                    safety -= (24.0 - wall_margin) * 30.0;
                }

                for other in &self.players {
                    for segment in other.trail.windows(2) {
                        if let (Some(a), Some(b)) = (segment[0], segment[1]) {
                            let trail_distance = distance_to_segment(probe, a, b);
                            if trail_distance < 28.0 {
                                safety -= (28.0 - trail_distance) * (step as f32 + 2.0) * 8.0;
                            }
                        }
                    }
                }

                for (other_idx, other) in self.players.iter().enumerate() {
                    if other_idx == player_idx || !self.is_player_alive(other_idx) {
                        continue;
                    }

                    let other_probe = other.pos + other.get_direction_vector() * *distance;
                    let head_distance = probe.distance(other_probe);
                    if head_distance < 48.0 {
                        safety -= (48.0 - head_distance) * (step as f32 + 2.0) * 14.0;
                    }
                }
            }

            if safety > best_danger {
                best_danger = safety;
                best_turn = turn;
            }
        }

        best_turn
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
            // Check for pause toggle
            if is_key_pressed(KeyCode::Backspace) {
                self.paused = !self.paused;
            }

            // Skip game updates if paused
            if self.paused {
                return;
            }
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

            // Update alive player positions based on input
            for player_idx in 0..self.players.len() {
                if self.is_player_alive(player_idx) {
                    let input = &self.inputs[player_idx];
                    let turn = if input.ai {
                        self.ai_turn(player_idx)
                    } else if crate::input::is_key_down(&input.left) {
                        -1.0
                    } else if crate::input::is_key_down(&input.right) {
                        1.0
                    } else {
                        0.0
                    };

                    self.players[player_idx].update(dt, turn, &self.config);
                }
            }

            self.check_collision();

            if self.campaign_level.is_some()
                && self.is_player_alive(0)
                && self
                    .campaign_goal
                    .is_some_and(|goal| self.players[0].pos.distance(goal) < 18.0)
            {
                self.round_state = RoundState::RoundOver { winner: Some(0) };
                self.powerups.clear();
                return;
            }

            for i in 0..self.players.len() {
                if !self.is_player_alive(i) {
                    continue;
                }

                let player_pos = self.players[i].pos;

                self.powerups.retain(|p| {
                    if player_pos.distance(p.pos) < 24.0 {
                        apply_powerup(
                            i,
                            p.kind,
                            &mut self.players,
                            &self.death_orders,
                            &mut self.config,
                        );
                        false // remove powerup
                    } else {
                        true
                    }
                });
            }

            if self.campaign_level.is_some() {
                if !self.is_player_alive(0) {
                    self.round_state = RoundState::RoundOver { winner: None };
                    self.powerups.clear();
                }
                return;
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
                let target_reached: Vec<_> = self
                    .scores
                    .iter()
                    .enumerate()
                    .filter(|(_, score)| *score >= &self.config.target_score)
                    .collect();

                let match_winner = if target_reached.is_empty() {
                    None
                } else {
                    let max_score = target_reached
                        .iter()
                        .map(|(_, score)| *score)
                        .max()
                        .unwrap();
                    let tied_winners: Vec<_> = target_reached
                        .iter()
                        .filter(|(_, score)| *score == max_score)
                        .collect();

                    if tied_winners.len() == 1 {
                        Some(tied_winners[0].0)
                    } else {
                        None // Multiple players tied at max score
                    }
                };

                if let Some(w) = match_winner {
                    self.round_state = RoundState::MatchOver { winner: Some(w) };
                    return;
                }

                self.round_state = RoundState::RoundOver { winner };
                self.powerups.clear();
            }
        }
    }

    fn draw_player(&self, player_idx: usize) {
        let player = &self.players[player_idx];
        let color = self.colors[player_idx];
        for i in 1..player.trail.len() {
            if let (Some(a), Some(b)) = (player.trail[i - 1], player.trail[i]) {
                draw_line(a.x, a.y, b.x, b.y, player.trail_thickness, color);
            }
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

    pub fn draw(&self, assets: &Assets) {
        draw_border();

        let powerup_size = 48.0;
        let no_color = Color::from_rgba(255, 255, 255, 255);
        // Powerups
        for p in &self.powerups {
            match p.kind {
                PowerupType::SpeedSelf => {
                    let mut params = DrawTextureParams::default();
                    params.dest_size = Some(vec2(powerup_size, powerup_size));
                    draw_texture_ex(
                        &assets.speed_self,
                        p.pos.x - powerup_size / 2.0,
                        p.pos.y - powerup_size / 2.0,
                        no_color,
                        params,
                    );
                }
                PowerupType::SpeedOthers => {
                    let mut params = DrawTextureParams::default();
                    params.dest_size = Some(vec2(powerup_size, powerup_size));
                    draw_texture_ex(
                        &assets.speed_others,
                        p.pos.x - powerup_size / 2.0,
                        p.pos.y - powerup_size / 2.0,
                        no_color,
                        params,
                    );
                }
                PowerupType::SlowSelf => {
                    let mut params = DrawTextureParams::default();
                    params.dest_size = Some(vec2(powerup_size, powerup_size));
                    draw_texture_ex(
                        &assets.slow_self,
                        p.pos.x - powerup_size / 2.0,
                        p.pos.y - powerup_size / 2.0,
                        no_color,
                        params,
                    );
                }
                PowerupType::SlowOthers => {
                    let mut params = DrawTextureParams::default();
                    params.dest_size = Some(vec2(powerup_size, powerup_size));
                    draw_texture_ex(
                        &assets.slow_others,
                        p.pos.x - powerup_size / 2.0,
                        p.pos.y - powerup_size / 2.0,
                        no_color,
                        params,
                    );
                }
                PowerupType::ThickenTrail => {
                    draw_circle(p.pos.x, p.pos.y, 12.0, BLUE);
                }
            };
        }

        for (a, b) in &self.maze_walls {
            draw_line(a.x, a.y, b.x, b.y, 6.0, GRAY);
        }

        if let Some(goal) = self.campaign_goal {
            draw_circle(goal.x, goal.y, 16.0, GREEN);
            draw_circle_lines(goal.x, goal.y, 16.0, 3.0, WHITE);
        }

        // Draw players and their trails after obstacles so walls do not cover riders.
        for player_idx in 0..self.players.len() {
            self.draw_player(player_idx);
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

        if let Some(level) = self.campaign_level {
            draw_text(&format!("LEVEL {}", level), panel_x, 70.0, 24.0, YELLOW);
            draw_text("Reach the green exit", panel_x, 105.0, 16.0, WHITE);
        }

        for (i, score) in self.scores.iter().enumerate() {
            draw_text(
                &format!("P{}: {}", i, score),
                panel_x,
                if self.campaign_level.is_some() {
                    135.0 + i as f32 * 30.0
                } else {
                    80.0 + i as f32 * 30.0
                },
                25.0,
                self.colors[i],
            );
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

        // Pause indicator
        if self.paused {
            draw_text(
                "PAUSED",
                SCREEN_W / 2.0 - 80.0,
                SCREEN_H / 2.0 - 20.0,
                60.0,
                YELLOW,
            );
            draw_text(
                "Press BACKSPACE to resume",
                SCREEN_W / 2.0 - 130.0,
                SCREEN_H / 2.0 + 40.0,
                25.0,
                WHITE,
            );
        }

        // results
        match self.round_state {
            RoundState::RoundOver { winner } => {
                let text = if self.campaign_level.is_some() {
                    if winner == Some(0) {
                        format!(
                            "Level {} complete! Press SPACE for the next level",
                            self.campaign_level.unwrap()
                        )
                    } else {
                        "You crashed! Press SPACE to retry the level".to_string()
                    }
                } else {
                    match winner {
                        Some(i) => format!("Player {} wins! Press SPACE to continue", i),
                        None => "It's a tie! Press SPACE to continue".to_string(),
                    }
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
        if self.campaign_level.is_some() {
            self.setup_campaign_level();
            return;
        }

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

                if self
                    .players
                    .iter()
                    .all(|other| other.pos.distance(pos) > min_distance)
                {
                    break;
                }
            }

            let dir = gen_range(0.0, std::f32::consts::PI * 2.0);
            self.players[i].reset(pos, dir);
        }

        self.powerups.clear();
        self.death_orders = vec![None; self.players.len()];

        self.round_state = RoundState::Countdown { timer: 3.0 };
    }

    pub fn restart_match(&mut self) {
        if let Some(level) = &mut self.campaign_level {
            *level += 1;
            self.setup_campaign_level();
            return;
        }

        self.scores = vec![0; self.players.len()];
        self.death_orders = vec![None; self.players.len()];
        self.restart_round();
    }

    pub fn handle_round_end_input(&mut self) -> RoundEndAction {
        match self.round_state {
            RoundState::RoundOver { .. } => {
                if is_key_pressed(KeyCode::Space) {
                    if self.campaign_level.is_some()
                        && matches!(self.round_state, RoundState::RoundOver { winner: Some(0) })
                    {
                        self.restart_match();
                        RoundEndAction::RestartMatch
                    } else {
                        self.restart_round();
                        RoundEndAction::RestartRound
                    }
                } else {
                    RoundEndAction::ContinuePlaying
                }
            }
            RoundState::MatchOver { .. } => {
                if is_key_pressed(KeyCode::R) {
                    self.restart_match();
                    RoundEndAction::RestartMatch
                } else if is_key_pressed(KeyCode::Enter) {
                    RoundEndAction::ReturnToMenu
                } else {
                    RoundEndAction::ContinuePlaying
                }
            }
            _ => RoundEndAction::ContinuePlaying,
        }
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

            if self
                .maze_walls
                .iter()
                .any(|&(a, b)| distance_to_segment(p, a, b) < COLLISION_RADIUS + 3.0)
            {
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
