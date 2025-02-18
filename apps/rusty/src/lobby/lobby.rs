use std::{borrow::BorrowMut, collections::HashMap, sync::Arc, thread::Thread};

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
pub struct Coordinates {
    x: i32,
    y: i32,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]

pub struct VisibleObject {
    id: String,
    x: i32,
    y: i32,
    rotation: f32,
    velocity: Coordinates,
    owner_id: String,
    r#type: VisibleObjectType,
    hidden: bool,
    animation: Option<String>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum VisibleObjectType {
    Person(PersonSkin),
    // starting speed
    // speed decay over acceleration
    // top speed
    // break power
    Car(CarSkin, i32, i32),
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum CarSkin {
    Sedan,
    Police,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum PersonSkin {
    Default,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct GameState {
    visible_objects: HashMap<String, VisibleObject>,
}
impl GameState {
    fn default() -> GameState {
        let mut hash = HashMap::new();
        hash.insert(
            "tim's person".to_string(),
            VisibleObject {
                hidden: false,
                x: 455,
                y: 789,
                id: "tim's person".to_string(),
                rotation: 1.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_id: "tim".to_string(),
                r#type: VisibleObjectType::Person(PersonSkin::Default),
                animation: Some("idle".to_string()),
            },
        );
        hash.insert(
            "tim's car".to_string(),
            VisibleObject {
                hidden: false,
                x: 455,
                id: "tim's car".to_string(),
                y: 789,
                rotation: 1.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_id: "tim".to_string(),
                r#type: VisibleObjectType::Car(CarSkin::Sedan, 150, 3),
                animation: Some("idle".to_string()),
            },
        );
        hash.insert(
            "bob's person".to_string(),
            VisibleObject {
                hidden: false,
                id: "bob's person".to_string(),
                x: 527,
                y: 789,
                rotation: 180.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_id: "bob".to_string(),
                r#type: VisibleObjectType::Person(PersonSkin::Default),
                animation: Some("idle".to_string()),
            },
        );
        hash.insert(
            "bob's car".to_string(),
            VisibleObject {
                hidden: false,
                id: "bob's car".to_string(),
                x: 527,
                y: 789,
                rotation: 180.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_id: "bob".to_string(),
                r#type: VisibleObjectType::Car(CarSkin::Police, 250, 4),
                animation: Some("idle".to_string()),
            },
        );
        GameState {
            visible_objects: hash,
        }
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
            if object.owner_id == user_id {
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
