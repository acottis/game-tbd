pub enum Anchor {
    Left,
    Right,
}

pub struct Text {
    pub anchor: Anchor,
    pub color: glyphon::Color,
    pub text: String,
    generation: u32,
}

impl Text {
    pub fn new(color: glyphon::Color, anchor: Anchor, text: impl Into<String>) -> Self {
        Self {
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
