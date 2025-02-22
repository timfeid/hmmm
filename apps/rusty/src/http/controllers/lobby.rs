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
use tokio::{
    sync::{Mutex, MutexGuard},
    time::interval,
};
use tokio_stream::StreamExt;

use crate::{
    error::{AppError, AppResult},
    gangsta::{map::Coordinates, GameObjectType, OutgoingGameObject, PlayerInput},
    http::context::Ctx,
    lobby::{
        lobby::{Lobby, LobbyChat, LobbyData},
        manager::LobbyManager,
    },
    services::jwt::{Claims, JwtService},
};

#[derive(Type, Serialize, Deserialize, Debug)]
pub struct PersonalizedGameData {
    visible_objects: HashMap<String, OutgoingGameObject>,
}

impl PersonalizedGameData {
    pub async fn new(command: &LobbyData, user_id: &str) -> PersonalizedGameData {
        let visible_objects = {
            let mut visible_objects = HashMap::new();
            let game = command.game.get_state().lock().await;
            for (object_id, obj) in game.players.iter() {
                visible_objects.insert(object_id.clone(), obj.to_outgoing_game_object());
            }
            for (object_id, obj) in game.objects.iter() {
                match &obj.details {
                    GameObjectType::Car(car) => {
                        visible_objects.insert(object_id.clone(), car.to_outgoing_game_object());
                    }
                }
            }

            visible_objects
        };

        PersonalizedGameData { visible_objects }
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
    pub r: f32,
    pub x: i32,
    pub y: i32,
}

impl LobbyController {
    pub async fn ready(ctx: Ctx, code: String) -> AppResult<()> {
        let user = ctx.required_user()?;

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

        let lobby_clone = lobby.clone();
        let code_clone = code.clone();
        let lobby_manager = ctx.lobby_manager.clone();

        tokio::spawn(async move {
            let tick_duration = Duration::from_millis(50);
            let mut ticker = interval(tick_duration);
            loop {
                ticker.tick().await;
                {
                    let mut lobby_guard = lobby_clone.lock().await;
                    lobby_guard.data.game.tick().await;
                }

                lobby_manager.notify_lobby(&code_clone).await.ok();
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
                PlayerInput {
                    rotation: args.r,
                    x: args.x,
                    y: args.y,
                },
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
