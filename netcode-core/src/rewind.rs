//! Generic lag-compensation time-rewind engine skeleton.

use core::time;

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
        ring_buffer: &RingBuffer<S, N>,
        target_timestamp_ms: u64,
        max_rewind_ms: u64,
        interpolate: impl FnOnce(&S, &S, f32) -> RewoundState,
        query: impl FnOnce(&RewoundState) -> Option<HitResult>,
    ) -> Option<HitResult>
    where
        S: Timestamped,
    {
        let newest = ring_buffer.newest()?;

        if newest.timestamp_ms().saturating_sub(target_timestamp_ms) > max_rewind_ms {
            return None;
        }

        let mut before: Option<&S> = None;
        let mut after: Option<&S> = None;

        for snapshot in ring_buffer.iter(){
            let timestamp = snapshot.timestamp_ms();
            if timestamp <= target_timestamp_ms {
                match before{
                    Some(previous) if timestamp <= previous.timestamp_ms() => {} // Snapshot is older/equal so don't replace it
                    _ => before = Some(snapshot) // replace in every other case
                }
            }
            if timestamp >= target_timestamp_ms {
                match after {
                    Some(next) if timestamp >= next.timestamp_ms() => {} // Snapshot is newer/equal so don't replace it
                    _ => after = Some(snapshot)
                }
            }
        }

        let before = before?;
        let after = after?;

        let span = after.timestamp_ms().saturating_sub(before.timestamp_ms());

        let t = if span == 0 {
            0.0
        } else {
            let numerator = target_timestamp_ms.saturating_sub(before.timestamp_ms());

            (numerator as f32 / span as f32).clamp(0.0, 1.0)
        };
        let rewound_state = interpolate(before, after, t);
        query(&rewound_state)
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
