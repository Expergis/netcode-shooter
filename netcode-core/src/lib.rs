//! # `netcode-core`
//!
//! A generic, transport-agnostic framework for building high-frequency (e.g. 128 Hz)
//! multiplayer game servers with zero-heap lag-compensation (time rewind) state tracking.
//!
//! ## Core Components
//! - [`RingBuffer`]: Fixed-capacity stack/inline circular buffer (`MaybeUninit<T>`) for zero-allocation tick snapshot history storage.
//! - [`TimeRewindEngine`]: Generic lag-compensation engine that interpolates historical snapshots to arbitrary client-stamped timestamps and runs spatial/game queries.
//! - [`sequence`]: Network sequence wrap-around and freshness checking algorithms.
//! - [`ticker`]: High-precision server game loop timing utilities.
//! - [`transport`]: Transport-agnostic datagram trait decoupling simulation logic from I/O (UDP, QUIC/quiche, or in-memory testing).

pub mod rewind;
pub mod ring_buffer;
pub mod sequence;
pub mod ticker;
pub mod transport;

pub use rewind::{TimeRewindEngine, Timestamped};
pub use ring_buffer::RingBuffer;
pub use sequence::{is_sequence_newer, sequence_less_than_or_equal};
pub use ticker::TickRate;
pub use transport::DatagramTransport;
