//! Client prediction & server reconciliation skeleton.

use std::collections::VecDeque;

use game_common::{InputPacket, StateSnapshot};
use glam::Vec2;

#[derive(Clone)]
struct InputRecord {
    packet: InputPacket,
    dt: f32,
}

pub struct PredictionEngine {
    pub local_id: Option<u32>,
    pub predicted_position: Vec2,
    history: VecDeque<InputRecord>,
    sequence: u32,
}

impl PredictionEngine {
    pub fn new() -> Self {
        Self {
            local_id: None,
            predicted_position: Vec2::ZERO,
            history: VecDeque::with_capacity(128),
            sequence: 0,
        }
    }

    pub fn apply_input(
        &mut self,
        _movement: Vec2,
        _fire: bool,
        _aim_angle: f32,
        _dt: f32,
    ) -> InputPacket {
        todo!("Advance sequence, construct InputPacket, predict position locally, record in history, return packet")
    }

    pub fn reconcile(&mut self, _snapshot: &StateSnapshot) {
        todo!("1. Purge acknowledged inputs (seq <= last_processed_seq)\n2. Reset predicted position to authoritative position\n3. Re-simulate unacknowledged inputs")
    }

    fn integrate(&mut self, _movement: Vec2, _dt: f32) {
        todo!("Integrate movement into predicted_position")
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}
