//! `game-client` macroquad entry point skeleton.

mod interpolation;
mod prediction;

use std::net::UdpSocket;
use interpolation::InterpolationBuffer;
use macroquad::prelude::*;
use prediction::PredictionEngine;

const SERVER_ADDR: &str = "127.0.0.1:4000";
const CLIENT_PORT: u16 = 4001;

struct GameClient {
    socket: UdpSocket,
    predictor: PredictionEngine,
    interpolator: InterpolationBuffer,
}

impl GameClient {
    fn new() -> anyhow::Result<Self> {
        let bind_addr = format!("127.0.0.1:{CLIENT_PORT}");
        let socket = UdpSocket::bind(&bind_addr)?;
        socket.set_nonblocking(true)?;
        socket.connect(SERVER_ADDR)?;

        Ok(Self {
            socket,
            predictor: PredictionEngine::new(),
            interpolator: InterpolationBuffer::new(),
        })
    }

    fn update(&mut self) {
        todo!("1. Sample WASD keys and mouse aiming\n2. Call predictor.apply_input and send InputPacket over UDP\n3. Drain incoming StateSnapshots from UDP socket\n4. Reconcile predictor & push to interpolator")
    }

    fn draw(&self) {
        todo!("Clear background, draw arena boundaries, draw interpolated remote entities (red), draw local predicted player (green), draw HUD")
    }
}

fn window_config() -> Conf {
    Conf {
        window_title: "netcode-shooter".to_owned(),
        window_width: 900,
        window_height: 700,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut client = match GameClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize client: {e}");
            return;
        }
    };

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // client.update();
        // client.draw();

        next_frame().await;
    }
}
