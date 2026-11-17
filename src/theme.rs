use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub accent: [u8; 3],
}

impl Theme {
    pub const PRESETS: [Theme; 6] = [
        Theme { name: "Red", accent: [220, 38, 38] },
        Theme { name: "Blue", accent: [37, 99, 235] },
        Theme { name: "Purple", accent: [147, 51, 234] },
        Theme { name: "Green", accent: [22, 163, 74] },
        Theme { name: "Orange", accent: [234, 88, 12] },
        Theme { name: "Pink", accent: [219, 39, 119] },
    ];

    pub fn color(self) -> Color {
        Color::from_rgba(self.accent[0], self.accent[1], self.accent[2], 255)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::PRESETS[0]
    }
}
