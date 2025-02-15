use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use rspc::internal::UnbuiltProcedureBuilder;
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use serde::Serialize;
use specta::Type;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::AppError;
use crate::http::context::Ctx;
use crate::http::controllers::lobby::LobbyInputArgs;
use crate::services::jwt::JwtService;
use crate::{http::controllers::lobby::LobbyController, lobby::lobby::LobbyData};

pub fn create_lobby_router() -> rspc::RouterBuilder<Ctx> {
    Router::<Ctx>::new()
        // .mutation("chat", |t| {
        //     t(|ctx, args: LobbyChatArgs| async move { Ok(LobbyController::chat(ctx, args).await?) })
        // })
        .mutation("join", |t| {
            t(|ctx, code: String| async move { Ok(LobbyController::join(ctx, code).await?) })
        })
        .mutation("ready", |t| {
            t(|ctx, code: String| async move { Ok(LobbyController::ready(ctx, code).await?) })
        })
        .mutation("input", |t| {
            t(|ctx, args: LobbyInputArgs| async move { Ok(LobbyController::input(ctx, args).await?) })
        })
        .mutation("create", |t| {
            t(|ctx, _: Vec<String>| async move { Ok(LobbyController::create(ctx).await?) })
        })
        .subscription("subscribe", |t| {
            t(|ctx, (code, access_token): (String, String)| {
                LobbyController::subscribe(ctx, code, access_token)
            })
        })
}
