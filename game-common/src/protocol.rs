//! Network protocol types skeleton.

use glam::Vec2;
use netcode_core::Timestamped;
use serde::{Deserialize, Serialize};

// ── Client → Server ───────────────────────────────────────────────────────────

/// A single sampled frame of player input sent from client to server.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InputPacket {
    pub sequence: u32,
    pub timestamp_ms: u64,
    pub movement: Vec2,
    pub fire: bool,
    pub aim_angle: f32,
}

impl InputPacket {
    #[inline]
    pub fn is_newer_than(&self, _other: &Self) -> bool {
        todo!("Delegate to netcode_core::is_sequence_newer")
    }
}

// ── Server → Client ───────────────────────────────────────────────────────────

/// Authoritative state for one entity at a given server tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntityState {
    pub id: u32,
    pub position: Vec2,
    pub rotation: f32,
    pub health: u8,
}

/// A complete authoritative world snapshot broadcast by the server each tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub timestamp_ms: u64,
    pub last_processed_seq: u32,
    pub entities: Vec<EntityState>,
}

impl Timestamped for StateSnapshot {
    fn timestamp_ms(&self) -> u64 {
        todo!("Return self.timestamp_ms")
    }
}

impl StateSnapshot {
    pub fn find_entity(&self, _id: u32) -> Option<&EntityState> {
        todo!("Binary search entities by id and return matching EntityState reference")
    }
}
