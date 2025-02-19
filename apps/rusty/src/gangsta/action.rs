use std::{
    any::Any,
    fmt::{self, Debug},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;

use super::{Game, GameState};

#[async_trait::async_trait]
pub trait Action: Send + Sync + Debug + 'static {
    async fn apply(
        &self,
        game: Arc<Mutex<GameState>>,
        // game: Arc<Mutex<Game>>,
        // card: Arc<Mutex<Card>>,
        // owner: Arc<Mutex<Player>>,
        // target: Option<FrontendTarget>,
        ability_id: String,
        user_id: String,
    ) -> Result<(), String>;
    fn as_any(&self) -> &dyn Any;
}

// #[derive(Clone)]
// pub struct Action {
//     pub action: Arc<dyn Fn(String) -> Arc<dyn ActionDetails + Send + Sync> + Send + Sync>,
// }

// impl Debug for Action {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Trigger").finish()
//     }
// }

#[derive(Debug, Clone, PartialEq, Type, Deserialize, Serialize)]
pub enum ActionTriggerType {
    ActionKeyPressed(u8),
}

// This function will be used to provide a default value for the field.
fn default_action() -> Arc<dyn Action + Send + Sync> {
    Arc::new(BlankAction {})
}

#[derive(Type, Deserialize, Serialize, Clone)]
pub struct ActionTrigger {
    // pub id: String,
    pub trigger_type: ActionTriggerType,

    #[serde(skip, default = "default_action")]
    pub action: Arc<dyn Action + Send + Sync>,
    // pub requirements: Arc<
    //     dyn Fn(
    //             Arc<Mutex<Game>>,
    //             Arc<Mutex<Card>>,
    //             Arc<Mutex<Player>>,
    //             Option<String>,
    //         ) -> Pin<Box<dyn Future<Output = bool> + Send>>
    //         + Send
    //         + Sync,
    // >,
}

#[derive(Type, Serialize, Clone, Debug)]
pub struct BlankAction {}

#[async_trait::async_trait]
impl Action for BlankAction {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn apply(
        &self,
        game: Arc<Mutex<GameState>>,
        object_id: String,
        user_id: String,
    ) -> Result<(), String> {
        Ok(())
    }
}

impl Debug for ActionTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActionTrigger")
            .field("trigger_type", &self.trigger_type)
            .finish()
    }
}

impl ActionTrigger {
    fn new(
        trigger_type: ActionTriggerType,
        // card_required_target: CardRequiredTarget,
        action: Arc<dyn Action + Send + Sync>,
        // requirements: Arc<
        //     dyn Fn(
        //             Arc<Mutex<Game>>,
        //             Arc<Mutex<Card>>,
        //             Arc<Mutex<Player>>,
        //             Option<String>,
        //         ) -> Pin<Box<dyn Future<Output = bool> + Send>>
        //         + Send
        //         + Sync,
        // >,
    ) -> Self {
        Self {
            // id: Ulid::new().to_string(),
            trigger_type,
            // card_required_target,
            action,
            // requirements,
        }
    }
}

pub struct ActionBuilder {
    trigger_type: ActionTriggerType,
    action: Option<Arc<dyn Action + Send + Sync>>,
    // requirements: Arc<
    //     dyn Fn(
    //             Arc<Mutex<Game>>,
    //             Arc<Mutex<Card>>,
    //             Arc<Mutex<Player>>,
    //             Option<String>,
    //         ) -> Pin<Box<dyn Future<Output = bool> + Send>>
    //         + Send
    //         + Sync,
    // >,
}

pub struct AsyncClosureAction {
    closure: Arc<
        dyn Fn(
                Arc<Mutex<GameState>>,
                String,
                String,
            ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync,
    >,
}
#[async_trait::async_trait]
impl Action for AsyncClosureAction {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn apply(
        &self,
        game: Arc<Mutex<GameState>>,
        object_id: String,
        user_id: String,
    ) -> Result<(), String> {
        let response = (self.closure)(game, object_id, user_id).await;
        return response;
    }
}
impl Debug for AsyncClosureAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncClosureActionWithAbilityId").finish()
    }
}

impl AsyncClosureAction {
    pub fn new<F>(closure: F) -> Self
    where
        F: Fn(
                Arc<Mutex<GameState>>,
                String,
                String,
            ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            closure: Arc::new(closure),
        }
    }
}

impl ActionBuilder {
    pub fn new(trigger_type: ActionTriggerType) -> Self {
        Self {
            trigger_type,
            action: None,
            // requirements: Arc::new(|_, _, _, _| Box::pin(async move { true })),
        }
    }

    pub fn closure_action<F>(mut self, action: F) -> Self
    where
        F: Fn(
                Arc<Mutex<GameState>>,
                String,
                // Arc<Mutex<Card>>,
                // Arc<Mutex<Player>>,
                // Option<FrontendTarget>,
                String,
            ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + 'static
            + Send
            + Sync,
    {
        self.action = Some(Arc::new(AsyncClosureAction::new(action)));
        self
    }

    pub fn action<F>(mut self, action: F) -> Self
    where
        F: Action + Send + Sync,
    {
        self.action = Some(Arc::new(action));
        self
    }

    // pub fn requirements<F>(mut self, requirements: F) -> Self
    // where
    //     F: Fn(
    //             Arc<Mutex<Game>>,
    //             Arc<Mutex<Card>>,
    //             Arc<Mutex<Player>>,
    //             Option<String>,
    //         ) -> Pin<Box<dyn Future<Output = bool> + Send>>
    //         + 'static
    //         + Send
    //         + Sync,
    // {
    //     self.requirements = Arc::new(requirements);
    //     self
    // }

    pub fn build(self) -> ActionTrigger {
        ActionTrigger::new(
            self.trigger_type,
            // self.target,
            self.action.unwrap(),
            // self.requirements,
        )
    }
}
