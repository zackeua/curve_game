use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use serde::{Deserialize, Serialize};

use crate::config::{COLOR_PALETTE, COLORS, GameConfig, SCREEN_H, SCREEN_W, SPEED, TURN_SPEED};
use crate::game::{Game, Player, PlayerInput, RoundState};

const SAVE_KEY: &str = "curve_game_menu";

#[derive(Clone, Deserialize, Serialize)]
pub struct PlayerConfig {
    pub left: Option<String>,
    pub right: Option<String>,
    pub color: (f32, f32, f32),
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
    start_requested: bool,
    config_inputs: [String; 6],
}

fn button(x: f32, y: f32, width: f32, height: f32, label: &str) -> bool {
    widgets::Button::new(label)
        .position(vec2(x, y))
        .size(vec2(width, height))
        .ui(&mut root_ui())
}

fn remove_button(x: f32, y: f32, width: f32, height: f32) -> bool {
    let mut ui = root_ui();
    let mut skin = ui.default_skin();
    skin.button_style = ui
        .style_builder()
        .color(Color::from_rgba(85, 30, 30, 255))
        .color_hovered(Color::from_rgba(150, 45, 45, 255))
        .color_clicked(Color::from_rgba(190, 60, 60, 255))
        .text_color(WHITE)
        .text_color_hovered(WHITE)
        .text_color_clicked(WHITE)
        .font_size(14)
        .build();
    ui.push_skin(&skin);
    let clicked = widgets::Button::new("×")
        .position(vec2(x, y))
        .size(vec2(width, height))
        .ui(&mut ui);
    ui.pop_skin();
    clicked
}

#[derive(Serialize, Deserialize)]
pub struct MenuSave {
    pub players: Vec<PlayerConfig>,
    pub game_config: GameConfig,
}

fn is_mouse_over(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn keycode_to_binding(key: KeyCode) -> Option<String> {
    Some(
        match key {
            KeyCode::Left => "ArrowLeft",
            KeyCode::Right => "ArrowRight",
            KeyCode::Up => "ArrowUp",
            KeyCode::Down => "ArrowDown",
            KeyCode::Space => " ",
            KeyCode::Enter => "Enter",
            KeyCode::Escape => "Escape",
            KeyCode::Tab => "Tab",
            KeyCode::Backspace => "Backspace",
            KeyCode::LeftShift | KeyCode::RightShift => "Shift",
            KeyCode::LeftControl | KeyCode::RightControl => "Control",
            KeyCode::LeftAlt | KeyCode::RightAlt => "Alt",
            KeyCode::A => "a",
            KeyCode::B => "b",
            KeyCode::C => "c",
            KeyCode::D => "d",
            KeyCode::E => "e",
            KeyCode::F => "f",
            KeyCode::G => "g",
            KeyCode::H => "h",
            KeyCode::I => "i",
            KeyCode::J => "j",
            KeyCode::K => "k",
            KeyCode::L => "l",
            KeyCode::M => "m",
            KeyCode::N => "n",
            KeyCode::O => "o",
            KeyCode::P => "p",
            KeyCode::Q => "q",
            KeyCode::R => "r",
            KeyCode::S => "s",
            KeyCode::T => "t",
            KeyCode::U => "u",
            KeyCode::V => "v",
            KeyCode::W => "w",
            KeyCode::X => "x",
            KeyCode::Y => "y",
            KeyCode::Z => "z",
            _ => return None,
        }
        .to_string(),
    )
}

fn get_last_binding() -> Option<String> {
    get_char_pressed()
        .map(|character| character.to_string())
        .or_else(|| get_last_key_pressed().and_then(keycode_to_binding))
}

impl Menu {
    pub fn new() -> Self {
        let mut menu = Self {
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
            start_requested: false,
            config_inputs: [
                SPEED.to_string(),
                TURN_SPEED.to_string(),
                "0.3".to_string(),
                "1.5".to_string(),
                "3.0".to_string(),
                "20".to_string(),
            ],
        };
        menu.load_config();
        menu
    }

    fn key_in_use(&self, key: &str) -> bool {
        if self
            .configs
            .iter()
            .any(|p| p.left.as_deref() == Some(key) || p.right.as_deref() == Some(key))
        {
            return true;
        }

        matches!(
            key,
            "n" | "N" | " " | "c" | "C" | "Enter" | "Escape" | "Backspace"
        )
    }

    fn next_free_color(&self) -> (f32, f32, f32) {
        for &c in &COLORS {
            if !self.configs.iter().any(|p| p.color == (c.r, c.g, c.b)) {
                return (c.r, c.g, c.b);
            }
        }
        (1.0, 1.0, 1.0)
    }

    pub fn is_ready(&self) -> bool {
        !self.configs.is_empty()
            && self
                .configs
                .iter()
                .all(|p| p.left.is_some() && p.right.is_some())
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
        if self.color_picker_open.is_some() {
            // Close color picker on Escape or clicking outside
            if is_key_pressed(KeyCode::Escape) {
                self.color_picker_open = None;
                return true;
            }

            return true;
        }
        false
    }

    fn handle_key_binding(&mut self) -> bool {
        if !matches!(self.binding, BindingState::None) {
            if let Some(key) = get_last_binding() {
                if !self.key_in_use(&key) {
                    match self.binding {
                        BindingState::Left(i) => {
                            self.configs[i].left = Some(key);
                            self.binding = BindingState::Right(i);
                            self.save_config();
                        }
                        BindingState::Right(i) => {
                            self.configs[i].right = Some(key);
                            self.binding = BindingState::None;
                            self.save_config();
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
        if is_key_pressed(KeyCode::N) {
            self.add_player();
            self.save_config();
        }

        // Select player with keyboard
        if is_key_pressed(KeyCode::Up) && self.selected > 0 {
            self.selected -= 1;
        }
        if is_key_pressed(KeyCode::Down) && self.selected + 1 < self.configs.len() {
            self.selected += 1;
        }

        // Skip remaining player actions if no players
        if self.configs.is_empty() {
            return;
        }

        // Bind keys
        if is_key_pressed(KeyCode::Space) {
            self.binding = BindingState::Left(self.selected);
        }

        // Change color - open color picker
        if is_key_pressed(KeyCode::C) {
            self.color_picker_open = Some(self.selected);
        }
    }

    fn handle_config_adjustment(&mut self) {
        // Keyboard: cycle through config items
        if is_key_pressed(KeyCode::U) {
            self.config_selected = (self.config_selected + 1) % 7;
            self.save_config();
        }

        // Keyboard: adjust selected config
        if is_key_pressed(KeyCode::Left) && self.game_config.target_score > 1 {
            self.adjust_config_left();
            self.sync_config_input(self.config_selected.min(5));
            self.save_config();
        }
        if is_key_pressed(KeyCode::Right) && self.game_config.target_score < 99 {
            self.adjust_config_right();
            self.sync_config_input(self.config_selected.min(5));
            self.save_config();
        }
    }

    pub fn should_start_game(&self) -> bool {
        self.start_requested
    }

    fn adjust_config_left(&mut self) {
        match self.config_selected {
            0 => self.game_config.speed = (self.game_config.speed - 10.0).max(50.0),
            1 => self.game_config.turn_speed = (self.game_config.turn_speed - 0.5).max(1.0),
            2 => self.game_config.hole_duration = (self.game_config.hole_duration - 0.1).max(0.1),
            3 => {
                self.game_config.hole_interval_min =
                    (self.game_config.hole_interval_min - 0.5).max(0.5);
            }
            4 => {
                self.game_config.hole_interval_max = (self.game_config.hole_interval_max - 0.5)
                    .max(self.game_config.hole_interval_min + 0.5);
            }
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
                self.game_config.hole_interval_min =
                    (self.game_config.hole_interval_min + 0.5).min(5.0);
            }
            4 => {
                self.game_config.hole_interval_max =
                    (self.game_config.hole_interval_max + 0.5).min(10.0);
            }
            5 => {
                self.game_config.target_score = (self.game_config.target_score + 1).min(99);
            }
            6 => {
                self.game_config.powerups_enabled = !self.game_config.powerups_enabled;
            }
            _ => {}
        }
    }

    fn sync_config_input(&mut self, index: usize) {
        self.config_inputs[index] = match index {
            0 => format!("{:.0}", self.game_config.speed),
            1 => format!("{:.1}", self.game_config.turn_speed),
            2 => format!("{:.1}", self.game_config.hole_duration),
            3 => format!("{:.1}", self.game_config.hole_interval_min),
            4 => format!("{:.1}", self.game_config.hole_interval_max),
            5 => self.game_config.target_score.to_string(),
            _ => String::new(),
        };
    }

    fn apply_config_input(&mut self, index: usize) {
        let Ok(value) = self.config_inputs[index].parse::<f32>() else {
            return;
        };

        match index {
            0 => self.game_config.speed = value.clamp(50.0, 400.0),
            1 => self.game_config.turn_speed = value.clamp(1.0, 10.0),
            2 => self.game_config.hole_duration = value.clamp(0.1, 2.0),
            3 => {
                self.game_config.hole_interval_min = value
                    .clamp(0.5, 5.0)
                    .min(self.game_config.hole_interval_max - 0.5)
            }
            4 => {
                self.game_config.hole_interval_max =
                    value.clamp(self.game_config.hole_interval_min + 0.5, 10.0)
            }
            5 => self.game_config.target_score = (value.round() as u32).clamp(1, 99),
            _ => {}
        }
    }

    pub fn draw(&mut self) {
        {
            let mut ui = root_ui();
            let mut skin = ui.default_skin();
            skin.button_style = ui
                .style_builder()
                .color(Color::from_rgba(35, 35, 35, 255))
                .color_hovered(Color::from_rgba(70, 70, 70, 255))
                .color_clicked(Color::from_rgba(100, 100, 100, 255))
                .text_color(WHITE)
                .text_color_hovered(YELLOW)
                .text_color_clicked(YELLOW)
                .font_size(16)
                .build();
            skin.editbox_style = ui
                .style_builder()
                .color(Color::from_rgba(25, 25, 25, 255))
                .color_hovered(Color::from_rgba(35, 35, 35, 255))
                .color_clicked(Color::from_rgba(35, 35, 35, 255))
                .color_selected(Color::from_rgba(65, 65, 105, 255))
                .text_color(WHITE)
                .font_size(16)
                .build();
            skin.window_style = ui
                .style_builder()
                .color(Color::from_rgba(0, 0, 0, 0))
                .build();
            ui.push_skin(&skin);
        }

        // Background panels
        draw_rectangle(10.0, 50.0, 370.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(
            10.0,
            50.0,
            370.0,
            530.0,
            2.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        draw_rectangle(390.0, 50.0, 420.0, 530.0, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(
            390.0,
            50.0,
            420.0,
            530.0,
            2.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        // Title
        draw_text("ZACHTUNG!", 20.0, 80.0, 40.0, YELLOW);

        if let Some(player_idx) = self.color_picker_open {
            self.draw_color_picker(player_idx);
        } else {
            self.draw_player_section();
            self.draw_config_section();
            self.draw_config_inputs();
        }

        root_ui().pop_skin();
    }

    fn draw_player_section(&mut self) {
        let section_x = 20.0;
        let section_y = 100.0;

        // Section header
        draw_text("PLAYERS", section_x, section_y, 28.0, WHITE);
        draw_line(
            section_x,
            section_y + 10.0,
            section_x + 150.0,
            section_y + 10.0,
            2.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        if button(section_x, section_y + 35.0, 120.0, 30.0, "Add Player") {
            self.add_player();
        }

        // Player list header
        draw_text(
            "Select Player (UP/DOWN):",
            section_x,
            section_y + 85.0,
            18.0,
            WHITE,
        );

        // Player list
        for i in 0..self.configs.len() {
            if i >= self.configs.len() {
                break;
            }
            let p = self.configs[i].clone();
            self.draw_player_item(i, &p, section_y + 110.0);
        }

        if !self.configs.is_empty() {
            let list_height = 40.0 * self.configs.len() as f32;
            let buttons_y = section_y + 120.0 + list_height;

            if button(section_x, buttons_y, 170.0, 30.0, "Bind Keys") {
                self.binding = BindingState::Left(self.selected);
            }
            if button(section_x + 180.0, buttons_y, 170.0, 30.0, "Change Color") {
                self.color_picker_open = Some(self.selected);
            }
        }

        // Key binding prompt
        match &self.binding {
            BindingState::Left(i) => {
                draw_rectangle(
                    section_x,
                    520.0,
                    350.0,
                    50.0,
                    Color::from_rgba(50, 0, 0, 255),
                );
                draw_text(
                    &format!("P{}: Press LEFT key", i),
                    section_x + 10.0,
                    545.0,
                    20.0,
                    YELLOW,
                );
            }
            BindingState::Right(i) => {
                draw_rectangle(
                    section_x,
                    520.0,
                    350.0,
                    50.0,
                    Color::from_rgba(50, 0, 0, 255),
                );
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

    fn draw_player_item(&mut self, index: usize, config: &PlayerConfig, base_y: f32) {
        let y = base_y + index as f32 * 40.0;
        let is_selected = index == self.selected;
        let is_hovered = is_mouse_over(20.0, y, 350.0, 35.0);

        if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(20.0, y, 285.0, 35.0) {
            self.selected = index;
        }

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
        let text_color = Color::new(config.color.0, config.color.1, config.color.2, 1.0);

        draw_text(
            &format!(
                "{}P{} | L:{} R:{}",
                prefix,
                index,
                config.left.as_deref().unwrap_or("-"),
                config.right.as_deref().unwrap_or("-"),
            ),
            30.0,
            y + 20.0,
            18.0,
            text_color,
        );

        // Color indicator circle
        draw_circle(
            330.0,
            y + 13.0,
            6.0,
            Color::new(config.color.0, config.color.1, config.color.2, 1.0),
        );

        if remove_button(340.0, y - 2.0, 30.0, 35.0) {
            self.configs.remove(index);
            if self.selected >= self.configs.len() && self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    fn draw_config_section(&mut self) {
        let section_x = 400.0;
        let section_y = 100.0;

        // Section header
        draw_text("GAME CONFIG", section_x, section_y, 28.0, WHITE);
        draw_line(
            section_x,
            section_y + 10.0,
            section_x + 200.0,
            section_y + 10.0,
            2.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        draw_text(
            "Click setting or use (U) to adjust",
            section_x,
            section_y + 40.0,
            14.0,
            Color::from_rgba(150, 150, 150, 255),
        );

        let items = [
            format!("Speed: {:.0}", self.game_config.speed),
            format!("Turn Speed: {:.1}", self.game_config.turn_speed),
            format!("Hole Size: {:.1}", self.game_config.hole_duration),
            format!("Hole min: {:.1}", self.game_config.hole_interval_min),
            format!("Hole max: {:.1}", self.game_config.hole_interval_max),
            format!("Target Score: {}", self.game_config.target_score),
            format!(
                "Powerups: {}",
                if self.game_config.powerups_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            ),
        ];

        let base_y = section_y + 70.0;
        for (i, text) in items.iter().enumerate() {
            self.draw_config_item(i, text, base_y + i as f32 * 50.0);
        }

        // Start button at the bottom
        let start_y = base_y + 360.0;
        if self.is_ready() && button(section_x, start_y, 180.0, 40.0, "Start") {
            self.start_requested = true;
        }
    }

    fn draw_config_item(&mut self, index: usize, text: &str, y: f32) {
        let is_selected = index == self.config_selected;
        let is_hovered = is_mouse_over(400.0, y, 400.0, 35.0);

        if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(400.0, y, 400.0, 35.0) {
            self.config_selected = index;
        }

        if button(410.0, y + 5.0, 30.0, 25.0, "<") {
            self.config_selected = index;
            self.adjust_config_left();
            if index < 6 {
                self.sync_config_input(index);
            }
        }
        if button(760.0, y + 5.0, 30.0, 25.0, ">") {
            self.config_selected = index;
            self.adjust_config_right();
            if index < 6 {
                self.sync_config_input(index);
            }
        }

        // Background
        if is_hovered || is_selected {
            let bg_color = if is_selected {
                Color::from_rgba(50, 50, 100, 255)
            } else {
                Color::from_rgba(40, 40, 40, 255)
            };
            draw_rectangle(400.0, y - 2.0, 400.0, 35.0, bg_color);
        }

        let prefix = if is_selected { ">" } else { " " };
        let text_color = if is_selected { YELLOW } else { WHITE };

        // Main text in the middle
        draw_text(prefix, 450.0, y + 20.0, 18.0, text_color);
        draw_text(
            text.split(':').next().unwrap_or(text),
            475.0,
            y + 20.0,
            18.0,
            text_color,
        );

        if index == 6
            && button(
                590.0,
                y,
                140.0,
                35.0,
                if self.game_config.powerups_enabled {
                    "ON"
                } else {
                    "OFF"
                },
            )
        {
            self.config_selected = index;
            self.adjust_config_right();
        }
    }

    fn draw_config_inputs(&mut self) {
        for index in 0..6 {
            let y = 170.0 + index as f32 * 50.0;
            root_ui().window(
                hash!("config-input-window", index),
                vec2(590.0, y),
                vec2(140.0, 35.0),
                |ui| {
                    widgets::InputText::new(hash!("config", index))
                        .size(vec2(140.0, 35.0))
                        .filter_numbers()
                        .ui(ui, &mut self.config_inputs[index]);
                },
            );
        }

        for index in 0..6 {
            self.apply_config_input(index);
        }
    }

    fn draw_color_picker(&mut self, player_idx: usize) {
        let title_y = 100.0;
        draw_text(
            &format!("Pick color for P{}", player_idx),
            420.0,
            title_y,
            24.0,
            YELLOW,
        );
        draw_text(
            "Click color or press ESC to cancel",
            420.0,
            title_y + 35.0,
            14.0,
            Color::from_rgba(150, 150, 150, 255),
        );

        // Draw color palette (5x5 grid)
        for row in 0..5 {
            for col in 0..5 {
                let idx = row * 5 + col;
                if idx < COLOR_PALETTE.len() {
                    let x = 450.0 + col as f32 * 50.0;
                    let y = 200.0 + row as f32 * 40.0;

                    let (r, g, b) = COLOR_PALETTE[idx];
                    let color = Color::new(r, g, b, 1.0);
                    if is_mouse_button_pressed(MouseButton::Left) && is_mouse_over(x, y, 40.0, 30.0)
                    {
                        self.configs[player_idx].color = (r, g, b);
                        self.color_picker_open = None;
                        self.save_config();
                    }

                    // Draw color box
                    draw_rectangle(x, y, 40.0, 30.0, color);

                    // Highlight on hover
                    if is_mouse_over(x, y, 40.0, 30.0) {
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
            colors.push(Color::new(c.color.0, c.color.1, c.color.2, 1.0));
        }

        let inputs = self
            .configs
            .iter()
            .map(|c| PlayerInput {
                left: c.left.clone().unwrap(),
                right: c.right.clone().unwrap(),
            })
            .collect();

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
            paused: false,
        }
    }

    fn save_config(&self) {
        let data = MenuSave {
            players: self.configs.clone(),
            game_config: self.game_config.clone(),
        };

        let json = serde_json::to_string(&data).unwrap();

        let storage = &mut quad_storage::STORAGE.lock().unwrap();
        storage.set(SAVE_KEY, &json);
    }

    fn load_config(&mut self) {
        let storage = &mut quad_storage::STORAGE.lock().unwrap();
        if let Some(json) = storage.get(SAVE_KEY) {
            if let Ok(data) = serde_json::from_str::<MenuSave>(&json) {
                self.configs = data.players;
                self.game_config = data.game_config;
            }
        }
    }
}
