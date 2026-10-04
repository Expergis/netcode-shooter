//! Lag-compensated shooter hit detection adapter skeleton.

use game_common::StateSnapshot;
use glam::Vec2;
use netcode_core::RingBuffer;

pub const PLAYER_RADIUS: f32 = 0.4;
pub const MAX_REWIND_MS: u64 = 1_000;

/// Shooter domain wrapper over `netcode_core::TimeRewindEngine`.
pub struct ShooterRewindAdapter;

impl ShooterRewindAdapter {
    /// Rewinds snapshot state and performs segment-circle hit detection.
    pub fn rewind_and_cast(
        _ring_buffer: &RingBuffer<StateSnapshot, 128>,
        _shot_timestamp: u64,
        _ray_origin: Vec2,
        _ray_dir: Vec2,
    ) -> Option<u32> {
        todo!("Invoke netcode_core::TimeRewindEngine::rewind_and_query with shooter entity interpolation and segment_circle_intersect query")
    }
}
