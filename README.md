# High-Tick Netcode Server & Time-Rewind Framework

A modular, production-grade Rust workspace providing a **High-Tick (128 Hz) Server & Lag-Compensated Time-Rewind State Engine** library (`netcode-core`), designed to be strictly **transport-agnostic** and ready for enterprise edge integration with **Cloudflare `quiche`** (QUIC) and **Cloudflare `pingora`** (Edge Gateway).

Accompanied by an interactive 2D top-down shooter demo template (`game-server` & `game-client`).

---

## 🏛️ Distributed Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       TIER 1: EDGE GATEWAY (Pingora)                        │
│   • Ingress reverse proxy & Anycast edge termination                        │
│   • DDoS mitigation, malformed packet filtering, session token inspection   │
│   • Dynamic routing to backend game server instances                        │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ High-speed internal VPC
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                 TIER 2: TRANSPORT LAYER (quiche / Raw UDP)                  │
│   • Decoupled via `DatagramTransport` trait                                 │
│   • Raw UDP: Minimal 8-byte overhead, ultra-low latency                     │
│   • QUIC / quiche: TLS 1.3 encrypted, RFC 9221 Unreliable Datagrams,        │
│     zero-downtime Wi-Fi <-> 5G connection migration via Connection IDs      │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Abstracted Datagram Stream
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                 TIER 3: SIMULATION & NETCODE (`netcode-core`)               │
│   • 100% Transport-Agnostic: Zero coupling to sockets, TLS, or routing      │
│   • 128 Hz discrete timestep server ticker                                  │
│   • Zero-allocation `RingBuffer<T, 128>` snapshot history                   │
│   • `TimeRewindEngine`: Lag-compensated snapshot LERP & raycast verification│
│   • Client-side prediction, server reconciliation, 100 ms interpolation     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Workspace Structure

```
netcode-shooter/
├── Cargo.toml                        ← Cargo workspace root
├── README.md                         ← Project overview & architecture
├── IMPLEMENTATION_GUIDE.md           ← Complete step-by-step implementation guide
├── netcode-core/                     ◄── ENGINE LIBRARY (Transport-Agnostic)
│   └── src/
│       ├── lib.rs
│       ├── transport.rs              ← DatagramTransport trait (UDP, quiche, or mock)
│       ├── ring_buffer.rs            ← RingBuffer<T, N> (MaybeUninit, zero heap alloc)
│       ├── rewind.rs                 ← TimeRewindEngine & Timestamped trait
│       ├── sequence.rs               ← u32 sequence wrap-around math
│       └── ticker.rs                 ← TickRate driver
├── game-common/                      ◄── PROTOCOL & MATH SKELETON
│   └── src/
│       ├── protocol.rs               ← InputPacket, StateSnapshot stubs
│       └── math.rs                   ← Lerp & segment-circle raycast stubs
├── game-server/                      ◄── 128 Hz SERVER SKELETON
│   └── src/
│       ├── rewind.rs                 ← ShooterRewindAdapter stubs
│       └── ticker.rs                 ← UDP drain, physics, broadcast stubs
└── game-client/                      ◄── DEMO CLIENT SKELETON
    └── src/
        ├── prediction.rs             ← Client prediction & reconciliation stubs
        ├── interpolation.rs          ← Snapshot interpolation stubs
        └── main.rs                   ← Macroquad client loop stubs
```

---

## 📖 How to Use This Project

1. Read **[`IMPLEMENTATION_GUIDE.md`](IMPLEMENTATION_GUIDE.md)** for detailed mathematical formulas, algorithms, data structure layouts, and cloud architecture deep dives (`quiche` & `pingora`).
2. Implement the `todo!()` functions in each crate:
   - Phase 1: `netcode-core` (Ring buffer, time-rewind engine, sequence wrap-around, transport trait)
   - Phase 2: `game-common` (Math raycasting, protocol serialization)
   - Phase 3: `game-server` (128 Hz server ticker, hit registration)
   - Phase 4: `game-client` (Prediction, reconciliation, interpolation)
3. Check and test your implementation:
   ```bash
   cargo check --workspace
   cargo test --workspace
   ```
