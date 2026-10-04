//! Remote entity snapshot interpolation skeleton.

use std::collections::HashMap;

use game_common::StateSnapshot;
use glam::Vec2;

pub const INTERP_DELAY_MS: u64 = 100;

pub struct InterpolationBuffer;

impl InterpolationBuffer {
    pub fn new() -> Self {
        Self
    }

    pub fn push_snapshot(&mut self, _snap: &StateSnapshot) {
        todo!("Insert snapshot into sorted buffer, dropping duplicates/out-of-order packets and pruning old entries")
    }

    pub fn interpolated_positions(&self, _now_ms: u64) -> HashMap<u32, Vec2> {
        todo!("Calculate render_time = now_ms - 100ms, find bracketing snapshots, LERP positions for each remote entity")
    }
}

impl Default for InterpolationBuffer {
    fn default() -> Self {
        Self::new()
    }
}
