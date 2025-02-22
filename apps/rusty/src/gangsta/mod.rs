use std::collections::HashMap;

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::{future::Future, pin::Pin};

use action::{Action, ActionBuilder, ActionTrigger, ActionTriggerType};
use axum::async_trait;
use map::{pixel_to_tile, Coordinates, Map};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;
use tokio::time::interval;
use vehicle::Vehicle;

use crate::error::{AppError, AppResult};

pub mod action;
pub mod map;
pub mod traffic_light;
pub mod vehicle;

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
pub struct OutgoingGameObject {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub rotation: f32,
    pub velocity: Coordinates,
    pub owner_user_id: String,
    pub controller_user_id: Option<String>,
    pub details: GameObjectInfo,
    pub action: Option<ActionTrigger>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct PersonDetails {
    pub user_id: String,
    pub skin: PersonSkin,
}

impl PersonDetails {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            skin: PersonSkin::Default,
        }
    }
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct CarDetails {
    pub skin: CarSkin,
    pub speed: u16,
    pub acceleration: u16,
    pub max_passengers: u8,
    pub passenger_user_ids: Vec<String>,
    pub rotation_speed: u16,
    pub driver_user_id: Option<String>,
}

impl CarDetails {
    pub fn new(
        skin: CarSkin,
        speed: u16,
        acceleration: u16,
        rotation_speed: u16,
        max_passengers: u8,
    ) -> Self {
        Self {
            skin,
            speed,
            acceleration,
            max_passengers,
            passenger_user_ids: vec![],
            rotation_speed,
            driver_user_id: None,
        }
    }
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum GameObjectInfo {
    Person(PersonDetails),
    Car(CarDetails),
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
pub struct Player {
    pub id: String,
    pub skin: PersonSkin,
    pub x: i32,
    pub y: i32,
    pub rotation: f32,
    pub velocity: Coordinates,
}

impl Player {
    pub fn to_outgoing_game_object(&self) -> OutgoingGameObject {
        OutgoingGameObject {
            id: self.id.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
            rotation: self.rotation.clone(),
            velocity: self.velocity.clone(),
            owner_user_id: self.id.clone(),
            controller_user_id: Some(self.id.clone()),
            action: None,
            details: GameObjectInfo::Person(PersonDetails {
                user_id: self.id.clone(),
                skin: self.skin.clone(),
            }),
        }
    }

    fn new(id: String) -> Self {
        Self {
            id,
            skin: PersonSkin::Default,
            x: 608,
            y: 800,
            rotation: 0.0,
            velocity: Coordinates { x: 0, y: 0 },
        }
    }

    fn input(&mut self, input: PlayerInput) {
        self.x = input.x;
        self.y = input.y;
        self.rotation = input.rotation;
    }
}

pub trait ToOutgoingGame: Debug + Send + Sized {
    fn to_outgoing_game_object(&self) -> OutgoingGameObject;
}

pub struct GameObject {
    pub details: GameObjectType,
}
impl GameObject {
    async fn tick(&mut self) -> AppResult<()> {
        match &mut self.details {
            GameObjectType::Car(car) => car.tick(),
        }
        Ok(())
    }

    async fn apply_action(
        &mut self,
        get_state: Arc<Mutex<GameState>>,
        user_id: String,
    ) -> AppResult<()> {
        match &mut self.details {
            GameObjectType::Car(car) => car.action(user_id),
        }
        Ok(())
    }
}

pub enum GameObjectType {
    Car(Vehicle),
}

pub struct GameState {
    pub players: HashMap<String, Player>,
    pub objects: HashMap<String, GameObject>,
    pub map: Map,
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameState")
            .field("players", &self.players)
            .finish()
    }
}

impl GameState {
    pub fn default() -> Self {
        let mut players = HashMap::new();
        let mut objects = HashMap::new();
        players.insert("tim".to_string(), Player::new("tim".to_string()));
        players.insert("bob".to_string(), Player::new("bob".to_string()));
        let map = Map::from_json(include_str!("maps/suburb.json")).unwrap();

        let mut vehicle = Vehicle::new(
            "tim's car".to_string(),
            Coordinates { x: 160, y: 368 },
            vehicle::VehicleBehavior::Normal,
            2,
        );

        let path = map
            .find_path(
                pixel_to_tile(vehicle.position),
                pixel_to_tile(Coordinates { x: 1360, y: 1360 }),
            )
            .expect("no path found");

        vehicle.set_tile_path(path);

        objects.insert(
            vehicle.id.clone(),
            GameObject {
                details: GameObjectType::Car(vehicle),
            },
        );

        Self {
            players,
            objects,
            map,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    state: Arc<Mutex<GameState>>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(GameState::default())),
        }
    }
}

pub struct PlayerInput {
    pub x: i32,
    pub y: i32,
    pub rotation: f32,
}

impl Game {
    pub fn get_state(&self) -> &Arc<Mutex<GameState>> {
        &self.state
    }

    pub async fn input(&mut self, user_id: String, input: PlayerInput) -> &Self {
        let mut state = self.get_state().lock().await;
        if let Some(player) = state.players.get_mut(&user_id) {
            player.input(input)
        }

        self
    }

    pub async fn action(&mut self, user_id: String, object_id: String) -> AppResult<&Self> {
        let mut state = self.get_state().lock().await;

        if let Some(obj) = state.objects.get_mut(&object_id) {
            obj.apply_action(Arc::clone(self.get_state()), user_id)
                .await?;
        }

        Ok(self)
    }

    pub async fn tick(&mut self) {
        let objects = &mut self.state.lock().await.objects;
        for (id, obj) in objects.iter_mut() {
            obj.tick().await.expect("hmm");
        }
    }
}

#[async_trait]
pub trait Controllable {
    async fn control(&mut self, player: &Player, state: Arc<Mutex<GameState>>) -> AppResult<()>;
}

#[async_trait]
impl Controllable for CarDetails {
    async fn control(&mut self, player: &Player, _state: Arc<Mutex<GameState>>) -> AppResult<()> {
        println!("Player {} is now controlling the car", player.id);
        self.driver_user_id = Some(player.id.clone());
        Ok(())
    }
}
