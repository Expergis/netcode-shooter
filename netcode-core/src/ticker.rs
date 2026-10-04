//! High-frequency server tick rate driver utilities.

use std::time::Duration;

/// Configuration for server tick execution rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickRate {
    pub hz: u64,
}

impl TickRate {
    /// 128 Hz standard competitive multiplayer tick rate (7.8125 ms per tick).
    pub const HZ_128: Self = Self { hz: 128 };

    /// 64 Hz standard tick rate (15.625 ms per tick).
    pub const HZ_64: Self = Self { hz: 64 };

    /// Creates a custom tick rate in Hz.
    pub fn new(_hz: u64) -> Self {
        todo!("Construct TickRate ensuring hz > 0")
    }

    /// Returns the exact interval duration per tick.
    #[inline]
    pub fn tick_duration(&self) -> Duration {
        todo!("Calculate Duration::from_nanos(1_000_000_000 / self.hz)")
    }

    /// Returns tick duration in seconds as `f32`.
    #[inline]
    pub fn dt(&self) -> f32 {
        todo!("Return tick_duration().as_secs_f32()")
    }
}

impl Default for TickRate {
    fn default() -> Self {
        Self::HZ_128
    }
}
