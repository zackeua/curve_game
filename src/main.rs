use macroquad::prelude::*;

const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

const SPEED: f32 = 120.0;      // pixels/sec
const TURN_SPEED: f32 = 4.0;   // radians/sec
const TRAIL_STEP: f32 = 5.0;   // distance between trail points
const COLLISION_RADIUS: f32 = 3.0; // collision distance
const SELF_GRACE_POINTS: usize = 15; // points to ignore on own trail

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

    fn update(&mut self, dt: f32, turn_left: KeyCode, turn_right: KeyCode) {
        if !self.alive { return; }

        if is_key_down(turn_left) {
            self.dir -= TURN_SPEED * dt;
        }
        if is_key_down(turn_right) {
            self.dir += TURN_SPEED * dt;
        }

        let velocity = vec2(self.dir.cos(), self.dir.sin()) * SPEED * dt;
        self.pos += velocity;

        // Add trail point only if far enough
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

// Distance from a point to a line segment
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

        // Wall collision
        let p = players[i].pos;
        if p.x < 0.0 || p.x > SCREEN_W || p.y < 0.0 || p.y > SCREEN_H {
            players[i].alive = false;
            continue;
        }

        // Trail collision
        for j in 0..players.len() {
            let trail = &players[j].trail;
            let len = trail.len();

            for k in 1..len {
                // Skip recent points on self trail
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

#[macroquad::main("Curve Fever Clone")]
async fn main() {
    let mut players = vec![
        Player::new(vec2(200.0, 300.0), 0.0, RED),
        Player::new(vec2(600.0, 300.0), std::f32::consts::PI, BLUE),
    ];

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        // Update players
        players[0].update(dt, KeyCode::A, KeyCode::D);
        players[1].update(dt, KeyCode::Left, KeyCode::Right);

        // Collision check
        check_collision(&mut players);

        // Draw players
        for p in &players { p.draw(); }

        // Restart
        if is_key_pressed(KeyCode::R) {
            players = vec![
                Player::new(vec2(200.0, 300.0), 0.0, RED),
                Player::new(vec2(600.0, 300.0), std::f32::consts::PI, BLUE),
            ];
        }

        // Simple UI
        draw_text("Player 1: A/D", 10.0, 20.0, 20.0, WHITE);
        draw_text("Player 2: ←/→", 10.0, 40.0, 20.0, WHITE);
        draw_text("Press R to restart", 10.0, 60.0, 20.0, WHITE);

        next_frame().await;
    }
}