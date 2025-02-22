use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;

use super::{
    action::{ActionBuilder, ActionTriggerType},
    map::{tile_to_pixel, Coordinates},
    CarDetails, CarSkin, GameObjectInfo, OutgoingGameObject,
};
use crate::gangsta::GameObjectType;

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum VehicleBehavior {
    Cautious,
    Normal,
    Aggressive,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct Vehicle {
    pub id: String,
    pub position: Coordinates,
    pub behavior: VehicleBehavior,
    pub path: VecDeque<Coordinates>,
    pub skin: CarSkin,
    pub current_speed: u16,
    pub acceleration: u16,
    pub max_speed: u16,
    pub rotation_speed: u16,
    pub driver_user_id: Option<String>,
    pub passenger_user_ids: Vec<String>,
    pub rotation: f32,
    pub velocity: Coordinates,
}

impl Vehicle {
    fn behavior_params(behavior: &VehicleBehavior) -> (u16, u16) {
        match behavior {
            VehicleBehavior::Cautious => (0, 2),
            VehicleBehavior::Normal => (1, 100),
            VehicleBehavior::Aggressive => (1, 120),
        }
    }

    pub fn new(
        id: String,
        start: Coordinates,
        behavior: VehicleBehavior,
        rotation_speed: u16,
    ) -> Self {
        let (acceleration, max_speed) = Self::behavior_params(&behavior);
        Self {
            id,
            position: start,
            behavior,
            rotation_speed,
            driver_user_id: None,
            passenger_user_ids: vec![],
            skin: CarSkin::Sedan,
            velocity: Coordinates { x: 0, y: 0 },
            rotation: 90.0,

            current_speed: 0,
            acceleration,
            max_speed,
            path: VecDeque::new(),
        }
    }

    pub fn move_step(&mut self) {
        println!("Remaining path: {:?}", self.path);
        if let Some(next_position) = self.path.pop_front() {
            self.position = next_position;
        }
    }

    pub fn set_path(&mut self, path: Vec<Coordinates>) -> &Vehicle {
        self.path = VecDeque::from(path);
        self
    }

    pub fn tick(&mut self) {
        self.update_position();
    }

    pub fn to_outgoing_game_object(&self) -> OutgoingGameObject {
        OutgoingGameObject {
            action: Some(
                ActionBuilder::new(ActionTriggerType::ActionKeyPressed(32))
                    .closure_action(
                        |state, object_id, user_id| -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
                            Box::pin(async move {
                                println!("user {} wants to do something with the car {}!", user_id, object_id);
                                if let Some(obj) = state.lock().await.objects.get_mut(&object_id) {
                                    if let GameObjectType::Car(car_details) = &mut obj.details {
                                        car_details.action(user_id);
                                    }
                                }
                                Ok(())
                            })
                        },
                    )
                    .build(),
            ),
            controller_user_id: self.driver_user_id.clone(),
            id: self.id.clone(),
            x: self.position.x,
            y: self.position.y,
            rotation: self.rotation,
            velocity: self.velocity.clone(),
            owner_user_id: self.id.clone(),
            details: GameObjectInfo::Car(CarDetails {
                skin: self.skin.clone(),
                speed: (self.current_speed * 128) as u16,
                acceleration: self.acceleration.clone(),
                max_passengers: self.passenger_user_ids.len() as u8,
                passenger_user_ids: self.passenger_user_ids.clone(),
                rotation_speed: self.rotation_speed,
                driver_user_id: self.driver_user_id.clone(),
            }),
        }
    }

    pub fn action(&mut self, user_id: String) {
        self.driver_user_id = Some(user_id);
    }

    pub fn set_tile_path(&mut self, tile_path: Vec<Coordinates>) {
        let pixel_path = tile_path
            .into_iter()
            .map(|tile| tile_to_pixel(tile))
            .collect();
        self.set_path(pixel_path);
    }

    pub fn update_position(&mut self) {
        if let Some(target) = self.path.front() {
            if self.current_speed < self.max_speed {
                self.current_speed += self.acceleration;
                if self.current_speed > self.max_speed {
                    self.current_speed = self.max_speed;
                }
            }

            let dx = target.x - self.position.x;
            let dy = target.y - self.position.y;
            let distance = ((dx * dx + dy * dy) as f64).sqrt();

            let desired_angle = (dy as f64).atan2(dx as f64) + std::f64::consts::FRAC_PI_2;

            let current_angle = self.rotation as f64;
            let mut angle_diff = desired_angle - current_angle;

            while angle_diff < -std::f64::consts::PI {
                angle_diff += 2.0 * std::f64::consts::PI;
            }
            while angle_diff > std::f64::consts::PI {
                angle_diff -= 2.0 * std::f64::consts::PI;
            }

            let rotation_change = if angle_diff.abs() < self.rotation_speed as f64 {
                angle_diff
            } else {
                self.rotation_speed as f64 * angle_diff.signum()
            };
            self.rotation = (current_angle + rotation_change) as f32;

            if distance < self.current_speed as f64 {
                self.position = self.path.pop_front().unwrap();
            } else {
                let norm_x = dx as f64 / distance;
                let norm_y = dy as f64 / distance;
                self.position.x += (norm_x * self.current_speed as f64).round() as i32;
                self.position.y += (norm_y * self.current_speed as f64).round() as i32;
            }
        }
    }
}
