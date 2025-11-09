// =============================================================================
// NETWORKING - Client
// =============================================================================

use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::Message;

#[derive(Resource)]
pub struct NetworkClient {
    outgoing: Sender<Vec<u8>>,
    incoming: Arc<Mutex<VecDeque<shared::protocol::ServerMessage>>>,
    connected: Arc<Mutex<bool>>,
    reconnect_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl NetworkClient {
    pub fn connect(server_url: &str) -> Result<Self, String> {
        info!("Connecting to {}", server_url);

        // Parse URL to extract host:port

        let (tx, rx) = channel::<Vec<u8>>();
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let connected = Arc::new(Mutex::new(true));
        let reconnect_handle = Arc::new(Mutex::new(None));

        // Spawn a thread to read messages
        let incoming_clone = incoming.clone();
        let connected_clone = connected.clone();
        let server_url_str = server_url.to_string();

        let handle = thread::spawn(move || {
            let mut retry_count = 0u32;
            let max_retries = 10;

            loop {
                match Self::establish_connection(&server_url_str, &rx, &incoming_clone, &connected_clone) {
                    Ok(_) => {
                        retry_count = 0;
                    }
                    Err(e) => {
                        error!("Connection lost of failed: {}", e);
                        *connected_clone.lock().unwrap() = false;

                        if retry_count < max_retries {
                            let mut delay_ms = 1000 * 2u64.pow(retry_count);
                            delay_ms = delay_ms.min(30000);

                            warn!(
                                "Reconneting in {}ms (attempt {}/{})",
                                delay_ms,
                                retry_count + 1,
                                max_retries
                            );

                            thread::sleep(std::time::Duration::from_millis(delay_ms));
                            retry_count += 1;
                        } else {
                            error!("Max retries reached, giving up");
                            break;
                        }
                    }
                }
            }

            info!("Exiting network thread");
        });

        *reconnect_handle.lock().unwrap() = Some(handle);

        info!("✓ Connected to server");

        Ok(Self {
            outgoing: tx,
            incoming,
            connected,
            reconnect_handle,
        })
    }

    pub fn establish_connection(
        server_url: &str,
        rx: &Receiver<Vec<u8>>,
        incoming: &Arc<Mutex<VecDeque<shared::protocol::ServerMessage>>>,
        connected: &Arc<Mutex<bool>>,
    ) -> Result<(), String> {
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

        *connected.lock().unwrap() = true;
        info!("✓ Connected to server");

        loop {
            // Check if disconnected
            if !*connected.lock().unwrap() {
                break;
            }

            let mut sent_any = false;

            while let Ok(data) = rx.try_recv() {
                info!("Sending {} bytes to server", data.len());
                if let Err(e) = socket.write(Message::Binary(data.into())) {
                    error!("Write error: {}", e);
                    *connected.lock().unwrap() = false;
                    return Err(format!("Write error: {}", e));
                }
                info!("✓ Message sent");
                sent_any = true;
            }

            // Flush immediately
            if sent_any {
                if let Err(e) = socket.flush() {
                    error!("Flush error: {}", e);
                    *connected.lock().unwrap() = false;
                    return Err(format!("Flush error: {}", e));
                }
                info!("✓ Messages flushed");
            }

            match socket.can_read() {
                true => {
                    match socket.read() {
                        Ok(Message::Binary(data)) => {
                            info!("Received {} bytes from server", data.len());
                            match bincode::decode_from_slice(&data[..], bincode::config::standard())
                            {
                                Ok((server_msg, _)) => {
                                    // info!("✓ Deserialized ServerMessage: {:?}", server_msg);
                                    incoming.lock().unwrap().push_back(server_msg);
                                }
                                Err(e) => {
                                    error!("Deserialize error: {}", e);
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("Server closed connection");
                            return Err("Server closed".to_string());
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
                            // thread::sleep(std::time::Duration::from_millis(10));
                            return Err(format!("Read error: {}", e));
                        }
                    }
                }
                false => {
                    // No data available
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        Err("Connection closed".to_string())
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
