# High-Tick Netcode Server & Time-Rewind Framework: Complete Implementation Guide

Welcome! This guide is written specifically for developers with **zero prior networking or netcode experience**. It provides the foundational theory, mathematical derivations, memory layout diagrams, algorithmic pseudocode, and step-by-step instructions to implement the entire codebase from scratch.

It also details how to integrate Cloudflare's open-source enterprise tools (**`quiche`** for QUIC transport and **`pingora`** for edge gateway/reverse-proxying) while keeping the core simulation engine (**`netcode-core`**) strictly **transport-agnostic**.

---

# Table of Contents
1. [Foundations of Multiplayer Netcode](#1-foundations-of-multiplayer-netcode)
   - [The Reality of Physical Networks](#the-reality-of-physical-networks)
   - [Why UDP Instead of TCP?](#why-udp-instead-of-tcp)
   - [Tick Rate (128 Hz) vs Frame Rate](#tick-rate-128-hz-vs-frame-rate)
   - [The Four Pillars of Modern Netcode](#the-four-pillars-of-modern-netcode)
2. [Distributed Architecture: Edge Gateway, Transport & Simulation Layers](#2-distributed-architecture-edge-gateway-transport--simulation-layers)
   - [Three-Tier Architecture Overview](#three-tier-architecture-overview)
   - [Why Keep `netcode-core` Transport-Agnostic?](#why-keep-netcode-core-transport-agnostic)
3. [Cloudflare `quiche` Integration (QUIC Transport Layer)](#3-cloudflare-quiche-integration-quic-transport-layer)
   - [Why QUIC for Gaming? (Streams vs Unreliable Datagrams)](#why-quic-for-gaming-streams-vs-unreliable-datagrams)
   - [Header Overhead & Performance Trade-offs](#header-overhead--performance-trade-offs)
   - [Implementing `DatagramTransport` over `quiche`](#implementing-datagramtransport-over-quiche)
   - [Zero-Downtime Connection Migration](#zero-downtime-connection-migration)
4. [Cloudflare `pingora` Integration (Edge Proxy & Gateway Layer)](#4-cloudflare-pingora-integration-edge-proxy--gateway-layer)
   - [Why Hide Game Servers Behind an Edge Proxy?](#why-hide-game-servers-behind-an-edge-proxy)
   - [The Edge Relay Pattern (Steam Datagram Relay / Riot Direct Model)](#the-edge-relay-pattern-steam-datagram-relay--riot-direct-model)
   - [Architecture of a Pingora Game Gateway](#architecture-of-a-pingora-game-gateway)
5. [Phase 1: `netcode-core` (Generic Library)](#5-phase-1-netcode-core-generic-library)
   - [1.1 `transport.rs` — Transport-Agnostic Datagram Trait](#51-transportrs--transport-agnostic-datagram-trait)
   - [1.2 `sequence.rs` — Wrap-Around Sequence Mathematics](#52-sequencers--wrap-around-sequence-mathematics)
   - [1.3 `ring_buffer.rs` — Zero-Allocation State History (`RingBuffer<T, N>`)](#53-ring_bufferrs--zero-allocation-state-history-ringbuffert-n)
   - [1.4 `ticker.rs` — High-Precision Tick Timing](#54-tickerrs--high-precision-tick-timing)
   - [1.5 `rewind.rs` — Generic Lag-Compensation Time-Rewind Engine](#55-rewindrs--generic-lag-compensation-time-rewind-engine)
6. [Phase 2: `game-common` (Protocol & Math)](#6-phase-2-game-common-protocol--math)
   - [2.1 `math.rs` — LERP & Quadratic Raycast Collision](#61-mathrs--lerp--quadratic-raycast-collision)
   - [2.2 `protocol.rs` — Network Packets & Binary Search](#62-protocolrs--network-packets--binary-search)
7. [Phase 3: `game-server` (128 Hz Authoritative Server)](#7-phase-3-game-server-128-hz-authoritative-server)
   - [3.1 `rewind.rs` — Shooter Domain Rewind Adapter](#71-rewindrs--shooter-domain-rewind-adapter)
   - [3.2 `ticker.rs` — UDP Draining, Two-Phase Simulation & Broadcast](#72-tickerrs--udp-draining-two-phase-simulation--broadcast)
   - [3.3 `main.rs` — Async Tokio Runtime Initialization](#73-mainrs--async-tokio-runtime-initialization)
8. [Phase 4: `game-client` (Prediction, Reconciliation & Interpolation)](#8-phase-4-game-client-prediction-reconciliation--interpolation)
   - [4.1 `prediction.rs` — Local Prediction & Server Reconciliation](#81-predictionrs--local-prediction--server-reconciliation)
   - [4.2 `interpolation.rs` — Snapshot Interpolation Buffer (100 ms Delay)](#82-interpolationrs--snapshot-interpolation-buffer-100-ms-delay)
   - [4.3 `main.rs` — Macroquad UDP Game Loop & Coordinate Mapping](#83-mainrs--macroquad-udp-game-loop--coordinate-mapping)
9. [Testing, Benchmarking & Troubleshooting Handbook](#9-testing-benchmarking--troubleshooting-handbook)

---

# 1. Foundations of Multiplayer Netcode

### The Reality of Physical Networks
In an offline single-player game, when you press `W`, the character moves on the very next frame (16 ms later at 60 FPS). 

In multiplayer games over the internet:
1. **Latency / Ping (RTT)**: Signals travel through fiber-optic cables, routers, and switches. A round-trip time (RTT) of 50 ms to 150 ms is standard.
2. **Jitter**: Packet transit times fluctuate. Packet A may take 40 ms, while Packet B takes 75 ms.
3. **Packet Loss**: Around 0.5% to 5% of packets are dropped by congested routers.
4. **Out-of-Order Delivery**: Packet #5 might arrive before Packet #4.

If you waited for the server to confirm every keypress before moving your character, your controls would feel sluggish, delayed, and unplayable.

---

### Why UDP Instead of TCP?
- **TCP (Transmission Control Protocol)** guarantees ordered, reliable byte streams. If packet #3 is dropped, TCP halts delivery of packets #4, #5, and #6 until packet #3 is retransmitted and acknowledged (**Head-of-Line Blocking**). In a fast-paced shooter, stale position data from 100 ms ago is useless; we only care about the latest state!
- **UDP (User Datagram Protocol)** is connectionless and sends independent packets ("datagrams"). If a packet is lost, it is forgotten; the next packet arrives 7.8 ms later anyway. This provides the minimum possible latency.

---

### Tick Rate (128 Hz) vs Frame Rate
- **Render Frame Rate (Client)**: Dependent on the user's GPU and monitor (e.g. 60 Hz, 144 Hz, 240 Hz). Varies dynamically.
- **Simulation Tick Rate (Server)**: A fixed discrete timestep clock. 
  $$\text{Interval} = \frac{1\,000\,000\,000 \text{ ns}}{128 \text{ Hz}} = 7\,812\,500 \text{ ns} = 7.8125 \text{ ms}$$
Every $7.8125\text{ ms}$, the server runs one complete tick: reading network inputs, moving entities, checking collisions, and broadcasting state.

---

### The Four Pillars of Modern Netcode

```
                              ┌────────────────────────────────────────┐
                              │  THE FOUR PILLARS OF ACTION NETCODE    │
                              └────────────────────────────────────────┘
                                                  │
         ┌────────────────────────┬───────────────┴──────────────┬────────────────────────┐
         ▼                        ▼                              ▼                        ▼
 1. CLIENT-SIDE           2. SERVER                      3. SNAPSHOT              4. LAG COMPENSATION
    PREDICTION               RECONCILIATION                 INTERPOLATION            (TIME REWIND)
 ─────────────────        ─────────────────              ─────────────────        ───────────────────
 Player moves instantly   Server corrects client         Smoothly glides          Server rolls back
 on keypress without      mistakes by resetting          remote players           hitboxes to client's
 waiting for server.      and re-simulating inputs.      100ms in the past.       shot time to verify hits.
```

1. **Client-Side Prediction**: When the player presses `W`, the client updates its position immediately.
2. **Server Reconciliation**: The server sends authoritative snapshots back to the client. If the server says the player was blocked by a wall or pushed by an enemy, the client snaps back to the server's confirmed position, then re-simulates all inputs that the server hasn't processed yet.
3. **Snapshot Interpolation**: Other players are moving across the network. If we rendered their positions as soon as packets arrived, jitter would cause them to stutter and teleport. Instead, we intentionally render them **100 ms in the past**, interpolating between two known historical snapshots.
4. **Lag Compensation (Time Rewind)**: When Player A shoots at Player B, Player A is looking at Player B's interpolated position from 100 ms ago. If the server checks the bullet against Player B's *current* position, it will miss! The server must rewind Player B's position back to the exact millisecond Player A pulled the trigger.

---

# 2. Distributed Architecture: Edge Gateway, Transport & Simulation Layers

A production multiplayer system is divided into three distinct operational tiers:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 TIER 1: EDGE GATEWAY (Pingora)                                  │
│  • Public IP Ingress & Edge Anycast Termination                                                 │
│  • DDoS Mitigation & Malformed Packet Filtering                                                 │
│  • Session Token Verification & Dynamic Match Routing                                           │
└───────────────────────────────────────────────┬─────────────────────────────────────────────────┘
                                                │ Internal high-speed VPC (UDP / QUIC)
                                                ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           TIER 2: TRANSPORT PROTOCOL (quiche / raw UDP)                         │
│  • Plugs into `DatagramTransport` Trait                                                         │
│  • Option A: Raw UDP (Lowest overhead: 8-byte header, zero encryption cost)                     │
│  • Option B: QUIC / quiche (TLS 1.3 encrypted, RFC 9221 Unreliable Datagrams, Connection IDs)   │
└───────────────────────────────────────────────┬─────────────────────────────────────────────────┘
                                                │ Abstracted Datagram Stream
                                                ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                          TIER 3: SIMULATION & NETCODE (`netcode-core`)                          │
│  • 100% Transport-Agnostic: Zero knowledge of sockets, TLS, or routing                          │
│  • 128 Hz Discrete Timestep Server Ticker                                                       │
│  • Zero-Allocation `RingBuffer<T, 128>` State History                                           │
│  • `TimeRewindEngine`: Lag-compensated snapshot interpolation & spatial hit verification        │
│  • Client-side Prediction, Reconciliation, and 100 ms Snapshot Interpolation                    │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### Why Keep `netcode-core` Transport-Agnostic?
1. **Separation of Concerns**: A game physics tick loop should never care whether a packet came from a raw Linux socket, a QUIC stream, WebTransport, or a local loopback mock.
2. **Deterministic Offline Testing**: You can test the 128 Hz tick loop, lag-compensation rewind, and packet loss recovery in unit tests without opening a single network port, using an in-memory test harness!
3. **Pluggable Protocols**: A game can deploy raw UDP on desktop platforms, QUIC/quiche on mobile (to survive Wi-Fi $\leftrightarrow$ 5G handovers), and WebTransport in browsers—all using the identical `netcode-core` engine.

---

# 3. Cloudflare `quiche` Integration (QUIC Transport Layer)

Cloudflare's **`quiche`** is an open-source Rust implementation of the IETF **QUIC** protocol and HTTP/3.

### Why QUIC for Gaming? (Streams vs Unreliable Datagrams)
Standard QUIC streams guarantee reliability and ordering. **Never use standard QUIC streams for continuous 128 Hz player positions**, because a lost packet will stall subsequent positions (head-of-line blocking).

However, QUIC includes **RFC 9221: An Unreliable Datagram Extension to QUIC**. This allows sending unordered, unreliable datagrams within an authenticated, encrypted QUIC session!

```
                                  QUIC CONNECTION
  ┌─────────────────────────────────────────────────────────────────────────────┐
  │                                                                             │
  │  Stream 0 (Reliable / Ordered): Login Handshake, Chat, Team Selection       │
  │                                                                             │
  │  Unreliable Datagrams (RFC 9221): 128 Hz WASD Inputs & World Snapshots      │
  │  (No retransmissions, no head-of-line blocking, but protected by TLS 1.3!)  │
  │                                                                             │
  └─────────────────────────────────────────────────────────────────────────────┘
```

### Header Overhead & Performance Trade-offs

| Transport | Header Size | TLS Encryption | Connection Migration | Head-of-Line Blocking? |
| :--- | :--- | :--- | :--- | :--- |
| **Raw UDP** | **8 bytes** | None (Plaintext) | ❌ No (IP change drops session) | ❌ None |
| **QUIC Datagrams (`quiche`)** | **~20 to 35 bytes** | ✅ TLS 1.3 built-in | ✅ Yes (64-bit Connection ID) | ❌ None (via RFC 9221) |
| **TCP** | **20 to 60 bytes** | Extra (TLS wrapper) | ❌ No | ⚠️ **Severe (HALTS stream)** |

> **When to choose raw UDP**: Competitive esports desktop titles (CS2, Valorant) where every byte counts at 128 Hz and IP addresses are static.
> **When to choose `quiche`**: Mobile games, cross-platform games, or web games where players frequently switch between Wi-Fi and 5G cellular networks, or where anti-packet-sniffing encryption is mandatory.

---

### Implementing `DatagramTransport` over `quiche`
To plug `quiche` into `netcode-core`, wrap the `quiche::Connection` event loop inside `DatagramTransport`:

- **State Management**:
  - Store the underlying non-blocking UDP socket (`tokio::net::UdpSocket`).
  - Maintain a `quiche::Config` configured with TLS credentials and unreliable datagram support enabled (`enable_dgram(true, 1024, 1024)`).
  - Track active sessions mapping client addresses (`SocketAddr`) to their respective `quiche::Connection` state machine instances.
- **Receiving Datagrams (`try_recv_from`)**:
  - Read raw incoming UDP packets from the OS socket without blocking.
  - Parse the QUIC header to identify or create the corresponding `quiche::Connection`.
  - Feed raw bytes into the connection's packet processing pipeline (`conn.recv`).
  - Poll for any pending application datagrams queued on that connection (`conn.dgram_recv`).
  - If a datagram is ready, return the payload slice and peer address; if no datagrams are waiting, return `Ok(None)` to signal the caller without blocking the tick loop.
- **Sending Datagrams (`send_to`)**:
  - Locate the target peer's active connection.
  - Queue the unencrypted game payload using QUIC's datagram queue (`conn.dgram_send`).
  - Flush the connection's egress packet pipeline: QUIC encrypts and frames the datagram, which is then written out to the OS socket.

---

### Zero-Downtime Connection Migration
With standard UDP, a player's session is identified by `(IP, Port)`. If a mobile player walks out of their house and their phone transitions from Wi-Fi to 5G, their IP address changes. The game server thinks a stranger connected and disconnects them.

With `quiche`, sessions are bound to a cryptographically secure **Connection ID (CID)**. Even when the client's public IP and port change, `quiche` matches the CID, authenticates the packet via TLS 1.3, seamlessly migrates the connection, and the 128 Hz game simulation continues without a disconnection!

---

# 4. Cloudflare `pingora` Integration (Edge Proxy & Gateway Layer)

Cloudflare's **`pingora`** is a high-performance async Rust framework built to build proxies, load balancers, and reverse proxies (handling over 40 million requests per second across Cloudflare's network).

### Why Hide Game Servers Behind an Edge Proxy?
Directly exposing your 128 Hz game server to public IPs is hazardous:
1. **Direct DDoS Floods**: A 10 Gbps UDP flood will saturate the server's NIC and starve the 128 Hz CPU tick budget.
2. **IP Harvesting**: Malicious players can target the server IP to force disconnects or exploit vulnerabilities.
3. **No Routing Intelligence**: Players in Brazil connecting to a server in Virginia might take poor BGP internet routes.

---

### The Edge Relay Pattern (Steam Datagram Relay / Riot Direct Model)

```
[ Client (Los Angeles) ] 
       │ (Nearest Edge Point of Presence)
       ▼
[ Cloudflare Pingora Edge Gateway (LA Edge) ]
  • Inspects authentication token
  • Drops SYN/UDP floods at edge capacity (Tbps scale)
  • Ingresses packet onto Cloudflare / Private Backbone
       │
       │ (Dedicated low-jitter fiber link)
       ▼
[ game-server Cluster (Virginia Data Center) ]
  • Receives clean, pre-authenticated datagrams
  • 100% of CPU is dedicated to 128 Hz physics & lag-compensation rewind
```

### Architecture of a Pingora Game Gateway
In a Pingora-powered gateway crate (`game-gateway`):
1. **Downstream Session Handler**:
   - Accepts incoming client UDP / QUIC connections.
   - Decodes a lightweight authentication ticket (verifying user ID, match ID, and signature).
2. **Load Balancer & Router**:
   - Maps the `match_id` to the internal VPC address of the authoritative `game-server` instance (e.g. `10.0.1.42:4000`).
3. **Upstream Forwarder**:
   - Forwards sanitized datagrams to the backend server over high-speed private networking.
   - Forwards authoritative state snapshots back down to the player.

---

# 5. Phase 1: `netcode-core` (Generic Library)

---

### 5.1 `transport.rs` — Transport-Agnostic Datagram Trait
- **File:** `netcode-core/src/transport.rs`
- Defines the unified abstraction for sending and receiving datagrams:
  - Trait `DatagramTransport` with an associated `Error` type implementing `std::fmt::Debug`.
  - Non-blocking receive method: `try_recv_from(&mut self, buf: &mut [u8]) -> Result<Option<(usize, SocketAddr)>, Self::Error>`.
    - If a packet is waiting, read into `buf` and return `Ok(Some((bytes_read, peer_addr)))`.
    - If no packet is waiting, return `Ok(None)` immediately without blocking.
  - Sending method: `send_to(&mut self, buf: &[u8], target: SocketAddr) -> Result<usize, Self::Error>`.
    - Transmit `buf` to destination `target` and return the number of bytes transferred.

---

### 5.2 `sequence.rs` — Wrap-Around Sequence Mathematics
- **File:** `netcode-core/src/sequence.rs`
- **Problem**: Packets contain a `sequence: u32` counter. `u32::MAX` is $4\,294\,967\,295$. At 128 packets/sec, this wraps around to `0` after ~388 days of uptime. Standard `<` or `>` comparisons will fail when comparing `seq = 2` against `seq = 4294967295`.

```
                      0 (Wrap point)
                   .  '  .
               .             .
       u32::MAX               1
            |                   |
            |     Half Range    |
            |      (2^31)       |
            |                   |
               .             .
                   .  _  .
                 u32::MAX / 2
```

#### Step-by-Step Implementation Requirements:
1. **`is_sequence_newer(sequence_a: u32, sequence_b: u32) -> bool`**:
   - Compute the modular difference between `sequence_a` and `sequence_b` using wrapping subtraction (`wrapping_sub`). This accounts for integer overflow without panicking in debug mode.
   - For `sequence_a` to be considered newer, two conditions must hold:
     - The difference must be non-zero (identical sequence numbers are not newer).
     - The difference must fall strictly within the forward half of the 32-bit integer range (i.e. less than $2^{31}$ or `1 << 31`).
2. **`sequence_less_than_or_equal(sequence_a: u32, sequence_b: u32) -> bool`**:
   - Compute the difference measuring how far `sequence_b` is ahead of `sequence_a` using wrapping subtraction (`sequence_b.wrapping_sub(sequence_a)`).
   - If that difference falls within the forward half-range ($< 2^{31}$), then `sequence_a` is either equal to or older than `sequence_b`. Otherwise, `sequence_a` is newer.

---

### 5.3 `ring_buffer.rs` — Zero-Allocation State History (`RingBuffer<T, N>`)
- **File:** `netcode-core/src/ring_buffer.rs`
- **Problem**: At 128 Hz, allocating heap memory (`Vec::push`, `Vec::remove(0)`) causes memory fragmentation. We store historical snapshots in an inline array of `[MaybeUninit<T>; N]` with single-cycle bitmask modulo indexing: `(head + 1) & (N - 1)`.

```
Physical Storage: [ MaybeUninit<T> ; N ]
Indices:             0     1     2     3     4     5     6     7
                  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
Slots:            │ S_8 │ S_9 │ S_2 │ S_3 │ S_4 │ S_5 │ S_6 │ S_7 │
                  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
                               ▲                 ▲
                              head              oldest (when full, head == oldest)
```

#### Step-by-Step Implementation Requirements:
1. **`new()`**:
   - Ensure $N$ is greater than zero and a power of two using a debug assertion.
   - Initialize the `[MaybeUninit<T>; N]` array using `MaybeUninit::uninit().assume_init()`. Since `MaybeUninit` holds uninitialized memory, assuming initialization of the array itself is safe and does not construct or initialize any instances of `T`.
   - Set both `head` and `len` counters to `0`.
2. **`push(value: T)`**:
   - Obtain a raw mutable pointer to the slot at `self.head` and write `value` into that location using pointer write operations (`write()`). Do not assign via dereference (`*ptr = value`) because uninitialized memory cannot be dropped.
   - Advance `head` by 1 using the bitwise mask `(self.head + 1) & (N - 1)` so it wraps around to `0` when reaching $N$.
   - If `self.len < N`, increment `self.len` by 1. Once `self.len == N`, the buffer is full and `self.len` remains fixed at $N$ while older entries are overwritten.
3. **`get(i: usize) -> Option<&T>`**:
   - Check if logical index `i` is within bounds (`i < self.len`). Return `None` if out of bounds.
   - Determine the physical array index of the oldest valid element:
     - When the buffer is full (`len == N`), `head` points to the next slot to be overwritten, which is also the oldest slot.
     - When the buffer is not yet full (`len < N`), the oldest slot is always index `0`.
   - Compute physical index: `(oldest + i) & (N - 1)`.
   - Return a shared reference to the initialized element at that slot (`assume_init_ref()`).
4. **`Drop` Implementation**:
   - Rust does not automatically run destructors for items wrapped in `MaybeUninit`. To prevent memory leaks if `T` owns heap resources (e.g. `String`, `Vec`), implement `Drop`.
   - Iterate through all initialized entries (`0..self.len`).
   - For each logical index, compute its physical index using the same translation as `get()`.
   - Call `assume_init_drop()` on each valid slot to drop the element in-place. Uninitialized slots must never be dropped.

---

### 5.4 `ticker.rs` — High-Precision Tick Timing
- **File:** `netcode-core/src/ticker.rs`
- Computes nanosecond durations:
  - `tick_duration = Duration::from_nanos(1_000_000_000 / self.hz)`
  - `dt = self.tick_duration().as_secs_f32()` (0.0078125s for 128 Hz).

---

### 5.5 `rewind.rs` — Generic Lag-Compensation Time-Rewind Engine
- **File:** `netcode-core/src/rewind.rs`
- Operates on any type implementing `Timestamped` (`fn timestamp_ms(&self) -> u64`):

```
Snapshots in RingBuffer:
[ t=100ms ]     [ t=108ms ]     [ t=116ms ]     [ t=124ms ]
                     ▲                               ▲
                   BEFORE                          AFTER
                               ▲
                      Target: t = 112ms
```

1. Verify shot age: $\text{newest.ts} - T_{\text{target}} \le \text{max\_rewind\_ms}$.
2. Scan ring buffer to find `before` snapshot ($\text{ts} \le T_{\text{target}}$) and `after` snapshot ($\text{ts} \ge T_{\text{target}}$).
3. Compute interpolation factor:
   $$\text{span} = \text{after.ts} - \text{before.ts}$$
   $$t = \begin{cases} 0.0 & \text{if span } = 0 \\ \left(\frac{T_{\text{target}} - \text{before.ts}}{\text{span}}\right).clamp(0.0, 1.0) & \text{otherwise} \end{cases}$$
4. Call `let rewound_state = interpolate(before, after, t);`.
5. Call `query(&rewound_state)`.

---

# 6. Phase 2: `game-common` (Protocol & Math)

---

### 6.1 `math.rs` — LERP & Quadratic Raycast Collision
- **File:** `game-common/src/math.rs`

#### 1. Linear Interpolation (LERP)
$$P(t) = A + (B - A) \times t$$

#### 2. Segment-Circle Intersection (Quadratic Discriminant Raycast)
Ray from $P$ along $D$ (length of $D$ is max range). Circle at $C$ with radius $R$.
Let $M = P - C$.
Quadratic formula: $a t^2 + 2b t + c = 0$ where:
- $a = \|D\|^2$ (normalized $D \implies a = 1$)
- $b = M \cdot D$
- $c = M \cdot M - R^2$
Discriminant: $\Delta = b^2 - c$.

```
Geometric Early Culls:
1. c > 0 (starts outside) && b > 0 (points away) ──► Miss! Return None.
2. Δ < 0 (discriminant negative)                 ──► Miss! Return None.
3. t = -b - sqrt(Δ)
4. t_norm = t / ||D||
5. If 0.0 <= t_norm <= 1.0 ─────────────────────► Hit! Return Some(t_norm).
```

---

### 6.2 `protocol.rs` — Network Packets & Binary Search
- **File:** `game-common/src/protocol.rs`
- **Data Structures**:
  - `InputPacket`: Contains sequential identifier (`sequence`), client generation millisecond timestamp (`timestamp_ms`), directional movement vector (`movement`), trigger state (`fire`), and aim heading in radians (`aim_angle`).
  - `EntityState`: Contains unique identifier (`id`), spatial coordinates (`position`), heading angle (`rotation`), and current health points (`health`).
  - `StateSnapshot`: Contains server logical tick number (`tick`), server capture timestamp (`timestamp_ms`), highest acknowledged client sequence (`last_processed_seq`), and an ordered collection of entity states (`entities`).
- **Traits & Methods**:
  - Implement `Timestamped` on `StateSnapshot` by returning `self.timestamp_ms`.
  - Implement `InputPacket::is_newer_than(&self, other: &Self) -> bool` by delegating to `netcode_core::is_sequence_newer`.
  - Implement `find_entity(&self, id: u32) -> Option<&EntityState>`:
    - Because the server maintains `self.entities` sorted by entity ID, perform a binary search over the slice using the entity `id` as the comparison key (e.g. using Rust's `binary_search_by_key`).
    - If found, map the slice index to an immutable reference to that `EntityState`; if not found, return `None`.

---

# 7. Phase 3: `game-server` (128 Hz Authoritative Server)

---

### 7.1 `rewind.rs` — Shooter Domain Rewind Adapter
- **File:** `game-server/src/rewind.rs`
- Calls `TimeRewindEngine::rewind_and_query`:
  - **Interpolate closure**: Interpolates entity positions between `before` and `after` snapshots using `lerp_vec2`.
  - **Query closure**: Runs `segment_circle_intersect` against all rewound entities and returns the `id` of the closest hit.

---

### 7.2 `ticker.rs` — UDP Draining, Two-Phase Simulation & Broadcast
- **File:** `game-server/src/ticker.rs`

#### The 5 Steps of `tick_once()`:
1. **Drain UDP**: Non-blocking `try_recv_from` loop (draining up to 256 packets/tick).
2. **Simulate Physics**: Update positions based on client WASD inputs; clamp to arena boundaries.
3. **Lag-Compensated Hit Registration (Two-Phase Pattern)**:
   - Extract fire events in a preliminary phase: iterate mutably over active clients, consume each client's `pending_fire` using `Option::take()`, and collect the resulting tuples containing shooter entity ID, firing muzzle position, and fire parameters into a temporary `Vec`. This finishes the mutable borrow on `self.clients`.
   - Run `ShooterRewindAdapter::rewind_and_cast` against the ring buffer for each collected fire event, recording hit entity IDs and damage amounts into a second temporary list.
   - Apply damage in a separate follow-up pass by looking up each victim entry in `self.clients` and decrementing health using saturating subtraction (`saturating_sub`).
4. **Record Snapshot**: Push current `StateSnapshot` to `RingBuffer<StateSnapshot, 128>`.
5. **Broadcast**: Serialize and send state snapshot to each connected client address.

---

### 7.3 `main.rs` — Async Tokio Runtime Initialization
- Binds UDP socket to `127.0.0.1:4000`.
- Passes the socket into `ServerTicker::new(socket).run().await`.

---

# 8. Phase 4: `game-client` (Prediction, Reconciliation & Interpolation)

---

### 8.1 `prediction.rs` — Local Prediction & Server Reconciliation
- **File:** `game-client/src/prediction.rs`

#### Prediction (`apply_input`)
1. Increment sequence: `self.sequence = self.sequence.wrapping_add(1)`.
2. Instantly integrate movement: `self.predicted_position += dir * SPEED * dt`.
3. Store in history: `self.history.push_back(InputRecord { packet, dt })`.

#### Reconciliation (`reconcile`)
When authoritative snapshot arrives:
1. Identify the local player's authoritative entity inside `snapshot` using `find_entity`. If the local player ID is not yet known or not present in the snapshot, initialize or early exit accordingly.
2. Read the server's acknowledged input sequence: `snapshot.last_processed_seq`.
3. **Purge**:
   - Inspect the front of the local input queue (`self.history`).
   - While the front entry's sequence number is less than or equal to `snapshot.last_processed_seq` (using `sequence_less_than_or_equal`), discard it via `pop_front()`.
   - Stop purging as soon as an unacknowledged input is encountered.
4. **Reset**:
   - Overwrite `self.predicted_position` with the server's authoritative position for the local player entity.
5. **Re-simulate**:
   - Iterate through every remaining unacknowledged input record in `self.history` from oldest to newest.
   - For each input, advance `self.predicted_position` by running the exact movement physics integration used in prediction, applying arena boundary clamping at each step.

---

### 8.2 `interpolation.rs` — Snapshot Interpolation Buffer (100 ms Delay)
- **File:** `game-client/src/interpolation.rs`
- Buffer holds up to 32 snapshots.
- Render time: $T_{\text{render}} = \text{now\_ms} - 100\text{ ms}$.
- Finds bounding snapshots and computes:
  $$t = \frac{T_{\text{render}} - \text{before.ts}}{\text{after.ts} - \text{before.ts}}$$
- Smoothly LERPs positions of all remote players.

---

### 8.3 `main.rs` — Macroquad UDP Game Loop & Coordinate Mapping
- **File:** `game-client/src/main.rs`
- Reads WASD inputs and mouse aim angle.
- Invokes `predictor.apply_input(...)` and transmits datagrams.
- Drains incoming UDP snapshots, calls `reconcile()` and `push_snapshot()`.
- Draws arena walls, remote entities (**RED**), and local player (**GREEN**).

---

# 9. Testing, Benchmarking & Troubleshooting Handbook

### Fast Verification Commands
```bash
# 1. Typecheck the entire workspace
cargo check --workspace

# 2. Run unit tests in netcode-core and game-common
cargo test -p netcode-core
cargo test -p game-common

# 3. Run all tests across the workspace
cargo test --workspace

# 4. Start the 128 Hz server in terminal 1
cargo run -p game-server

# 5. Start client 1 in terminal 2
cargo run -p game-client

# 6. Start client 2 in terminal 3 (to test multiplayer interpolation & shooting)
cargo run -p game-client
```

---

### Common Pitfalls & How to Fix Them

| Symptom | Cause | Solution |
| :--- | :--- | :--- |
| **Player snaps / rubbers back constantly** | Prediction and server physics logic do not match exactly. | Ensure `PLAYER_SPEED` and clamping bounds are identical on client and server. |
| **Compiler Error: `set_nonblocking` not found** | `tokio::net::UdpSocket` has no `set_nonblocking` method. | `tokio::net::UdpSocket::bind` is already non-blocking. Only `std::net::UdpSocket` needs `set_nonblocking`. |
| **Remote players teleport or stutter** | `InterpolationBuffer` is receiving out-of-order packets. | Discard incoming snapshots if `snapshot.timestamp_ms <= last_timestamp_ms`. |
| **Hits register on client but server reports miss** | Shot timestamp was not rewound or lookback limit exceeded. | Verify client sends `unix_time_ms()` in `InputPacket.timestamp_ms`, and `TimeRewindEngine` uses that timestamp for LERP. |
| **Borrow Checker Error in Server Ticker** | Attempting to mutate victim health while iterating over clients map. | Use the Two-Phase pattern: collect hits into a `Vec<(u32, u8)>` first, then apply damage in a second loop. |

---

### Simulating Real-World Lag on Linux (Advanced Testing)
```bash
# Add 100ms artificial latency and 2% packet loss to loopback:
sudo tc qdisc add dev lo root netem delay 50ms loss 2%

# When finished testing, remove the rule:
sudo tc qdisc del dev lo root netem
```
*(Notice how your local green player still moves instantly with zero latency, remote players glide smoothly via interpolation, and your shots still register thanks to time rewind!)*
