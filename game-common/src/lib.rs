//! game-common – shared protocol, math utilities, and data structures.
//!
//! This crate is a `no_std`-compatible core that is consumed by both
//! `game-server` and `game-client`.  All types that cross the network wire
//! derive [`serde::Serialize`] / [`serde::Deserialize`] and are serialised
//! with [`bincode`] for compact, deterministic encoding.

pub mod math;
pub mod protocol;

// Re-export the most frequently used types at the crate root so dependants
// can write `use game_common::StateSnapshot` instead of the full path.
pub use math::{lerp, lerp_vec2, segment_circle_intersect};
pub use protocol::{EntityState, InputPacket, StateSnapshot};
