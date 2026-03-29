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
    }
}

fn apply_powerup(player_idx: usize, kind: PowerupType, players: &mut [Player], config: &mut GameConfig) {
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

                if let Some(w) = winner {
                    self.scores[w] += 1;

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
                target_score: 5,
                powerups_enabled: true,
            },
            config_selected: 0,
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

    fn update(&mut self) {

        // Ignore input if we're currently binding keys
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
            return;
        }

        if is_key_pressed(KeyCode::U) {
            self.config_selected = (self.config_selected + 1) % 7;
        }

        // Add player
        if is_key_pressed(KeyCode::N) {
            let color = self.next_free_color();
            self.configs.push(PlayerConfig {
                left: None,
                right: None,
                color,
            });
        }

        // Select player
        if is_key_pressed(KeyCode::Up) && self.selected > 0 {
            self.selected -= 1;
        }

        if is_key_pressed(KeyCode::Down) && self.selected + 1 < self.configs.len() {
            self.selected += 1;
        }

        // Start binding
        if is_key_pressed(KeyCode::Space) && !self.configs.is_empty() {
            self.binding = BindingState::Left(self.selected);
        }

        if is_key_pressed(KeyCode::Left) && self.game_config.target_score > 1 {
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

        if is_key_pressed(KeyCode::Right) && self.game_config.target_score < 99 {
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


        // Change color
        if is_key_pressed(KeyCode::C) && !self.configs.is_empty() {
            let selected = self.selected;

    let current_color = self.configs[selected].color;

    // Colors used by OTHER players
    let used: Vec<Color> = self.configs.iter()
        .enumerate()
        .filter(|(i, _)| *i != selected)
        .map(|(_, p)| p.color)
        .collect();

    // Find current index in color list
    let mut idx = COLORS.iter()
        .position(|&c| c == current_color)
        .unwrap_or(0);

    // Try next colors (wrap around)
    for _ in 0..COLORS.len() {
        idx = (idx + 1) % COLORS.len();
        let candidate = COLORS[idx];

        if !used.contains(&candidate) {
            self.configs[selected].color = candidate;
            break;
        }
    }
        }
    }

    fn draw(&self) {
        draw_text("MENU", 20.0, 40.0, 40.0, WHITE);
        draw_text("N = Add player", 20.0, 80.0, 25.0, WHITE);
        draw_text("up/down = Select player", 20.0, 110.0, 25.0, WHITE);
        draw_text("SPACE = Bind keys", 20.0, 140.0, 25.0, WHITE);
        draw_text("C = Change color", 20.0, 170.0, 25.0, WHITE);
        draw_text("ENTER = Start", 20.0, 200.0, 25.0, WHITE);

        for (i, p) in self.configs.iter().enumerate() {
            let y = 260.0 + i as f32 * 40.0;


            let is_selected = i == self.selected;
            let prefix = if is_selected { ">" } else { " " };
            let suffix = if is_selected { "<" } else { " " };

            draw_text(
                &format!(
                    "{} P{} | Left: {} Right: {} {}",
                    prefix,
                    i + 1,
                    key_to_string(p.left),
                    key_to_string(p.right),
                    suffix
                ),
                20.0,
                y,
                25.0,
                p.color,
            );
        }

        match self.binding {
            BindingState::Left(i) => {
                draw_text(
                    &format!("Player {}: press LEFT key", i + 1),
                    20.0,
                    550.0,
                    30.0,
                    YELLOW,
                );
            }
            BindingState::Right(i) => {
                draw_text(
                    &format!("Player {}: press RIGHT key", i + 1),
                    20.0,
                    550.0,
                    30.0,
                    YELLOW,
                );
            }
            _ => {}
        }

        let base_y = 400.0;
        let items = [
            format!("Speed: {:.0}", self.game_config.speed),
            format!("Turn Speed: {:.1}", self.game_config.turn_speed),
            format!("Hole Size: {:.1}", self.game_config.hole_duration),
            format!("Hole min Interval: {:.1}", self.game_config.hole_interval_min),
            format!("Hole max Interval: {:.1}", self.game_config.hole_interval_max),
            format!("Target Score: {}", self.game_config.target_score),
            format!("Powerups: {}", if self.game_config.powerups_enabled { "ON" } else { "OFF" }),
        ];


        draw_text("GAME CONFIG", 400.0, 340.0, 30.0, WHITE);
        draw_text("Use U to select setting, Left/Right to change", 400.0, 370.0, 20.0, WHITE);
        for (i, text) in items.iter().enumerate() {
            let prefix = if i == self.config_selected { ">" } else { " " };

            draw_text(
                &format!("{} {}", prefix, text),
                400.0,
                base_y + i as f32 * 30.0,
                25.0,
                WHITE,
            );
        }
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

                if is_key_pressed(KeyCode::Enter) && menu.is_ready() {
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