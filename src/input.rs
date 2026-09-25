use std::collections::HashSet;

use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Input {
    held_keys: HashSet<PhysicalKey>,
    pub pressed_keys: HashSet<PhysicalKey>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
        }
    }

    /// TODO: Change to fixed size array with booleans and indices?
    pub fn handle_keyboard(&mut self, event: &KeyEvent) {
        let key = event.physical_key;
        if event.state.is_pressed() {
            // Only insert into press_keys if insert suceeds
            // which makes it a new key event
            if self.held_keys.insert(key) {
                self.pressed_keys.insert(key);
            }
        } else {
            self.held_keys.remove(&key);
        }
    }

    pub fn is_held(&self, key: KeyCode) -> bool {
        self.held_keys.contains(&PhysicalKey::Code(key))
    }
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&PhysicalKey::Code(key))
    }
}
