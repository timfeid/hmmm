use std::{
    sync::Arc,
    thread::{sleep, Thread},
    time::Duration,
};

use async_stream::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, MutexGuard};
use tokio_stream::StreamExt;

use crate::{
    error::{AppError, AppResult},
    http::context::Ctx,
    lobby::{
        lobby::{DeckSelector, Lobby, LobbyChat, LobbyData},
        manager::{LobbyCommand, LobbyManager},
    },
    services::jwt::{Claims, JwtService},
};

fn personalize_lobby_data_for_player(command: &mut LobbyCommand, user_id: &str) {
    // if let LobbyCommand::Updated(lobby_data) = command {
    //     for (id, player_state) in &mut lobby_data.game_state.players {
    //         if id != user_id {
    //             player_state.hand.clear();
    //         }
    //     }
    // }
}

pub struct LobbyController {}

#[derive(Type, Deserialize, Debug)]
pub struct LobbyInputArgs {
    access_token: String,
    lobby_id: String,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub rotation: f32,
    pub x: i32,
    pub y: i32,
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
                sleep(Duration::from_millis(150));
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
        // let user = ctx.required_user()?;
        let user_claims = JwtService::decode(&args.access_token).unwrap().claims;
        let lobby = ctx
            .lobby_manager
            .get_lobby(&args.lobby_id)
            .await
            .map_err(|x| AppError::BadRequest("Bad lobby id".to_string()))?;
        // let data = &lobby.lock().await.data;

        // println!("adding message to lobby {} {:?}", data.join_code, lobby);

        // println!("{:?}", args);
        lobby.lock().await.input(args, user_claims.sub).await;
        // println!("added, notifying lobby");
        // lobby.lock().await.message(user, args.text);
        // ctx.lobby_manager.notify_lobby(&args.lobby_id).await.ok();

        Ok(())
    }

    pub(crate) fn subscribe(
        ctx: Ctx,
        join_code: String,
        access_token: String,
    ) -> impl Stream<Item = LobbyCommand> + Send + 'static {
        let manager = Arc::clone(&ctx.lobby_manager);
        let user_claims = JwtService::decode(&access_token).unwrap().claims;

        let async_stream = stream! {
            if let Ok(mut post_stream) = manager.subscribe_to_lobby_updates(join_code, access_token).await {
                while let Some(mut lobby_data) = post_stream.next().await {
                        match &lobby_data {
                            // LobbyCommand::ChooseFromSelection(ability_details) => {
                            //     if ability_details.player_id == user_claims.sub.clone() {
                            //         yield lobby_data;
                            //     }
                            // },
                            // LobbyCommand::MandatoryExecuteAbility(ability_details) => {
                            //     if ability_details.player_id == user_claims.sub.clone() {
                            //         yield lobby_data;
                            //     }
                            // },
                            // LobbyCommand::AskExecuteAbility(ability_details) => {
                            //     if ability_details.player_id == user_claims.sub.clone() {
                            //         yield lobby_data;
                            //     }
                            // },
                            _ => {
                                personalize_lobby_data_for_player(&mut lobby_data, &user_claims.sub);

                                yield lobby_data;
                            }
                        }


                }
            }
        };
        let async_stream = async_stream;
        async_stream
    }
}
