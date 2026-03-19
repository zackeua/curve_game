use macroquad::prelude::*;
use macroquad::rand::srand;
// ================== CONSTANTS ==================

const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

const SPEED: f32 = 120.0;
const TURN_SPEED: f32 = 4.0;
const TRAIL_STEP: f32 = 5.0;
const COLLISION_RADIUS: f32 = 3.0;
const SELF_GRACE_POINTS: usize = 15;

const COLORS: [Color; 6] = [RED, BLUE, GREEN, YELLOW, ORANGE, PINK];

// ================== GAME CORE ==================

#[derive(Clone)]
struct Player {
    pos: Vec2,
    dir: f32,
    color: Color,
    alive: bool,
    trail: Vec<Vec2>,
}

impl Player {
    fn new(pos: Vec2, dir: f32, color: Color) -> Self {
        Self {
            pos,
            dir,
            color,
            alive: true,
            trail: vec![pos],
        }
    }

    fn update(&mut self, dt: f32, turn: f32) {
        if !self.alive { return; }

        self.dir += turn * TURN_SPEED * dt;

        let velocity = vec2(self.dir.cos(), self.dir.sin()) * SPEED * dt;
        self.pos += velocity;

        if self.trail.last().unwrap().distance(self.pos) > TRAIL_STEP {
            self.trail.push(self.pos);
        }
    }

    fn draw(&self) {
        for i in 1..self.trail.len() {
            let a = self.trail[i - 1];
            let b = self.trail[i];
            draw_line(a.x, a.y, b.x, b.y, 3.0, self.color);
        }
        draw_circle(self.pos.x, self.pos.y, 4.0, self.color);
    }
}

struct PlayerInput {
    left: KeyCode,
    right: KeyCode,
}

enum RoundState {
    Playing,
    RoundOver { winner: Option<usize> },
}

struct Game {
    players: Vec<Player>,
    inputs: Vec<PlayerInput>,
    scores: Vec<u32>,
    round_state: RoundState,
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
        if let RoundState::Playing = self.round_state {
            for (p, input) in self.players.iter_mut().zip(self.inputs.iter()) {
                let mut turn = 0.0;

                if is_key_down(input.left) {
                    turn -= 1.0;
                }
                if is_key_down(input.right) {
                    turn += 1.0;
                }

                p.update(dt, turn);
            }

            check_collision(&mut self.players);

            // COunt alivbe players
            let alive: Vec<usize> = self.players.iter()
                .enumerate()
                .filter(|(_, p)| p.alive)
                .map(|(i, _)| i)
                .collect();

            if alive.len() <= 1 {
                let winner = alive.first().cloned();

                if let Some(w) = winner {
                    self.scores[w] += 1;
                }

                self.round_state = RoundState::RoundOver { winner };
            }

        }
    }

    fn draw(&self) {
        draw_border();

        for p in &self.players {
            p.draw();
        }

        // Scores
        for (i, score) in self.scores.iter().enumerate() {
            draw_text(
                &format!("P{}: {}", i + 1, score),
                 20.0,
                 30.0 + i as f32 * 25.0,
                 25.0,
                 self.players[i].color
            );
        }

        // Round results
        if let RoundState::RoundOver { winner } = self.round_state {
            let text = match winner {
                Some(i) => format!("Player {} wins! Press SPACE to continue", i + 1),
                None => "It's a tie! Press SPACE to continue".to_string(),
            };

            draw_text(&text, 200.0, 50.0, 30.0, YELLOW);
        }
    }

    fn restart_round(&mut self) {
        use macroquad::rand::gen_range;

        let margin = 50.0;

        for p in &mut self.players {
            p.pos = vec2(
                gen_range(margin, SCREEN_W - margin),
                gen_range(margin, SCREEN_H - margin),
            );
            p.dir = gen_range(0.0, std::f32::consts::PI * 2.0);
            p.trail.clear();
            p.trail.push(p.pos);
            p.alive = true;
        }


        self.round_state = RoundState::Playing;
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

        // Handle key input
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
            round_state: RoundState::Playing,
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

                if distance_to_segment(players[i].pos, trail[k - 1], trail[k]) < COLLISION_RADIUS {
                    players[i].alive = false;
                    break;
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

#[macroquad::main("Curve Fever Clone")]
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

                if let RoundState::RoundOver { .. }  = game.round_state {
                    if is_key_pressed(KeyCode::Space) {
                        game.restart_round();
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::Menu(Menu::new());
                }
            }
        }

        next_frame().await;
    }
}