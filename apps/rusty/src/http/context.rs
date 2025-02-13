use std::sync::Arc;

use axum::http::request::Parts;
use sqlx::{Pool, Postgres};

use crate::{
    error::{AppError, AppResult},
    lobby::manager::LobbyManager,
    services::jwt::{Claims, JwtService},
};

#[derive(Debug)]
pub struct Ctx {
    pub pool: Arc<Pool<Postgres>>,
    user: Option<Claims>,
    pub lobby_manager: Arc<LobbyManager>,
}

impl Ctx {
    pub fn new(pool: Arc<Pool<Postgres>>, parts: Parts, lobby_manager: Arc<LobbyManager>) -> Ctx {
        // println!("{:?}", parts.headers);
        let user = match parts.headers.get("Authorization") {
            Some(bearer) => JwtService::decode(bearer.to_str().unwrap_or_default())
                .and_then(|r| Ok(r.claims))
                .ok(),
            None => None,
        };

        Ctx {
            pool,
            user,
            lobby_manager,
        }
    }

    pub fn required_user(self: &Ctx) -> AppResult<&Claims> {
        // println!("{:?}", self);
        if self.user.is_none() {
            return Err(AppError::Unauthorized);
        }
        // Err(AppError::Unauthorized)
        Ok(self.user.as_ref().unwrap())
    }
}
