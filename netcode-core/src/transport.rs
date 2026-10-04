//! Transport-agnostic datagram networking abstraction.
//!
//! Enables `netcode-core` to remain completely decoupled from physical I/O.
//! Can be backed by standard UDP (`tokio::net::UdpSocket`), QUIC Unreliable Datagrams
//! (Cloudflare `quiche`), or in-memory mock channels for deterministic unit tests.

use std::net::SocketAddr;

/// An unbuffered, non-blocking datagram transport interface.
pub trait DatagramTransport {
    type Error: std::fmt::Debug;

    /// Attempt to receive a single datagram without blocking.
    ///
    /// - Returns `Ok(Some((bytes_read, peer_addr)))` when a datagram is available.
    /// - Returns `Ok(None)` when no datagrams are queued (equivalent to WouldBlock).
    fn try_recv_from(&mut self, buf: &mut [u8]) -> Result<Option<(usize, SocketAddr)>, Self::Error>;

    /// Send a datagram packet to a target peer.
    fn send_to(&mut self, buf: &[u8], target: SocketAddr) -> Result<usize, Self::Error>;
}
