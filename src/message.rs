use serde::{Deserialize, Serialize};
use tcod::Color;

use crate::{BAR_WIDTH, PANEL_HEIGHT, SCREEN_WIDTH};

pub const MSG_X: i32 = BAR_WIDTH + 2;
pub const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
pub const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

#[derive(Serialize, Deserialize)]
pub struct Messages {
    messages: Vec<(String, Color)>,
}
impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color))
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
    }
}
