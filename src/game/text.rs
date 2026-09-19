use glam::Vec2;

pub struct Label {
    pub position: Vec2,
    pub color: glyphon::Color,
    pub text: String,
}

impl Label {
    pub fn new(position: Vec2, color: glyphon::Color, text: impl Into<String>) -> Self {
        Self {
            position,
            color,
            text: text.into(),
        }
    }
}
