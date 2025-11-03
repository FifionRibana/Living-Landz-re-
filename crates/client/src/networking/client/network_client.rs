// =============================================================================
// NETWORKING - Client
// =============================================================================

use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::Message;

#[derive(Resource)]
pub struct NetworkClient {
    outgoing: Sender<Vec<u8>>,
    incoming: Arc<Mutex<VecDeque<shared::protocol::ServerMessage>>>,
    connected: Arc<Mutex<bool>>,
}

impl NetworkClient {
    pub fn connect(server_url: &str) -> Result<Self, String> {
        info!("Connecting to {}", server_url);

        // Parse URL to extract host:port
        let url = url::Url::parse(server_url).map_err(|e| format!("Invalid URL: {}", e))?;
        let host = url.host_str().ok_or("No host in URL")?;
        let port = url.port().unwrap_or(9001);

        // Connect TCP directly
        let tcp_stream = std::net::TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| format!("TCP connection failed: {}", e))?;

        info!("TCP connected, upgrading to WebSocket...");

        let (mut socket, _) = tungstenite::client(server_url, tcp_stream)
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Set non-blocking BEFORE WebSocket upgrade
        socket
            .get_mut()
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        let (tx, rx) = channel::<Vec<u8>>();
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let connected = Arc::new(Mutex::new(true));

        // Spawn a thread to read messages
        let incoming_clone = incoming.clone();
        let connected_clone = connected.clone();

        thread::spawn(move || {
            loop {
                // Check if disconnected
                if !*connected_clone.lock().unwrap() {
                    break;
                }

                let mut sent_any = false;

                while let Ok(data) = rx.try_recv() {
                    info!("Sending {} bytes to server", data.len());
                    if let Err(e) = socket.write(Message::Binary(data.into())) {
                        error!("Write error: {}", e);
                        *connected_clone.lock().unwrap() = false;
                        return;
                    } else {
                        info!("✓ Message sent");
                    }
                    sent_any = true;
                }

                // Flush immediately
                if sent_any {
                    if let Err(e) = socket.flush() {
                        error!("Flush error: {}", e);
                        *connected_clone.lock().unwrap() = false;
                        return;
                    }
                    info!("✓ Messages flushed");
                }

                match socket.can_read() {
                    true => {
                        match socket.read() {
                            Ok(Message::Binary(data)) => {
                                info!("Received {} bytes from server", data.len());
                                match bincode::decode_from_slice(
                                    &data[..],
                                    bincode::config::standard(),
                                ) {
                                    Ok((server_msg, _)) => {
                                        // info!("✓ Deserialized ServerMessage: {:?}", server_msg);
                                        incoming_clone.lock().unwrap().push_back(server_msg);
                                    }
                                    Err(e) => {
                                        error!("Deserialize error: {}", e);
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                info!("Server closed connection");
                                *connected_clone.lock().unwrap() = false;
                                break;
                            }
                            Ok(_) => {}
                            Err(tungstenite::Error::Io(ref e))
                                if e.kind() == std::io::ErrorKind::WouldBlock =>
                            {
                                // No-blocking read, no data available
                                thread::sleep(std::time::Duration::from_millis(10));
                            }
                            Err(e) => {
                                error!("Error reading: {}", e);
                                thread::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    }
                    false => {
                        // No data available
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }

            info!("Exiting network thread");
        });

        info!("✓ Connected to server");

        Ok(Self {
            outgoing: tx,
            incoming,
            connected,
        })
    }

    pub fn send_message(&mut self, message: shared::protocol::ClientMessage) {
        if !*self.connected.lock().unwrap() {
            warn!("Cannot send message, not connected");
            return;
        }

        match bincode::encode_to_vec(&message, bincode::config::standard()) {
            Ok(data) => {
                info!(
                    "Queuing message ({} bytes) to server: {:?}",
                    data.len(),
                    message
                );
                if let Err(e) = self.outgoing.send(data) {
                    error!("Failed to queue message: {}", e);
                    *self.connected.lock().unwrap() = false;
                }
            }
            Err(e) => {
                error!("Serialize error: {}", e);
            }
        }
    }

    pub fn poll_messages(&mut self) -> Vec<shared::protocol::ServerMessage> {
        let mut messages = self.incoming.lock().unwrap();
        messages.drain(..).collect()
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
}
