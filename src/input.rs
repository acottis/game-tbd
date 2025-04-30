use std::collections::HashSet;

use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Input {
    held_keys: HashSet<PhysicalKey>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
        }
    }

    pub fn handle_keyboard(&mut self, event: &KeyEvent) {
        if event.state.is_pressed() {
            self.held_keys.insert(event.physical_key);
        } else {
            self.held_keys.remove(&event.physical_key);
        }
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.held_keys.contains(&PhysicalKey::Code(key))
    }
}
