use futures::stream::StreamExt;
use futures::Stream;
use redis::aio::PubSub;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tokio::sync::mpsc;
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
use crate::services::jwt::{Claims, JwtService};

#[derive(Clone)]
pub struct LobbyManager {
    redis_client: Arc<redis::Client>,
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

#[derive(Type, Clone, Deserialize, Serialize, Debug)]
#[specta(export = false)]
pub enum LobbyCommand {
    Updated(LobbyData),
    Messages(Vec<String>),
    DebugMessage(String),
    TurnMessages(LobbyTurnMessage),
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

    // Stream game updates from Redis for a specific lobby
    pub async fn subscribe_to_lobby_updates(
        &self,
        lobby_id: String,
        access_token: String,
    ) -> AppResult<impl Stream<Item = LobbyCommand>> {
        let user = JwtService::decode(&access_token).or(Err(AppError::Unauthorized))?;
        let (tx, rx) = mpsc::channel::<LobbyCommand>(100);

        println!("{:?} has joined!", user.claims);

        // Clone redis client so it can be passed into the async block.
        let redis_client = Arc::clone(&self.redis_client);

        // Spawn the Redis subscription in a new task, but keep the mutex scope minimal
        tokio::spawn(async move {
            if let Err(e) = Self::handle_lobby_subscription(redis_client, lobby_id, tx).await {
                eprintln!("Error in subscription: {:?}", e);
            }
        });

        // Return the receiver stream
        Ok(ReceiverStream::new(rx))
    }

    // This function handles the subscription logic to keep the original method clean
    async fn handle_lobby_subscription(
        redis_client: Arc<redis::Client>,
        lobby_id: String,
        tx: mpsc::Sender<LobbyCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pubsub_conn = redis_client.get_async_pubsub().await?;
        pubsub_conn.subscribe(&lobby_id).await?;

        let mut pubsub_stream = pubsub_conn.on_message();
        while let Some(message) = pubsub_stream.next().await {
            let payload: String = message.get_payload()?;
            if let Ok(game) = serde_json::from_str::<LobbyCommand>(&payload) {
                if tx.send(game).await.is_err() {
                    eprintln!("Receiver dropped");
                    break;
                }
            }
        }
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

    pub async fn send_command(
        &self,
        lobby_id: &str,
        command: LobbyCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Step 1: Get the Redis connection
        let mut redis_conn = self.redis_client.get_multiplexed_async_connection().await?;

        let lobby_data = serde_json::to_string(&command)?;
        // Step 5: Publish the data to Redis.
        redis_conn.publish(lobby_id, lobby_data).await?;

        Ok(())
    }

    pub async fn notify_lobby(&self, lobby_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // self.update_game_state(lobby_id).await;

        // Step 1: Get the Redis connection
        let mut redis_conn = self.redis_client.get_multiplexed_async_connection().await?;

        // Step 2: Lock `lobbies` and extract the `lobby` reference.
        let lobby = {
            let lobbies = self.lobbies.lock().await;
            let lobby = lobbies.get(lobby_id).ok_or("Lobby not found")?.clone();
            lobby // release lobbies lock here
        };

        // Step 3: Now lock the `lobby` with a timeout to detect potential deadlock.
        let data = LobbyCommand::Updated({
            let lobby = match timeout(Duration::from_secs(5), lobby.lock()).await {
                Ok(lock) => lock.data.clone(),
                Err(_) => {
                    eprintln!("Timeout trying to acquire lobby lock");
                    return Err("Timeout while locking lobby".into());
                }
            };
            lobby
        });

        // Step 4: Serialize the lobby data.
        let lobby_data = serde_json::to_string(&data)?;

        // Step 5: Publish the data to Redis.
        redis_conn.publish(lobby_id, lobby_data).await?;

        Ok(())
    }

    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis_client: Arc::new(client),
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
