//! Generic lag-compensation time-rewind engine skeleton.

use crate::ring_buffer::RingBuffer;

/// Trait for snapshot types that store a timestamp in milliseconds.
pub trait Timestamped {
    fn timestamp_ms(&self) -> u64;
}

/// Generic time-rewind lag compensation engine.
pub struct TimeRewindEngine;

impl TimeRewindEngine {
    /// Rewinds world state in `ring_buffer` to `target_timestamp_ms`,
    /// interpolates between bracketing snapshots using `interpolate`, and executes `query`.
    pub fn rewind_and_query<S, const N: usize, RewoundState, HitResult>(
        _ring_buffer: &RingBuffer<S, N>,
        _target_timestamp_ms: u64,
        _max_rewind_ms: u64,
        _interpolate: impl FnOnce(&S, &S, f32) -> RewoundState,
        _query: impl FnOnce(&RewoundState) -> Option<HitResult>,
    ) -> Option<HitResult>
    where
        S: Timestamped,
    {
        todo!("1. Check buffer empty/age\n2. Find bracketing snapshots (before, after)\n3. Calculate t factor\n4. Interpolate state\n5. Execute query")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySnapshot {
        ts: u64,
    }

    impl Timestamped for DummySnapshot {
        fn timestamp_ms(&self) -> u64 {
            self.ts
        }
    }

    #[test]
    #[should_panic]
    fn test_rewind_skeleton() {
        let buf: RingBuffer<DummySnapshot, 4> = RingBuffer::new();
        let _ = TimeRewindEngine::rewind_and_query(&buf, 100, 1000, |b, _, _| b.ts, |_| Some(()));
    }
}
