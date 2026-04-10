use macroquad::prelude::*;

#[derive(Clone)]
pub struct Player {
    pub pos: Vec2,
    pub dir: f32,
    pub trail: Vec<Option<Vec2>>,

    pub hole_timer: f32,
    pub hole_cooldown: f32,
    pub in_hole: bool,

    pub speed_multiplier: f32,
    pub turn_multiplier: f32,
    pub effect_timer: f32,
    pub trail_thickness: f32,
}

impl Player {
    pub fn new(pos: Vec2, dir: f32) -> Self {
        Self {
            pos,
            dir,
            trail: vec![Some(pos)],
            hole_timer: 0.0,
            hole_cooldown: macroquad::rand::gen_range(1.5, 3.0),
            in_hole: false,
            speed_multiplier: 1.0,
            turn_multiplier: 1.0,
            effect_timer: 0.0,
            trail_thickness: 3.0,
        }
    }

    pub fn get_direction_vector(&self) -> Vec2 {
        vec2(self.dir.cos(), self.dir.sin())
    }

    pub fn update(&mut self, dt: f32, turn: f32, config: &crate::GameConfig) {
        const TRAIL_STEP: f32 = 5.0;
        const SPEED: f32 = 120.0;

        self.dir += turn * config.turn_speed * dt;
        self.hole_timer += dt;

        let velocity = self.get_direction_vector() * config.speed * dt;
        self.pos += velocity * self.speed_multiplier;

        if self.in_hole && self.hole_timer > config.hole_duration {
            self.in_hole = false;
            self.hole_timer = 0.0;
            self.hole_cooldown =
                macroquad::rand::gen_range(config.hole_interval_min, config.hole_interval_max);
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

    pub fn reset(&mut self, pos: Vec2, dir: f32) {
        self.pos = pos;
        self.dir = dir;
        self.trail.clear();
        self.trail.push(Some(pos));
        self.hole_timer = 0.0;
        self.in_hole = false;
        self.speed_multiplier = 1.0;
        self.turn_multiplier = 1.0;
        self.effect_timer = 0.0;
        self.trail_thickness = 3.0;
    }
}
