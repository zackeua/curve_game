use macroquad::prelude::*;

use crate::config::{SCREEN_W, SCREEN_H, COLORS, COLOR_PALETTE, SPEED, TURN_SPEED, GameConfig};
use crate::game::{Game, Player, PlayerInput, RoundState};

#[derive(Clone)]
pub struct PlayerConfig {
    pub left: Option<KeyCode>,
    pub right: Option<KeyCode>,
    pub color: Color,
}

pub enum BindingState {
    None,
    Left(usize),
    Right(usize),
}

pub struct Menu {
    pub configs: Vec<PlayerConfig>,
    pub selected: usize,
    pub binding: BindingState,
    
    pub game_config: GameConfig,
    pub config_selected: usize,
    
    pub mouse_x: f32,
    pub mouse_y: f32,
    
    pub color_picker_open: Option<usize>,
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
    pub fn new() -> Self {
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
            color_picker_open: None,
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

    pub fn is_ready(&self) -> bool {
        !self.configs.is_empty()
            && self.configs.iter().all(|p| p.left.is_some() && p.right.is_some())
    }

    fn add_player(&mut self) {
        let color = self.next_free_color();
        self.configs.push(PlayerConfig {
            left: None,
            right: None,
            color,
        });
    }

    pub fn update(&mut self) {
        // Update mouse position
        let (mx, my) = mouse_position();
        self.mouse_x = mx;
        self.mouse_y = my;

        // Handle color picker input (takes priority)
        if self.handle_color_picker() {
            return;
        }

        // Handle key binding input (takes priority)
        if self.handle_key_binding() {
            return;
        }

        self.handle_player_management();
        self.handle_config_adjustment();
    }

    fn handle_color_picker(&mut self) -> bool {
        if let Some(player_idx) = self.color_picker_open {
            // Close color picker on Escape or clicking outside
            if is_key_pressed(KeyCode::Escape) {
                self.color_picker_open = None;
                return true;
            }

            // Check for clicks on the color palette
            for row in 0..5 {
                for col in 0..5 {
                    let x = 450.0 + col as f32 * 50.0;
                    let y = 200.0 + row as f32 * 40.0;
                    
                    if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(x, y, 40.0, 30.0) {
                        let idx = row * 5 + col;
                        if idx < COLOR_PALETTE.len() {
                            let (r, g, b) = COLOR_PALETTE[idx];
                            self.configs[player_idx].color = Color::new(r, g, b, 1.0);
                            self.color_picker_open = None;
                        }
                    }
                }
            }
            return true;
        }
        false
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

        // Change color - open color picker
        if (is_key_pressed(KeyCode::C)) 
            || (is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(200.0, buttons_y, 170.0, 30.0)) {
            self.color_picker_open = Some(self.selected);
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

    pub fn should_start_game(&self) -> bool {
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

    pub fn draw(&self) {
        // Background panels
        draw_rectangle(10.0, 50.0, 370.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(10.0, 50.0, 370.0, 530.0, 2.0, Color::from_rgba(100, 100, 100, 255));
        
        draw_rectangle(390.0, 50.0, 420.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(390.0, 50.0, 420.0, 530.0, 2.0, Color::from_rgba(100, 100, 100, 255));

        // Title
        draw_text("ZACHTUNG!", 20.0, 80.0, 40.0, YELLOW);

        if let Some(player_idx) = self.color_picker_open {
            self.draw_color_picker(player_idx);
        } else {
            self.draw_player_section();
            self.draw_config_section();
        }
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
        draw_text("Select Player (UP/DOWN):", section_x, section_y + 85.0, 18.0, WHITE);

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
        match &self.binding {
            BindingState::Left(i) => {
                draw_rectangle(section_x, 520.0, 350.0, 50.0, Color::from_rgba(50, 0, 0, 255));
                draw_text(
                    &format!("P{}: Press LEFT key", i),
                    section_x + 10.0,
                    545.0,
                    20.0,
                    YELLOW,
                );
            }
            BindingState::Right(i) => {
                draw_rectangle(section_x, 520.0, 350.0, 50.0, Color::from_rgba(50, 0, 0, 255));
                draw_text(
                    &format!("P{}: Press RIGHT key", i),
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
        let text_color = config.color;
        
        draw_text(
            &format!(
                "{}P{} | L:{} R:{}",
                prefix,
                index,
                key_to_string(config.left),
                key_to_string(config.right)
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
        
        draw_text("Click setting or use (U) to adjust", section_x, section_y + 40.0, 14.0, Color::from_rgba(150, 150, 150, 255));
        
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

    fn draw_color_picker(&self, player_idx: usize) {
        let title_y = 100.0;
        draw_text(&format!("Pick color for P{}", player_idx), 420.0, title_y, 24.0, YELLOW);
        draw_text("Click color or press ESC to cancel", 420.0, title_y + 35.0, 14.0, Color::from_rgba(150, 150, 150, 255));

        // Draw color palette (5x5 grid)
        for row in 0..5 {
            for col in 0..5 {
                let idx = row * 5 + col;
                if idx < COLOR_PALETTE.len() {
                    let x = 450.0 + col as f32 * 50.0;
                    let y = 200.0 + row as f32 * 40.0;
                    
                    let (r, g, b) = COLOR_PALETTE[idx];
                    let color = Color::new(r, g, b, 1.0);
                    let is_hovered = is_mouse_over(x, y, 40.0, 30.0);
                    
                    // Draw color box
                    draw_rectangle(x, y, 40.0, 30.0, color);
                    
                    // Highlight on hover
                    if is_hovered {
                        draw_rectangle_lines(x, y, 40.0, 30.0, 3.0, YELLOW);
                    } else {
                        draw_rectangle_lines(x, y, 40.0, 30.0, 1.0, WHITE);
                    }
                }
            }
        }
    }

    pub fn build_game(&self) -> Game {
        use macroquad::rand::gen_range;

        let mut players: Vec<Player> = vec![];
        let mut colors: Vec<Color> = vec![];

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

            players.push(Player::new(pos, dir));
            colors.push(c.color);
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
            colors,
            death_orders: vec![None; self.configs.len()],
            scores: vec![0; self.configs.len()],
            round_state: RoundState::Countdown { timer: 3.0 },
            config: self.game_config.clone(),
            powerups: vec![],
            spawn_timer: 0.0,
        }
    }
}
