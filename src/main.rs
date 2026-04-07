use macroquad::prelude::*;
use macroquad::rand::srand;
// ================== CONSTANTS ==================


const UI_WIDTH: f32 = 200.0;
const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

const WINDOW_W: f32 = SCREEN_W + UI_WIDTH;
const WINDOW_H: f32 = SCREEN_H;

const SPEED: f32 = 120.0;
const TURN_SPEED: f32 = 4.0;
const TRAIL_STEP: f32 = 5.0;
const COLLISION_RADIUS: f32 = 3.0;
const SELF_GRACE_POINTS: usize = 15;

const COLORS: [Color; 6] = [RED, BLUE, GREEN, YELLOW, ORANGE, PINK];

// ================== GAME CORE ==================

#[derive(Clone, Copy)]
enum PowerupType {
    SpeedSelf,
    SpeedOthers,
    SlowSelf,
    SlowOthers,
    ThickenTrail,
}

struct  Powerup {
    pos: Vec2,
    kind: PowerupType,
}


#[derive(Clone)]
struct Player {
    pos: Vec2,
    dir: f32,
    color: Color,
    alive: bool,
    trail: Vec<Option<Vec2>>,

    hole_timer: f32,
    hole_cooldown: f32,
    in_hole: bool,

    speed_multiplier: f32,
    turn_multiplier: f32,
    effect_timer: f32,
    trail_thickness: f32,
    
    death_order: Option<usize>,
}

impl Player {
    fn new(pos: Vec2, dir: f32, color: Color) -> Self {
        Self {
            pos,
            dir,
            color,
            alive: true,
            trail: vec![Some(pos)],
            hole_timer: 0.0,
            hole_cooldown: rand::gen_range(1.5, 3.0),
            in_hole: false,
            speed_multiplier: 1.0,
            turn_multiplier: 1.0,
            effect_timer: 0.0,
            trail_thickness: 3.0,
            death_order: None,
        }
    }

    fn get_direction_vector(&self) -> Vec2 {
        vec2(self.dir.cos(), self.dir.sin())
    }

    fn update(&mut self, dt: f32, turn: f32, config: &GameConfig) {
        if !self.alive { return; }

        self.dir += turn * config.turn_speed * dt;
        self.hole_timer += dt;

        let velocity = self.get_direction_vector() * config.speed * dt;
        self.pos += velocity * self.speed_multiplier;

        if self.in_hole && self.hole_timer > config.hole_duration {
            self.in_hole = false;
            self.hole_timer = 0.0;
            self.hole_cooldown = rand::gen_range(config.hole_interval_min, config.hole_interval_max);
            
        } else if !self.in_hole && self.hole_timer > self.hole_cooldown {
            self.in_hole = true;
            self.hole_timer = 0.0;
        }

        let last = self.trail.iter().rev().find_map(|&p| p);

        let step_distance = TRAIL_STEP * config.speed / SPEED; // Adjust step based on speed config
        if let Some(last_pos) = last {
            if last_pos.distance(self.pos) > step_distance {
                if self.in_hole {
                    // Only insert one None (avoid spam)
                    if !matches!(self.trail.last(), Some(None)) {
                        self.trail.push(None);
                    }
                } else {
                    self.trail.push(Some(self.pos));
                }
            }
        }

        if self.effect_timer > 0.0 {
            self.effect_timer -= dt;
            if self.effect_timer <= 0.0 {
                self.speed_multiplier = 1.0;
                self.turn_multiplier = 1.0;
                self.trail_thickness = 3.0;
            }
        }
    }

    fn draw(&self, show_direction: bool) {
        for i in 1..self.trail.len() {
            if let (Some(a), Some(b)) = (self.trail[i - 1], self.trail[i]) {
                draw_line(a.x, a.y, b.x, b.y, self.trail_thickness, self.color);
            }
        }
        if show_direction {
            let dir_vec = self.get_direction_vector() * 10.0;
            draw_line(
                self.pos.x,
                self.pos.y,
                self.pos.x + dir_vec.x,
                self.pos.y + dir_vec.y,
                2.0,
                YELLOW,
            );
        }
        draw_circle(self.pos.x, self.pos.y, 4.0, self.color);
    }

    fn reset(&mut self, pos: Vec2, dir: f32) {
        self.pos = pos;
        self.dir = dir;
        self.trail.clear();
        self.trail.push(Some(pos));
        self.alive = true;
        self.hole_timer = 0.0;
        self.in_hole = false;
        self.speed_multiplier = 1.0;
        self.turn_multiplier = 1.0;
        self.effect_timer = 0.0;
        self.trail_thickness = 3.0;
        self.death_order = None;
    }
}

fn apply_powerup(player_idx: usize, kind: PowerupType, players: &mut [Player], _config: &mut GameConfig) {
    match kind {
        PowerupType::SpeedSelf => {
            players[player_idx].speed_multiplier = 1.5;
            players[player_idx].effect_timer = 5.0;
        }
        PowerupType::SpeedOthers => {
            for (i, p) in players.iter_mut().enumerate() {
                if i != player_idx {
                    p.speed_multiplier = 1.5;
                    p.effect_timer = 5.0;
                }
            }
        }
        PowerupType::SlowOthers => {
            for (i, p) in players.iter_mut().enumerate() {
                if i != player_idx {
                    p.speed_multiplier = 0.5;
                    p.effect_timer = 5.0;
                }
            }
        }
        PowerupType::SlowSelf => {
            players[player_idx].speed_multiplier = 0.5;
            players[player_idx].effect_timer = 5.0;
        }
        PowerupType::ThickenTrail => {
            players[player_idx].trail_thickness = 6.0;
            players[player_idx].effect_timer = 5.0;
        }
    }
}

struct PlayerInput {
    left: KeyCode,
    right: KeyCode,
}

#[derive(PartialEq, Clone, Debug)]
enum RoundState {
    Countdown { timer: f32},
    Playing,
    RoundOver { winner: Option<usize> },
    MatchOver { winner: Option<usize> },
}

#[derive(Clone)]
struct GameConfig {
    speed: f32,
    turn_speed: f32,
    hole_interval_min: f32,
    hole_interval_max: f32,
    hole_duration: f32,
    target_score: u32,

    powerups_enabled: bool,
}

struct Game {
    players: Vec<Player>,
    inputs: Vec<PlayerInput>,
    scores: Vec<u32>,
    round_state: RoundState,

    config: GameConfig,

    powerups: Vec<Powerup>,
    spawn_timer: f32,
}

fn draw_border() {
    let thickness = 4.0;

    draw_rectangle_lines(
        0.0,
        0.0,
        SCREEN_W,
        SCREEN_H,
        thickness,
        WHITE,);
}

impl Game {
    fn update(&mut self, dt: f32) {

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

            check_collision(&mut self.players);

            for i in 0..self.players.len() {
                if !self.players[i].alive { continue; }

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
            let alive: Vec<usize> = self.players.iter()
                .enumerate()
                .filter(|(_, p)| p.alive)
                .map(|(i, _)| i)
                .collect();

            if alive.len() <= 1 {
                let winner = alive.first().cloned();

                // Award points based on death order
                for (player_idx, player) in self.players.iter().enumerate() {
                    if !player.alive && player.death_order.is_some() {
                        // Points = number of players, minus their death order (0 = first to die gets 1 point, etc.)
                        let rank = player.death_order.unwrap();
                        let points: usize = rank.saturating_sub(1);
                        self.scores[player_idx] += points as u32;
                    } else if alive.contains(&player_idx) {
                        // Last player alive gets max points
                        let points = self.players.len() - 1;
                        self.scores[player_idx] += points as u32;
                    }
                }

                if let Some(w) = winner {
                    if self.scores[w] >= self.config.target_score {
                        self.round_state = RoundState::MatchOver { winner: Some(w) };
                        return;
                    }
                }

                self.round_state = RoundState::RoundOver { winner };
                self.powerups.clear();
            }

        }
    }

    fn draw(&self) {
        draw_border();

        let show_direction = matches!(self.round_state, RoundState::Countdown { .. });

        for p in &self.players {
            p.draw(show_direction);
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
        draw_rectangle(SCREEN_W, 0.0, UI_WIDTH, SCREEN_H, Color::from_rgba(30, 30, 30, 255));

        draw_text("SCORES", panel_x, 40.0, 30.0, WHITE);

        for (i, score) in self.scores.iter().enumerate() {
            draw_text(
                &format!("P{}: {}", i + 1, score),
                 panel_x,
                 80.0 + i as f32 * 30.0,
                 25.0,
                 self.players[i].color
            );
        }
        // Powerup info
        if self.config.powerups_enabled {
            let y_offset = 80.0 + self.players.len() as f32 * 30.0 + 40.0;
            draw_text("POWERUPS:", panel_x, y_offset, 30.0, WHITE);
            draw_text("Yellow: Speed Self", panel_x, y_offset + 40.0, 20.0, YELLOW);
            draw_text("Pink: Speed Others", panel_x, y_offset + 70.0, 20.0, PINK);
            draw_text("Orange: Slow Self", panel_x, y_offset + 100.0, 20.0, ORANGE);
            draw_text("Red: Slow Others", panel_x, y_offset + 130.0, 20.0, RED);
            //draw_text("Blue: Thicken Trail", panel_x, y_offset + 160.0, 20.0, BLUE);
        }


        // results
        match self.round_state {
            RoundState::RoundOver { winner } =>{
                let text = match winner {
                    Some(i) => format!("Player {} wins! Press SPACE to continue", i + 1),
                    None => "It's a tie! Press SPACE to continue".to_string(),
                };

                draw_text(&text, 200.0, 50.0, 30.0, YELLOW);
            },
            RoundState::MatchOver { winner } => {
                let text = match winner {
                    Some(i) => format!("Player {} wins the match!", i + 1),
                    None => "It's a tie! Press SPACE to restart".to_string(),
                };

                draw_text(&text, 180.0, 50.0, 40.0, YELLOW);
                draw_text("R = Replay | ENTER = Menu", 220.0, 90.0, 25.0, YELLOW);
            },
            _ => {}
        }
        
    }

    fn restart_round(&mut self) {
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

            self.players[i].reset(
                pos,
                gen_range(0.0, std::f32::consts::PI * 2.0)
            );
        }

        self.powerups.clear();

        self.round_state = RoundState::Countdown { timer: 3.0 };
    }

    fn restart_match(&mut self) {
        self.scores = vec![0; self.players.len()];
        // self.scores.iter_mut().for_each(|s| *s = 0);
        self.restart_round();
    }

}

// ================== MENU ==================

#[derive(Clone)]
struct PlayerConfig {
    left: Option<KeyCode>,
    right: Option<KeyCode>,
    color: Color,
}

enum BindingState {
    None,
    Left(usize),
    Right(usize),
}

struct Menu {
    configs: Vec<PlayerConfig>,
    selected: usize,
    binding: BindingState,
    
    game_config: GameConfig,
    config_selected: usize,
    
    mouse_x: f32,
    mouse_y: f32,
}

fn is_mouse_over(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn key_to_string(key: Option<KeyCode>) -> String {
    match key {
        Some(k) => match k {
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            _ => format!("{:?}", k),
        },
        None => "-".to_string(),
    }
}

impl Menu {
    fn new() -> Self {
        Self {
            configs: vec![],
            selected: 0,
            binding: BindingState::None,

            game_config: GameConfig {
                speed: SPEED,
                turn_speed: TURN_SPEED,
                hole_interval_min: 1.5,
                hole_interval_max: 3.0,
                hole_duration: 0.3,
                target_score: 20,
                powerups_enabled: true,
            },
            config_selected: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }

    fn key_in_use(&self, key: KeyCode) -> bool {
        if self.configs.iter().any(|p|
            p.left == Some(key) || p.right == Some(key)
        ) {
            return true;
        }
        if key == KeyCode::N || key == KeyCode::Space || key == KeyCode::C || key == KeyCode::Enter {
            return true;
        }
        false
    }

    fn next_free_color(&self) -> Color {
        for &c in &COLORS {
            if !self.configs.iter().any(|p| p.color == c) {
                return c;
            }
        }
        WHITE
    }

    fn is_ready(&self) -> bool {
        !self.configs.is_empty()
            && self.configs.iter().all(|p| p.left.is_some() && p.right.is_some())
    }

    fn cycle_player_color(&mut self, player_idx: usize) {
        let current_color = self.configs[player_idx].color;
        let used: Vec<Color> = self.configs.iter()
            .enumerate()
            .filter(|(i, _)| *i != player_idx)
            .map(|(_, p)| p.color)
            .collect();
        
        let mut idx = COLORS.iter()
            .position(|&c| c == current_color)
            .unwrap_or(0);
        
        for _ in 0..COLORS.len() {
            idx = (idx + 1) % COLORS.len();
            let candidate = COLORS[idx];
            if !used.contains(&candidate) {
                self.configs[player_idx].color = candidate;
                break;
            }
        }
    }

    fn add_player(&mut self) {
        let color = self.next_free_color();
        self.configs.push(PlayerConfig {
            left: None,
            right: None,
            color,
        });
    }

    fn update(&mut self) {
        // Update mouse position
        let (mx, my) = mouse_position();
        self.mouse_x = mx;
        self.mouse_y = my;

        // Handle key binding input (takes priority)
        if self.handle_key_binding() {
            return;
        }

        self.handle_player_management();
        self.handle_config_adjustment();
    }

    fn handle_key_binding(&mut self) -> bool {
        if !matches!(self.binding, BindingState::None) {
            if let Some(key) = get_last_key_pressed() {
                if !self.key_in_use(key) {
                    match self.binding {
                        BindingState::Left(i) => {
                            self.configs[i].left = Some(key);
                            self.binding = BindingState::Right(i);
                        }
                        BindingState::Right(i) => {
                            self.configs[i].right = Some(key);
                            self.binding = BindingState::None;
                        }
                        _ => {}
                    }
                }
            }
            return true;
        }
        false
    }

    fn handle_player_management(&mut self) {
        // Add player
        if (is_key_pressed(KeyCode::N)) 
            || (is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(20.0, 135.0, 120.0, 30.0)) {
            self.add_player();
        }

        // Select player with keyboard
        if is_key_pressed(KeyCode::Up) && self.selected > 0 {
            self.selected -= 1;
        }
        if is_key_pressed(KeyCode::Down) && self.selected + 1 < self.configs.len() {
            self.selected += 1;
        }

        // Select player with mouse
        let list_base_y = 210.0;
        for i in 0..self.configs.len() {
            let y = list_base_y + i as f32 * 40.0;
            if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(20.0, y, 350.0, 35.0) {
                self.selected = i;
            }
        }

        // Skip remaining player actions if no players
        if self.configs.is_empty() {
            return;
        }

        let list_height = 40.0 * self.configs.len() as f32;
        let buttons_y = 220.0 + list_height;
        
        // Bind keys
        if (is_key_pressed(KeyCode::Space)) 
            || (is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(20.0, buttons_y, 170.0, 30.0)) {
            self.binding = BindingState::Left(self.selected);
        }

        // Change color
        if (is_key_pressed(KeyCode::C)) 
            || (is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(200.0, buttons_y, 170.0, 30.0)) {
            self.cycle_player_color(self.selected);
        }
    }

    fn handle_config_adjustment(&mut self) {
        // Keyboard: cycle through config items
        if is_key_pressed(KeyCode::U) {
            self.config_selected = (self.config_selected + 1) % 7;
        }

        // Keyboard: adjust selected config
        if is_key_pressed(KeyCode::Left) && self.game_config.target_score > 1 {
            self.adjust_config_left();
        }
        if is_key_pressed(KeyCode::Right) && self.game_config.target_score < 99 {
            self.adjust_config_right();
        }

        // Mouse: interact with config items
        // section_y = 100.0, base_y = 170.0, each item is 50 units apart
        let base_y = 170.0;
        for i in 0..7 {
            let y = base_y + (i as f32 * 50.0);
            
            // Click on config item to select it
            if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(400.0, y, 400.0, 35.0) {
                self.config_selected = i;
            }
            
            // Click left arrow to decrease
            if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(410.0, y + 5.0, 30.0, 25.0) && self.game_config.target_score > 1 {
                self.config_selected = i;
                self.adjust_config_left();
            }
            
            // Click right arrow to increase
            if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(760.0, y + 5.0, 30.0, 25.0) && self.game_config.target_score < 99 {
                self.config_selected = i;
                self.adjust_config_right();
            }
        }
    }

    fn should_start_game(&self) -> bool {
        // Calculate start button position (matches draw_config_section)
        let base_y = 170.0; // section_y (100.0) + 70.0
        let start_y = base_y + 360.0;
        let start_x = 400.0;
        
        is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(start_x, start_y, 180.0, 40.0)
    }

    fn adjust_config_left(&mut self) {
        match self.config_selected {
            0 => self.game_config.speed = (self.game_config.speed - 10.0).max(50.0),
            1 => self.game_config.turn_speed = (self.game_config.turn_speed - 0.5).max(1.0),
            2 => self.game_config.hole_duration = (self.game_config.hole_duration - 0.1).max(0.1),
            3 => {
                self.game_config.hole_interval_min = (self.game_config.hole_interval_min - 0.5).max(0.5);
            },
            4 => {
                self.game_config.hole_interval_max = (self.game_config.hole_interval_max - 0.5).max(self.game_config.hole_interval_min + 0.5);
            },
            5 => {
                self.game_config.target_score = (self.game_config.target_score - 1).max(1);
            }
            6 => {
                self.game_config.powerups_enabled = !self.game_config.powerups_enabled;
            }
            _ => {}
        }
    }

    fn adjust_config_right(&mut self) {
        match self.config_selected {
            0 => self.game_config.speed = (self.game_config.speed + 10.0).min(400.0),
            1 => self.game_config.turn_speed = (self.game_config.turn_speed + 0.5).min(10.0),
            2 => self.game_config.hole_duration = (self.game_config.hole_duration + 0.1).min(2.0),
            3 => {
                self.game_config.hole_interval_min = (self.game_config.hole_interval_min + 0.5).min(5.0);
            },
            4 => {
                self.game_config.hole_interval_max = (self.game_config.hole_interval_max + 0.5).min(10.0);
            },
            5 => {
                self.game_config.target_score = (self.game_config.target_score + 1).min(99);
            }
            6 => {
                self.game_config.powerups_enabled = !self.game_config.powerups_enabled;
            }
            _ => {}
        }
    }

    fn draw(&self) {
        // Background panels
        draw_rectangle(10.0, 50.0, 370.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(10.0, 50.0, 370.0, 530.0, 2.0, Color::from_rgba(100, 100, 100, 255));
        
        draw_rectangle(390.0, 50.0, 420.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(390.0, 50.0, 420.0, 530.0, 2.0, Color::from_rgba(100, 100, 100, 255));

        // Title
        draw_text("ZACHTUNG!", 20.0, 80.0, 40.0, YELLOW);

        self.draw_player_section();
        self.draw_config_section();
    }

    fn draw_player_section(&self) {
        let section_x = 20.0;
        let section_y = 100.0;
        
        // Section header
        draw_text("PLAYERS", section_x, section_y, 28.0, WHITE);
        draw_line(section_x, section_y + 10.0, section_x + 150.0, section_y + 10.0, 2.0, Color::from_rgba(100, 100, 100, 255));
        
        // Add player button
        let add_btn_hover = is_mouse_over(section_x, section_y + 35.0, 120.0, 30.0);
        let btn_color = if add_btn_hover { YELLOW } else { Color::from_rgba(80, 80, 80, 255) };
        draw_rectangle_lines(section_x, section_y + 35.0, 120.0, 30.0, 2.0, btn_color);
        draw_text(
            "[N] Add Player",
            section_x + 10.0,
            section_y + 55.0,
            18.0,
            btn_color
        );

        // Player list header
        draw_text("Select Player (↑↓):", section_x, section_y + 85.0, 18.0, WHITE);

        // Player list
        for (i, p) in self.configs.iter().enumerate() {
            self.draw_player_item(i, p, section_y + 110.0);
        }

        if !self.configs.is_empty() {
            let list_height = 40.0 * self.configs.len() as f32;
            let buttons_y = section_y + 120.0 + list_height;
            
            // Bind keys button
            let bind_btn_hover = is_mouse_over(section_x, buttons_y, 170.0, 30.0);
            let btn_color = if bind_btn_hover { YELLOW } else { Color::from_rgba(80, 80, 80, 255) };
            draw_rectangle_lines(section_x, buttons_y, 170.0, 30.0, 2.0, btn_color);
            draw_text(
                "[SPACE] Bind Keys",
                section_x + 10.0,
                buttons_y + 20.0,
                16.0,
                btn_color
            );

            // Change color button
            let color_btn_hover = is_mouse_over(section_x + 180.0, buttons_y, 170.0, 30.0);
            let btn_color = if color_btn_hover { YELLOW } else { Color::from_rgba(80, 80, 80, 255) };
            draw_rectangle_lines(section_x + 180.0, buttons_y, 170.0, 30.0, 2.0, btn_color);
            draw_text(
                "[C] Change Color",
                section_x + 190.0,
                buttons_y + 20.0,
                16.0,
                btn_color
            );
        }

        // Key binding prompt
        match self.binding {
            BindingState::Left(i) => {
                draw_rectangle(section_x, 520.0, 350.0, 50.0, Color::from_rgba(50, 0, 0, 255));
                draw_text(
                    &format!("P{}: Press LEFT key", i + 1),
                    section_x + 10.0,
                    545.0,
                    20.0,
                    YELLOW,
                );
            }
            BindingState::Right(i) => {
                draw_rectangle(section_x, 520.0, 350.0, 50.0, Color::from_rgba(50, 0, 0, 255));
                draw_text(
                    &format!("P{}: Press RIGHT key", i + 1),
                    section_x + 10.0,
                    545.0,
                    20.0,
                    YELLOW,
                );
            }
            _ => {}
        }
    }

    fn draw_player_item(&self, index: usize, config: &PlayerConfig, base_y: f32) {
        let y = base_y + index as f32 * 40.0;
        let is_selected = index == self.selected;
        let is_hovered = is_mouse_over(20.0, y, 350.0, 35.0);
        
        // Background
        if is_hovered || is_selected {
            let bg_color = if is_selected {
                Color::from_rgba(50, 50, 100, 255)
            } else {
                Color::from_rgba(40, 40, 40, 255)
            };
            draw_rectangle(20.0, y - 2.0, 350.0, 35.0, bg_color);
        }

        let prefix = if is_selected { "> " } else { "  " };
        let text_color = if is_selected { YELLOW } else { config.color };
        
        draw_text(
            &format!(
                "{}P{} | L:{} R:{} [{}]",
                prefix,
                index + 1,
                key_to_string(config.left),
                key_to_string(config.right),
                "●"
            ),
            30.0,
            y + 20.0,
            18.0,
            text_color,
        );
        
        // Color indicator circle
        draw_circle(340.0, y + 13.0, 6.0, config.color);
    }

    fn draw_config_section(&self) {
        let section_x = 400.0;
        let section_y = 100.0;
        
        // Section header
        draw_text("GAME CONFIG", section_x, section_y, 28.0, WHITE);
        draw_line(section_x, section_y + 10.0, section_x + 200.0, section_y + 10.0, 2.0, Color::from_rgba(100, 100, 100, 255));
        
        draw_text("Click setting or use (UP/DOWN) to adjust", section_x, section_y + 40.0, 14.0, Color::from_rgba(150, 150, 150, 255));
        
        let items = [
            format!("Speed: {:.0}", self.game_config.speed),
            format!("Turn Speed: {:.1}", self.game_config.turn_speed),
            format!("Hole Size: {:.1}", self.game_config.hole_duration),
            format!("Hole min: {:.1}", self.game_config.hole_interval_min),
            format!("Hole max: {:.1}", self.game_config.hole_interval_max),
            format!("Target Score: {}", self.game_config.target_score),
            format!("Powerups: {}", if self.game_config.powerups_enabled { "ON" } else { "OFF" }),
        ];

        let base_y = section_y + 70.0;
        for (i, text) in items.iter().enumerate() {
            self.draw_config_item(i, text, base_y + i as f32 * 50.0);
        }

        // Start button at the bottom
        let start_y = base_y + 360.0;
        let start_hover = is_mouse_over(section_x, start_y, 180.0, 40.0);
        let btn_color = if start_hover || !self.is_ready() { YELLOW } else { Color::from_rgba(80, 80, 80, 255) };
        let btn_bg = if !self.is_ready() { Color::from_rgba(100, 50, 50, 255) } else { Color::from_rgba(50, 100, 50, 255) };
        
        draw_rectangle(section_x, start_y, 180.0, 40.0, btn_bg);
        draw_rectangle_lines(section_x, start_y, 180.0, 40.0, 2.0, btn_color);
        draw_text(
            "[ENTER] Start",
            section_x + 20.0,
            start_y + 27.0,
            18.0,
            btn_color
        );
    }

    fn draw_config_item(&self, index: usize, text: &str, y: f32) {
        let is_selected = index == self.config_selected;
        let is_hovered = is_mouse_over(400.0, y, 400.0, 35.0);
        
        // Background
        if is_hovered || is_selected {
            let bg_color = if is_selected {
                Color::from_rgba(50, 50, 100, 255)
            } else {
                Color::from_rgba(40, 40, 40, 255)
            };
            draw_rectangle(400.0, y - 2.0, 400.0, 35.0, bg_color);
        }

        let prefix = if is_selected { "> " } else { "  " };
        let text_color = if is_selected { YELLOW } else { WHITE };
        
        // Draw left arrow button
        let left_hover = is_mouse_over(410.0, y + 5.0, 30.0, 25.0);
        draw_rectangle_lines(410.0, y + 5.0, 30.0, 25.0, 1.0, if left_hover { YELLOW } else { Color::from_rgba(60, 60, 60, 255) });
        draw_text("<", 420.0, y + 20.0, 16.0, if left_hover { YELLOW } else { WHITE });
        
        // Main text in the middle
        draw_text(
            &format!("{}{}", prefix, text),
            450.0,
            y + 20.0,
            18.0,
            text_color,
        );
        
        // Draw right arrow button
        let right_hover = is_mouse_over(760.0, y + 5.0, 30.0, 25.0);
        draw_rectangle_lines(760.0, y + 5.0, 30.0, 25.0, 1.0, if right_hover { YELLOW } else { Color::from_rgba(60, 60, 60, 255) });
        draw_text(">", 770.0, y + 20.0, 16.0, if right_hover { YELLOW } else { WHITE });
    }

    fn build_game(&self) -> Game {
        use macroquad::rand::gen_range;

        let mut players: Vec<Player> = vec![];

        let margin = 50.0;
        let min_distance = 80.0;
        for c in &self.configs {
            let mut pos;

            // try until we find a non-colliding position
            loop {
                pos = vec2(
                    gen_range(margin, SCREEN_W - margin),
                    gen_range(margin, SCREEN_H - margin),
                );

                if players.iter().all(|p| p.pos.distance(pos) > min_distance) {
                    break;
                }
            }
            let dir = gen_range(0.0, std::f32::consts::PI * 2.0);

            players.push(Player::new(pos, dir, c.color));
        }

        let inputs = self.configs.iter().map(|c| {
            PlayerInput {
                left: c.left.unwrap(),
                right: c.right.unwrap(),
            }
        }).collect();

        Game {
            players,
            inputs,
            scores: vec![0; self.configs.len()],
            round_state: RoundState::Countdown { timer: 3.0 },
            config: self.game_config.clone(),
            powerups: vec![],
            spawn_timer: 0.0,
        }
    }
}

// ================== COLLISION ==================

fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    if ab.length_squared() == 0.0 { return p.distance(a); }
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance(closest)
}

fn check_collision(players: &mut [Player]) {
    for i in 0..players.len() {
        if !players[i].alive { continue; }

        let p = players[i].pos;
        if p.x < 0.0 || p.x > SCREEN_W || p.y < 0.0 || p.y > SCREEN_H {
            let deaths_so_far = players.iter().filter(|pl| !pl.alive).count();
            players[i].death_order = Some(deaths_so_far + 1);
            players[i].alive = false;
            continue;
        }

        for j in 0..players.len() {
            let trail = &players[j].trail;
            let len = trail.len();

            for k in 1..len {
                if i == j && k > len.saturating_sub(SELF_GRACE_POINTS) {
                    continue;
                }

                if let (Some(a), Some(b)) = (trail[k - 1], trail[k]) {
                    if distance_to_segment(players[i].pos, a, b) < COLLISION_RADIUS {
                        let deaths_so_far = players.iter().filter(|pl| !pl.alive).count();
                        players[i].death_order = Some(deaths_so_far + 1);
                        players[i].alive = false;
                        break;
                    }
                }
            }

            if !players[i].alive { break; }
        }
    }
}

// ================== APP ==================

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