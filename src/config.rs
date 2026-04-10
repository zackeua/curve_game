use macroquad::prelude::*;

// ================== CONSTANTS ==================

pub const UI_WIDTH: f32 = 200.0;
pub const SCREEN_W: f32 = 800.0;
pub const SCREEN_H: f32 = 600.0;

pub const WINDOW_W: f32 = SCREEN_W + UI_WIDTH;
pub const WINDOW_H: f32 = SCREEN_H;

pub const SPEED: f32 = 120.0;
pub const TURN_SPEED: f32 = 4.0;
pub const TRAIL_STEP: f32 = 5.0;
pub const COLLISION_RADIUS: f32 = 3.0;
pub const SELF_GRACE_POINTS: usize = 15;

pub const COLORS: [Color; 6] = [RED, BLUE, GREEN, YELLOW, ORANGE, PINK];
pub const COLOR_PALETTE: &[(f32, f32, f32); 25] = &[
    // Bright primary and secondary colors
    (1.0, 0.0, 0.0), (0.0, 0.0, 1.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0), (1.0, 0.0, 1.0),
    // Orange, cyan, and other bright colors
    (1.0, 0.5, 0.0), (0.0, 1.0, 1.0), (1.0, 0.0, 0.5), (0.5, 1.0, 0.0), (0.0, 0.5, 1.0),
    // Darker shades
    (0.7, 0.0, 0.0), (0.0, 0.0, 0.7), (0.0, 0.7, 0.0), (0.7, 0.7, 0.0), (0.7, 0.0, 0.7),
    // Pastel/lighter shades
    (1.0, 0.6, 0.6), (0.6, 0.6, 1.0), (0.6, 1.0, 0.6), (1.0, 1.0, 0.6), (1.0, 0.6, 1.0),
    // Additional distinct colors
    (0.8, 0.4, 0.0), (0.0, 0.8, 0.8), (1.0, 0.2, 0.6), (0.4, 0.8, 0.0), (0.2, 0.4, 0.8),
];

// ================== GAME CONFIG ==================

#[derive(Clone)]
pub struct GameConfig {
    pub speed: f32,
    pub turn_speed: f32,
    pub hole_interval_min: f32,
    pub hole_interval_max: f32,
    pub hole_duration: f32,
    pub target_score: u32,
    pub powerups_enabled: bool,
}
