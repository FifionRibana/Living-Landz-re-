use bevy::prelude::*;
use shared::protocol::ServerMessage;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub type MessageSender = mpsc::UnboundedSender<ServerMessage>;

#[derive(Clone)]
pub struct SessionData {
    pub player_id: Option<u64>, // None avant login, Some(player_id) après
    pub addr: SocketAddr,
    pub sender: MessageSender,
}

#[derive(Resource, Clone)]
pub struct Sessions {
    // session_id -> SessionData
    sessions: Arc<RwLock<HashMap<u64, SessionData>>>,
    // player_id -> session_id (pour envoyer des messages aux joueurs)
    player_to_session: Arc<RwLock<HashMap<u64, u64>>>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            player_to_session: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Sessions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insère une nouvelle session (avant le login)
    pub async fn insert(&self, session_id: u64, addr: SocketAddr, sender: MessageSender) {
        let session_data = SessionData {
            player_id: None, // Pas encore authentifié
            addr,
            sender,
        };
        self.sessions.write().await.insert(session_id, session_data);
    }

    /// Associe un player_id à une session après le login
    pub async fn authenticate_session(&self, session_id: u64, player_id: u64) {
        let mut sessions = self.sessions.write().await;
        if let Some(session_data) = sessions.get_mut(&session_id) {
            session_data.player_id = Some(player_id);

            // Créer le mapping player_id -> session_id
            self.player_to_session.write().await.insert(player_id, session_id);

            tracing::info!("Session {} authenticated as player {}", session_id, player_id);
        }
    }

    /// Retire une session
    pub async fn remove(&self, session_id: &u64) {
        let mut sessions = self.sessions.write().await;

        // Si la session avait un player_id, retirer le mapping inverse
        if let Some(session_data) = sessions.remove(session_id) {
            if let Some(player_id) = session_data.player_id {
                self.player_to_session.write().await.remove(&player_id);
            }
        }
    }

    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Envoie un message à un joueur spécifique (par player_id)
    pub async fn send_to_player(&self, player_id: u64, message: ServerMessage) -> Result<(), String> {
        // Trouver la session correspondant au player_id
        let player_to_session = self.player_to_session.read().await;
        let session_id = player_to_session.get(&player_id)
            .ok_or_else(|| format!("Player {} not found in sessions", player_id))?;

        // Envoyer le message via la session
        let sessions = self.sessions.read().await;
        if let Some(session_data) = sessions.get(session_id) {
            session_data.sender.send(message)
                .map_err(|e| format!("Failed to send message to player {}: {}", player_id, e))?;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Broadcast un message à tous les joueurs connectés
    pub async fn broadcast(&self, message: ServerMessage) {
        let sessions = self.sessions.read().await;
        for (_session_id, session_data) in sessions.iter() {
            let _ = session_data.sender.send(message.clone());
        }
    }
}
