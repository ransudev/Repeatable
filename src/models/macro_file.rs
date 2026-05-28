use crate::models::event::InputEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroFile {
    pub id: String,
    pub name: String,
    pub version: u8,
    pub events: Vec<InputEvent>,
}

impl MacroFile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            version: 1,
            events: Vec::new(),
        }
    }
}
