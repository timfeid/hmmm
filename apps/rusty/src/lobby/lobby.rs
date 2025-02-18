use std::{borrow::BorrowMut, collections::HashMap, sync::Arc, thread::Thread, vec};

use futures::StreamExt;

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyChat {
    user_id: String,
    message: String,
}
impl LobbyChat {
    pub fn new(user_id: String, message: String) -> Self {
        Self { user_id, message }
    }
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyData {
    pub join_code: String,
    pub chat: Vec<LobbyChat>,
    pub game_state: GameState,
}
impl Default for LobbyData {
    fn default() -> LobbyData {
        let mut game_state = GameState::default();
        let code = ulid::Ulid::new().to_string();
        // game_state.code = code.clone();

        LobbyData {
            join_code: code,
            chat: vec![],
            game_state: game_state,
        }
    }
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct Lobby {
    #[serde(skip_serializing, skip_deserializing)]
    client: Option<Client>,

    pub data: LobbyData,
}

impl Lobby {}

use redis::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use ulid::Ulid;

use crate::{
    error::{AppError, AppResult},
    game::GameState,
    http::controllers::lobby::LobbyInputArgs,
    services::jwt::Claims,
};

use super::manager::LobbyManager;

impl Lobby {
    pub async fn new(user: &Claims) -> Self {
        let mut lobby = Lobby {
            data: LobbyData::default(),
            client: None,
        };

        lobby.join(user).await;

        lobby
    }

    pub async fn join(&mut self, user: &Claims) -> &mut Self {
        println!("JOIN {:?}", self);

        self
    }

    pub async fn input(&mut self, args: LobbyInputArgs, user_id: String) -> &mut Self {
        if let Some(object) = self
            .data
            .game_state
            .visible_objects
            .get_mut(&args.object_id)
        {
            if object.owner_user_id == user_id {
                object.rotation = args.rotation;
                object.x = args.x;
                object.y = args.y;
                object.hidden = args.hidden;
                object.animation = args.animation;
            }
        }

        self
    }

    pub async fn ready(&mut self, user: &Claims) -> &mut Self {
        self
    }

    pub fn message(&mut self, user: &Claims, message: String) -> &mut Self {
        self.data
            .chat
            .push(LobbyChat::new(user.sub.clone(), message));

        self
    }
}

mod test {
    use std::{cell::RefCell, rc::Rc};

    use tokio_stream::StreamExt;

    use crate::{lobby::lobby::Lobby, services::jwt::Claims};

    #[tokio::test]
    async fn test() {
        let user_id = Claims {
            sub: "boob".to_string(),
            jti: Some("boob".to_string()),
            exp: 0,
        };
        let user_id2 = Claims {
            sub: "sakdfakjs".to_string(),
            jti: Some("asdkjfjskd".to_string()),
            exp: 0,
        };
        let lobby = &Rc::new(RefCell::new(Lobby::new(&user_id).await));
        let redis_url = "redis://127.0.0.1/".to_string();
        let redis = redis::Client::open(redis_url).unwrap();

        // async_stream::stream! {
        //     // let mut post_stream = lobby.clone().borrow_mut().subscribe(redis);
        //     while let Some(post) = post_stream.next().await {
        //         println!("{:?}", post);
        //         yield post;
        //     }
        // };

        lobby
            .clone()
            .borrow_mut()
            .join(&user_id2)
            .await
            .message(&user_id2, "test".to_string());
    }
}
