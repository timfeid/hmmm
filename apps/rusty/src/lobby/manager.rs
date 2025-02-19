use futures::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use futures::{future, Stream};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tokio::sync::{broadcast, mpsc};
use tokio::task;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use ulid::Ulid;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::lobby::{Lobby, LobbyData};
use crate::error::{AppError, AppResult};
use crate::http::controllers::lobby::PersonalizedGameData;
use crate::services::jwt::{Claims, JwtService};

#[derive(Clone)]
pub struct LobbyManager {
    lobbies: Arc<Mutex<HashMap<String, Arc<Mutex<Lobby>>>>>,
}

#[derive(Type, Deserialize, Clone, Serialize, Debug)]
pub struct LobbyTurnMessage {
    pub messages: Vec<String>,
}

#[derive(Type, Deserialize, Clone, Serialize)]
pub struct ModalButton {
    pub id: String,
    pub text: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub action: Option<
        Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>,
    >,
}
impl ModalButton {
    pub(crate) fn new<F>(text: &str, action: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync + 'static,
    {
        Self {
            id: Ulid::new().to_string(),
            text: text.to_string(),
            action: Some(Arc::new(action)),
        }
    }
}

impl std::fmt::Debug for ModalButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalButton")
            .field("id", &self.id)
            .field("text", &self.text)
            .finish()
    }
}

impl std::fmt::Debug for LobbyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LobbyManager")
            .field("lobbies", &self.lobbies)
            .finish()
    }
}

impl LobbyManager {
    pub async fn create_lobby(self: &Arc<Self>, user: &Claims) -> AppResult<String> {
        let mut lobbies = self.lobbies.lock().await;
        let lobby = Lobby::new(user).await;
        let lobby_id = lobby.data.join_code.clone();
        let lobby_manager_weak = Arc::downgrade(self);
        let lobby_id_clone = lobby_id.clone();

        lobbies.insert(lobby_id.clone(), Arc::new(Mutex::new(lobby)));

        // tokio::spawn(async move {
        //     let rx = {
        //         let game = game_arc_clone.lock().await;
        //         game.broadcast_sender
        //             .as_ref()
        //             .map(|sender| sender.subscribe())
        //     };

        //     if let Some(mut rx) = rx {
        //         while let Ok(message) = rx.recv().await {
        //             if let Some(lobby_manager) = lobby_manager_weak.upgrade() {
        //                 if let Some(command) = message {
        //                     lobby_manager
        //                         .send_command(&lobby_id_clone, command)
        //                         .await
        //                         .ok();
        //                 } else {
        //                     lobby_manager.notify_lobby(&lobby_id_clone).await.ok();
        //                 }
        //             } else {
        //                 // The LobbyManager has been dropped; exit the task
        //                 break;
        //             }
        //         }
        //     }
        // });

        Ok(lobby_id)
    }

    pub async fn get_lobby(&self, join_code: &String) -> AppResult<Arc<Mutex<Lobby>>> {
        // Lock the `lobbies` to get the lobby reference.
        let lobbies = self.lobbies.lock().await;

        // Find the specific lobby or return an error if it doesn't exist.
        let lobby = lobbies
            .get(join_code)
            .ok_or(AppError::BadRequest("Lobby not found".to_owned()))?
            .clone();

        Ok(lobby)
    }

    pub async fn subscribe_to_lobby_updates(
        &self,
        lobby_id: String,
        claims: Claims,
    ) -> AppResult<impl tokio_stream::Stream<Item = PersonalizedGameData>> {
        // Look up the lobby.
        let lobby_arc = {
            let lobbies = self.lobbies.lock().await;
            lobbies
                .get(&lobby_id)
                .ok_or(AppError::BadRequest("Lobby not found".to_owned()))?
                .clone()
        };

        // Lock the lobby briefly to extract the broadcast sender.
        let pub_tx = {
            let lobby = lobby_arc.lock().await;
            lobby.pub_tx.clone().ok_or(AppError::InternalServerError(
                "PubSub not initialized".to_owned(),
            ))?
        };

        // Subscribe to the broadcast channel.
        let rx = pub_tx.subscribe();

        // Wrap the broadcast receiver in a stream and map the data.
        let stream = BroadcastStream::new(rx).filter_map(move |result| {
            let claims_cl = claims.clone();
            async move {
                match result {
                    Ok(data) => Some(PersonalizedGameData::new(&data, &claims_cl.sub).await),
                    Err(e) => {
                        eprintln!("Error receiving broadcast: {:?}", e);
                        None
                    }
                }
            }
        });

        Ok(stream)
    }

    /// Notifies the lobby of an update.
    pub async fn notify_lobby(&self, lobby_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Look up the lobby.
        let lobby_arc = {
            let lobbies = self.lobbies.lock().await;
            lobbies.get(lobby_id).ok_or("Lobby not found")?.clone()
        };

        // Lock the lobby to extract both data and the broadcast sender.
        let (lobby_data, pub_tx) = {
            let lobby = lobby_arc.lock().await;
            (
                lobby.data.clone(),
                lobby.pub_tx.clone().ok_or("PubSub not initialized")?,
            )
        };

        // Send the updated data.
        pub_tx.send(lobby_data)?;
        Ok(())
    }

    pub async fn join_lobby(&self, lobby_id: &str, user: &Claims) -> Option<()> {
        {
            let hash_map = self.lobbies.lock().await;
            let lobby = hash_map.get(lobby_id)?;
            lobby.lock().await.join(user).await;
        }
        // lobby.lock().await.message(user, args.text);
        self.notify_lobby(lobby_id).await.ok();

        Some(())
    }

    // pub async fn notify_lobby(&self, lobby_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     // Look up the lobby.
    //     let lobby_arc = {
    //         let lobbies = self.lobbies.lock().await;
    //         lobbies.get(lobby_id).ok_or("Lobby not found")?.clone()
    //     };

    //     // Lock the lobby once to extract both the current data and the broadcast sender.
    //     let (lobby_data, pub_tx) = {
    //         let lobby = lobby_arc.lock().await;
    //         (
    //             lobby.data.clone(),
    //             lobby.pub_tx.clone().ok_or("PubSub not initialized")?,
    //         )
    //     };

    //     // Send the updated data.
    //     pub_tx.send(lobby_data)?;
    //     Ok(())
    // }

    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
