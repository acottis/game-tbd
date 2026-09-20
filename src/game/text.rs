use glam::Vec2;

pub enum Anchor {
    Left,
    Right,
}

pub struct Label {
    pub position: Vec2,
    pub anchor: Anchor,
    pub color: glyphon::Color,
    pub text: String,
    generation: u32,
}

impl Label {
    pub fn new(
        position: Vec2,
        color: glyphon::Color,
        anchor: Anchor,
        text: impl Into<String>,
    ) -> Self {
        Self {
            position,
            color,
            text: text.into(),
            // Start at one so we immediately need to draw
            generation: 1,
            anchor,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}
