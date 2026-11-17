use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Main,
    Settings,
    About,
}

#[derive(Serialize, Deserialize)]
struct SavedSettings {
    theme_index: usize,
    show_fps: bool,
}

impl Default for SavedSettings {
    fn default() -> Self {
        Self { theme_index: 0, show_fps: true }
    }
}

pub struct NexusApp {
    screen: Screen,
    theme_index: usize,
    show_fps: bool,
    status: String,
    settings_path: Option<PathBuf>,
    click_consumed: bool,
}

impl NexusApp {
    pub fn load() -> Self {
        let settings_path = settings_path();
        let saved = settings_path.as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<SavedSettings>(&s).ok())
            .unwrap_or_default();

        Self {
            screen: Screen::Main,
            theme_index: saved.theme_index.min(Theme::PRESETS.len() - 1),
            show_fps: saved.show_fps,
            status: "Rust client foundation initialized".to_owned(),
            settings_path,
            click_consumed: false,
        }
    }

    pub fn update(&mut self) {
        self.click_consumed = false;

        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Main;
        }

        if is_key_pressed(KeyCode::F3) {
            self.show_fps = !self.show_fps;
            self.save();
        }
    }

    pub fn draw(&mut self) {
        let bg = Color::from_rgba(13, 15, 19, 255);
        let panel = Color::from_rgba(23, 26, 32, 255);
        let muted = Color::from_rgba(153, 161, 175, 255);
        let accent = Theme::PRESETS[self.theme_index].color();

        clear_background(bg);

        let w = screen_width();
        let h = screen_height();
        let compact = w < 700.0;
        let margin = if compact { 22.0 } else { 56.0 };

        // Header / logo
        draw_rectangle(margin, margin, 46.0, 46.0, accent);
        draw_text("N", margin + 13.0, margin + 33.0, 31.0, WHITE);
        draw_text("NexusClient", margin + 60.0, margin + 27.0, 27.0, WHITE);
        draw_text("JAVA EDITION  /  RUST", margin + 61.0, margin + 43.0, 10.0, muted);

        match self.screen {
            Screen::Main => self.draw_main(margin, h, compact, accent, panel, muted),
            Screen::Settings => self.draw_settings(margin, h, compact, accent, panel, muted),
            Screen::About => self.draw_about(margin, h, compact, accent, panel, muted),
        }

        if self.show_fps {
            let fps = format!("{} FPS", get_fps());
            let size = 17.0;
            let tw = measure_text(&fps, None, size as u16, 1.0).width;
            draw_text(&fps, w - tw - 16.0, 25.0, size, Color::from_rgba(210, 215, 224, 220));
        }

        draw_text(&self.status, margin, h - 18.0, 13.0, muted);
    }

    fn draw_main(&mut self, margin: f32, h: f32, compact: bool, accent: Color, panel: Color, muted: Color) {
        let w = screen_width();
        let title_y = if compact { h * 0.28 } else { h * 0.31 };
        let title_size = if compact { 34.0 } else { 52.0 };
        draw_text("Your game. Your rules.", margin, title_y, title_size, WHITE);
        draw_text("A custom Minecraft Java client, built in Rust.", margin, title_y + 29.0, 17.0, muted);

        let button_w = if compact { (w - margin * 2.0).min(420.0) } else { 330.0 };
        let button_h = 52.0;
        let gap = 12.0;
        let x = margin;
        let mut y = title_y + 72.0;

        if self.button("Singleplayer", x, y, button_w, button_h, accent, true) {
            self.status = "Singleplayer world engine is planned for the next milestone.".into();
        }
        y += button_h + gap;
        if self.button("Multiplayer", x, y, button_w, button_h, panel, false) {
            self.status = "Multiplayer protocol and server browser are not implemented yet.".into();
        }
        y += button_h + gap;
        if self.button("Settings", x, y, button_w, button_h, panel, false) {
            self.screen = Screen::Settings;
        }
        y += button_h + gap;
        if self.button("About NexusClient", x, y, button_w, button_h, panel, false) {
            self.screen = Screen::About;
        }

        let info_x = if compact { margin } else { w * 0.57 };
        let info_y = if compact { h - 145.0 } else { title_y + 5.0 };
        let info_w = if compact { w - margin * 2.0 } else { w * 0.34 };
        draw_rectangle(info_x, info_y, info_w, if compact { 78.0 } else { 190.0 }, panel);
        draw_text("PROJECT STATUS", info_x + 18.0, info_y + 27.0, 13.0, accent);
        draw_text("Client foundation", info_x + 18.0, info_y + 55.0, 19.0, WHITE);
        draw_text("Rendering and UI shell online", info_x + 18.0, info_y + 78.0, 13.0, muted);
        if !compact {
            draw_text("Minecraft world support", info_x + 18.0, info_y + 112.0, 13.0, muted);
            draw_text("Not implemented yet", info_x + 18.0, info_y + 137.0, 13.0, muted);
            draw_text("Protocol compatibility", info_x + 18.0, info_y + 164.0, 13.0, muted);
            draw_text("Planned milestone", info_x + 18.0, info_y + 181.0, 11.0, muted);
        }
    }

    fn draw_settings(&mut self, margin: f32, h: f32, compact: bool, accent: Color, panel: Color, muted: Color) {
        draw_text("Settings", margin, h * 0.23, 38.0, WHITE);
        draw_text("Personalize the NexusClient interface.", margin, h * 0.23 + 28.0, 15.0, muted);

        let y = h * 0.23 + 66.0;
        let width = if compact { screen_width() - margin * 2.0 } else { 520.0 };
        draw_rectangle(margin, y, width, 150.0, panel);
        draw_text("ACCENT COLOR", margin + 18.0, y + 29.0, 13.0, muted);

        let swatch_y = y + 52.0;
        let mut x = margin + 18.0;
        for (i, theme) in Theme::PRESETS.iter().enumerate() {
            let color = theme.color();
            draw_rectangle(x, swatch_y, 48.0, 42.0, color);
            if i == self.theme_index {
                draw_rectangle_lines(x - 3.0, swatch_y - 3.0, 54.0, 48.0, 2.0, WHITE);
            }
            if self.hit(x, swatch_y, 48.0, 42.0) {
                self.theme_index = i;
                self.save();
            }
            x += 61.0;
        }

        let fps_y = y + 119.0;
        let fps_label = if self.show_fps { "FPS counter: ON (F3)" } else { "FPS counter: OFF (F3)" };
        draw_text(fps_label, margin + 18.0, fps_y, 14.0, WHITE);
        if self.hit(margin + 12.0, fps_y - 18.0, width - 24.0, 28.0) {
            self.show_fps = !self.show_fps;
            self.save();
        }

        if self.button("Back", margin, y + 172.0, 160.0, 48.0, accent, true) {
            self.screen = Screen::Main;
        }
    }

    fn draw_about(&mut self, margin: f32, h: f32, compact: bool, accent: Color, panel: Color, muted: Color) {
        draw_text("About NexusClient", margin, h * 0.25, 34.0, WHITE);
        let width = if compact { screen_width() - margin * 2.0 } else { 560.0 };
        draw_rectangle(margin, h * 0.30, width, 150.0, panel);
        draw_text("NexusClient 0.1.0", margin + 18.0, h * 0.30 + 34.0, 21.0, accent);
        draw_text("A native Rust client foundation for Minecraft Java.", margin + 18.0, h * 0.30 + 66.0, 14.0, WHITE);
        draw_text("This build is an application shell, not yet a playable game.", margin + 18.0, h * 0.30 + 92.0, 13.0, muted);
        draw_text("No Mojang or Microsoft affiliation.", margin + 18.0, h * 0.30 + 119.0, 12.0, muted);
        if self.button("Back", margin, h * 0.30 + 172.0, 160.0, 48.0, accent, true) {
            self.screen = Screen::Main;
        }
    }

    fn button(&mut self, label: &str, x: f32, y: f32, w: f32, h: f32, color: Color, primary: bool) -> bool {
        let hovered = {
            let (mx, my) = mouse_position();
            mx >= x && mx <= x + w && my >= y && my <= y + h
        };
        let fill = if hovered { lighten(color, 1.12) } else { color };
        draw_rectangle(x, y, w, h, fill);
        let text_color = if primary { WHITE } else { Color::from_rgba(232, 235, 240, 255) };
        let font_size = 18.0;
        let measured = measure_text(label, None, font_size as u16, 1.0);
        draw_text(label, x + (w - measured.width) / 2.0, y + (h + measured.height) / 2.0 - 4.0, font_size, text_color);

        hovered && is_mouse_button_pressed(MouseButton::Left)
    }

    fn hit(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mx, my) = mouse_position();
        mx >= x && mx <= x + w && my >= y && my <= y + h
            && is_mouse_button_pressed(MouseButton::Left)
    }

    fn save(&self) {
        let Some(path) = &self.settings_path else { return; };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let settings = SavedSettings {
            theme_index: self.theme_index,
            show_fps: self.show_fps,
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = fs::write(path, json);
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        // The first milestone keeps settings in the app's writable working directory.
        Some(PathBuf::from("nexus-settings.json"))
    }
    #[cfg(not(target_os = "android"))]
    {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home).join(".config").join("nexusclient").join("settings.json")
        })
    }
}

fn lighten(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r * factor).min(1.0),
        (color.g * factor).min(1.0),
        (color.b * factor).min(1.0),
        color.a,
    )
}
