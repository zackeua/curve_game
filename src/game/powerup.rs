use macroquad::prelude::Vec2;
use super::player::Player;
use crate::config::GameConfig;

#[derive(Clone, Copy)]
pub enum PowerupType {
    SpeedSelf,
    SpeedOthers,
    SlowSelf,
    SlowOthers,
    ThickenTrail,
}

pub struct Powerup {
    pub pos: Vec2,
    pub kind: PowerupType,
}

pub fn apply_powerup(
    player_idx: usize,
    kind: PowerupType,
    players: &mut [Player],
    death_orders: &[Option<usize>],
    _config: &mut GameConfig,
) {
    match kind {
        PowerupType::SpeedSelf => {
            players[player_idx].speed_multiplier = 1.5;
            players[player_idx].effect_timer = 5.0;
        }
        PowerupType::SpeedOthers => {
            for (i, p) in players.iter_mut().enumerate() {
                let player_is_alive = death_orders[i].is_none();
                if i != player_idx && player_is_alive {
                    p.speed_multiplier = 1.5;
                    p.effect_timer = 5.0;
                }
            }
        }
        PowerupType::SlowOthers => {
            for (i, p) in players.iter_mut().enumerate() {
                let player_is_alive = death_orders[i].is_none();
                if i != player_idx && player_is_alive {
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