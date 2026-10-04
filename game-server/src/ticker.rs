//! 128 Hz server ticker skeleton.

use std::{collections::HashMap, net::SocketAddr};

use game_common::{EntityState, InputPacket, StateSnapshot};
use glam::Vec2;
use netcode_core::{RingBuffer, TickRate};
use tokio::net::UdpSocket;

pub const TICK_RATE: TickRate = TickRate::HZ_128;
pub const PLAYER_SPEED: f32 = 5.0;

struct ClientEntry {
    last_seq: u32,
    position: Vec2,
    rotation: f32,
    health: u8,
    pending_fire: Option<PendingFire>,
}

struct PendingFire {
    shot_timestamp: u64,
    aim_angle: f32,
}

impl ClientEntry {
    fn new(_id: u32) -> Self {
        todo!("Construct new player entry with spawn position")
    }

    fn to_entity_state(&self, _id: u32) -> EntityState {
        todo!("Construct EntityState from self")
    }
}

pub struct ServerTicker {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, (u32, ClientEntry)>,
    next_id: u32,
    tick: u64,
    ring_buffer: RingBuffer<StateSnapshot, 128>,
}

impl ServerTicker {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            clients: HashMap::new(),
            next_id: 1,
            tick: 0,
            ring_buffer: RingBuffer::new(),
        }
    }

    pub async fn run(self) {
        todo!("Run 128Hz tokio interval game loop calling tick_once")
    }

    async fn tick_once(&mut self) {
        todo!("1. Drain UDP socket non-blocking\n2. Integrate physics\n3. Perform time-rewind hit registration\n4. Record RingBuffer snapshot\n5. Broadcast snapshot to clients")
    }

    fn handle_datagram(&mut self, _data: &[u8], _from: SocketAddr, _now_ms: u64) {
        todo!("Deserialize InputPacket, validate sequence, update movement, and record fire intent")
    }
}
