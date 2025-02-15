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
pub struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]

pub struct VisibleUser {
    desired_x: Option<i32>,
    desired_y: Option<i32>,
    x: i32,
    y: i32,
    rotation: f32,
    velocity: Velocity,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct GameState {
    visible_users: HashMap<String, VisibleUser>,
}
impl GameState {
    fn default() -> GameState {
        let mut hash = HashMap::new();
        hash.insert(
            "tim".to_string(),
            VisibleUser {
                x: 455,
                y: 789,
                desired_x: None,
                desired_y: None,
                rotation: 1.57,
                velocity: Velocity { x: 50, y: 0 },
            },
        );
        hash.insert(
            "bob".to_string(),
            VisibleUser {
                x: 527,
                y: 789,
                desired_x: None,
                desired_y: None,
                rotation: 180.57,
                velocity: Velocity { x: -50, y: 0 },
            },
        );
        GameState {
            visible_users: hash,
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
    // #[serde(skip_serializing, skip_deserializing)]
    // game: Arc<Mutex<Game>>,
}

impl Lobby {
    // pub async fn get_state(&self) -> PublicGameInfo {
    //     let priority_queue = {
    //         let game = self.game.lock().await;
    //         if let Some((player, time_left, _)) = &game.current_priority_player {
    //             Some(PriorityQueue {
    //                 player_id: player.lock().await.name.clone(),
    //                 time_left: time_left.clone(),
    //             })
    //         } else {
    //             None
    //         }
    //     };

    //     let combat = self.game.lock().await.combat.clone();
    //     let blocks = {
    //         let mut blocks = vec![];
    //         for (blocker, attacker) in combat.blockers.iter() {
    //             blocks.push(Block {
    //                 attacker: {
    //                     Game::frontend_target_from_card(&self.game, attacker)
    //                         .await
    //                         .expect("hm")
    //                 },
    //                 blocker: {
    //                     Game::frontend_target_from_card(&self.game, blocker)
    //                         .await
    //                         .expect("hm")
    //                 },
    //             })
    //         }

    //         blocks
    //     };

    //     let attacks = {
    //         let mut attacks = vec![];
    //         let cloned_game = self.cloned_game();
    //         let game = cloned_game.lock().await;
    //         if let Some(turn) = game.current_turn.clone() {
    //             let player = &turn.current_player;
    //             let player_id = turn.current_player.lock().await.name.clone();
    //             let cards = player.lock().await.cards_in_play.clone();
    //             for (index, card) in cards.iter().enumerate() {
    //                 for (attacker, target) in game.combat.attackers.iter() {
    //                     if Arc::ptr_eq(attacker, card) {
    //                         attacks.push(Attack {
    //                             target: target.clone(),
    //                             attacker: FrontendCardTarget {
    //                                 player_id: player_id.clone(),
    //                                 pile: FrontendPileName::Play,
    //                                 card_index: index as i32,
    //                             },
    //                         });
    //                     }
    //                 }
    //             }
    //         }

    //         attacks
    //     };

    //     PublicGameInfo {
    //         current_turn: self.game.lock().await.current_turn.clone(),
    //         priority_queue,
    //         attacks,
    //         blocks,
    //     }
    // }

    // pub fn cloned_game(&self) -> Arc<Mutex<Game>> {
    //     Arc::clone(&self.game)
    // }
}

use redis::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use ulid::Ulid;

#[derive(Type, Deserialize, Clone, Serialize, Debug)]
pub enum DeckSelector {
    // Elves,
    // Elves2,
    // Blue,
    // Black,
    // Angels,
    Vegas,
    // AngelsBlue,
    // Red,
}

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
        if let Some(player) = self.data.game_state.visible_users.get_mut(&user_id) {
            player.rotation = args.rotation;
            player.x = args.x;
            player.y = args.y;
            // player.velocity = args
            // for user in &self.data.game_state.visible_users.iter() {

            // }
            // self.data.game_state.visible_users = PlayerStatus::Ready;
            // let mut p = player.player.lock().await;
            // let mut deck = Deck::new_from_selection(&player.deck);
            // deck.set_owner(&player.player).await;

            // p.deck = deck;
        }

        self
    }

    pub async fn ready(&mut self, user: &Claims) -> &mut Self {
        // if let Some(player) = self.data.game_state.players.get_mut(&user.sub) {
        //     player.status = PlayerStatus::Ready;
        //     let mut p = player.player.lock().await;
        //     let mut deck = Deck::new_from_selection(&player.deck);
        //     deck.set_owner(&player.player).await;

        //     p.deck = deck;
        // }

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
