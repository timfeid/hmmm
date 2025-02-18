use std::{collections::HashMap, future::Future, pin::Pin};

use action::{Action, ActionBuilder, ActionTrigger, ActionTriggerType};
use serde::{Deserialize, Serialize};
use specta::Type;

mod action;

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct GameState {
    pub visible_objects: HashMap<String, GameObject>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]

pub struct GameObject {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub rotation: f32,
    pub velocity: Coordinates,
    pub owner_user_id: String,
    pub info: GameObjectInfo,
    pub hidden: bool,
    pub animation: Option<String>,
    #[serde(skip_deserializing)]
    pub action: Option<ActionTrigger>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct PersonDetails {
    pub user_id: String,
    pub skin: PersonSkin,
}
impl PersonDetails {
    fn new(user_id: String) -> Self {
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
    pub max_passengers: u8,
    pub passenger_user_ids: Vec<String>,
    pub rotation_speed: u16,
    pub driver_user_id: Option<String>,
}
impl CarDetails {
    fn new(skin: CarSkin, speed: u16, rotation_speed: u16, max_passengers: u8) -> Self {
        Self {
            skin,
            speed,
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
    // starting speed
    // speed decay over acceleration
    // top speed
    // break power
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
impl GameState {
    pub fn default() -> GameState {
        let mut hash = HashMap::new();
        hash.insert(
            "tim's person".to_string(),
            GameObject {
                hidden: false,
                x: 455,
                y: 789,
                id: "tim's person".to_string(),
                rotation: 0.0,
                velocity: Coordinates { x: 0, y: 0 },
                owner_user_id: "tim".to_string(),
                info: GameObjectInfo::Person(PersonDetails::new("tim".to_string())),
                animation: Some("idle".to_string()),
                action: None,
            },
        );
        hash.insert(
            "tim's car".to_string(),
            GameObject {
                hidden: false,
                x: 455,
                id: "tim's car".to_string(),
                y: 789,
                rotation: 1.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_user_id: "tim".to_string(),
                info: GameObjectInfo::Car(CarDetails::new(CarSkin::Sedan, 150, 3, 2)),
                animation: Some("idle".to_string()),
                action: Some(
                    ActionBuilder::new(ActionTriggerType::ActionKeyPressed(32))
                        .closure_action(
                            |user_id| -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
                                Box::pin(async move {
                                    println!(
                                        "user {} wants to do something with the car!",
                                        user_id
                                    );
                                    Ok(())
                                })
                            },
                        )
                        .build(),
                ),
            },
        );
        hash.insert(
            "bob's person".to_string(),
            GameObject {
                hidden: false,
                id: "bob's person".to_string(),
                x: 527,
                y: 789,
                rotation: 0.0,
                velocity: Coordinates { x: 0, y: 0 },
                owner_user_id: "bob".to_string(),
                info: GameObjectInfo::Person(PersonDetails::new("bob".to_string())),
                animation: Some("idle".to_string()),
                action: None,
            },
        );
        hash.insert(
            "bob's car".to_string(),
            GameObject {
                hidden: false,
                id: "bob's car".to_string(),
                x: 527,
                y: 789,
                rotation: 180.57,
                velocity: Coordinates { x: 0, y: 0 },
                owner_user_id: "bob".to_string(),
                info: GameObjectInfo::Car(CarDetails::new(CarSkin::Police, 250, 4, 3)),
                animation: Some("idle".to_string()),
                action: None,
            },
        );
        GameState {
            visible_objects: hash,
        }
    }
}
