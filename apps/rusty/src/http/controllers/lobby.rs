use std::{
    collections::HashMap,
    sync::Arc,
    thread::{sleep, Thread},
    time::Duration,
};

use async_stream::stream;
use futures::{pin_mut, Stream};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, MutexGuard};
use tokio_stream::StreamExt;

use crate::{
    error::{AppError, AppResult},
    gangsta::GameObject,
    http::context::Ctx,
    lobby::{
        lobby::{Lobby, LobbyChat, LobbyData},
        manager::LobbyManager,
    },
    services::jwt::{Claims, JwtService},
};

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct PersonalizedGameData {
    visible_objects: HashMap<String, GameObject>,
}

impl PersonalizedGameData {
    pub async fn new(command: &LobbyData, user_id: &str) -> PersonalizedGameData {
        // if let LobbyCommand::Updated(lobby_data) = command {
        //     for (id, player_state) in &mut lobby_data.game_state.players {
        //         if id != user_id {
        //             player_state.hand.clear();
        //         }
        //     }
        // }
        PersonalizedGameData {
            visible_objects: command
                .game
                .get_state()
                .lock()
                .await
                .clone()
                .visible_objects,
        }
    }
}

pub struct LobbyController {}

#[derive(Type, Deserialize, Debug)]
pub struct LobbyActionArgs {
    access_token: String,
    lobby_id: String,
    pub action_id: String,
}

#[derive(Type, Deserialize, Debug)]
pub struct LobbyInputArgs {
    access_token: String,
    lobby_id: String,
    pub object_id: String,
    pub rotation: f32,
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
    pub animation: Option<String>,
}

impl LobbyController {
    pub async fn ready(ctx: Ctx, code: String) -> AppResult<()> {
        let user = ctx.required_user()?;

        // Step 1: Get the lobby instance from the lobby manager and release the lock
        let l = Arc::clone(&ctx.lobby_manager);
        let lobby = l
            .get_lobby(&code)
            .await
            .map_err(|_| AppError::BadRequest("No such lobby".to_string()))?;

        lobby.lock().await.ready(user).await;

        ctx.lobby_manager.notify_lobby(&code).await.ok();

        Ok(())
    }

    pub async fn create(ctx: Ctx) -> AppResult<LobbyData> {
        let user = ctx.required_user()?;
        let code = ctx.lobby_manager.create_lobby(user).await?;
        let lobby = ctx
            .lobby_manager
            .get_lobby(&code)
            .await
            .map_err(|x| AppError::BadRequest("No such lobby".to_string()))?;
        let data = lobby.lock().await.data.clone();
        let mng = Arc::clone(&ctx.lobby_manager);
        let join_code_cloned = code.clone();
        tokio::spawn(async move {
            loop {
                mng.notify_lobby(&join_code_cloned).await.ok();
                sleep(Duration::from_millis(3));
            }
        });

        Ok(data)
    }

    pub(crate) async fn join(ctx: Ctx, join_code: String) -> AppResult<()> {
        let user = ctx.required_user()?;
        ctx.lobby_manager
            .join_lobby(&join_code, user)
            .await
            .ok_or(AppError::BadRequest("Bad lobby id".to_string()))?;

        Ok(())
    }

    pub(crate) async fn input(ctx: Ctx, args: LobbyInputArgs) -> AppResult<()> {
        let user_claims = JwtService::decode(&args.access_token).unwrap().claims;
        let lobby = ctx
            .lobby_manager
            .get_lobby(&args.lobby_id)
            .await
            .map_err(|x| AppError::BadRequest("Bad lobby id".to_string()))?;

        lobby
            .lock()
            .await
            .data
            .game
            .input(
                user_claims.sub,
                args.object_id,
                args.rotation,
                args.x,
                args.y,
                args.hidden,
                args.animation,
            )
            .await;

        Ok(())
    }

    pub(crate) async fn action(ctx: Ctx, args: LobbyActionArgs) -> AppResult<()> {
        let user_claims = JwtService::decode(&args.access_token).unwrap().claims;
        let lobby = ctx
            .lobby_manager
            .get_lobby(&args.lobby_id)
            .await
            .map_err(|x| AppError::BadRequest("Bad lobby id".to_string()))?;

        lobby
            .lock()
            .await
            .data
            .game
            .action(user_claims.sub, args.action_id)
            .await?;

        Ok(())
    }

    pub(crate) fn subscribe(
        ctx: Ctx,
        join_code: String,
        access_token: String,
    ) -> impl Stream<Item = PersonalizedGameData> + Send + 'static {
        let manager = Arc::clone(&ctx.lobby_manager);
        let user_claims = JwtService::decode(&access_token).unwrap().claims;

        async_stream::stream! {
            match manager.subscribe_to_lobby_updates(join_code, user_claims).await {
                Ok(mut post_stream) => {
                    println!("Subscribed to lobby updates");
                    pin_mut!(post_stream);

                    while let Some(item) = post_stream.next().await {
                        yield item;
                    }
                }
                Err(e) => {
                    eprintln!("Error subscribing to lobby updates: {:?}", e);
                }
            }
        }
    }
}
