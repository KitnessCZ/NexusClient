
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Dashboard,
    Versions,
    Mods,
    ResourcePacks,
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
        Self {
            theme_index: 0,
            show_fps: true,
        }
    }
}

pub struct NexusApp {
    screen: Screen,
    theme_index: usize,
    show_fps: bool,
    settings_path: Option<PathBuf>,
}

impl NexusApp {
    pub fn load() -> Self {
        let settings_path = settings_path();
        let saved = settings_path
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str::<SavedSettings>(&contents).ok())
            .unwrap_or_default();

        Self {
            screen: Screen::Dashboard,
            theme_index: saved.theme_index.min(Theme::PRESETS.len() - 1),
            show_fps: saved.show_fps,
            settings_path,
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Dashboard;
        }

        if is_key_pressed(KeyCode::F3) {
            self.show_fps = !self.show_fps;
            self.save();
        }
    }

    pub fn draw(&mut self) {
        let background = Color::from_rgba(15, 16, 21, 255);
        let sidebar = Color::from_rgba(19, 20, 27, 255);
        let surface = Color::from_rgba(27, 28, 37, 255);
        let surface_alt = Color::from_rgba(35, 36, 46, 255);
        let muted = Color::from_rgba(157, 159, 176, 255);
        let white = Color::from_rgba(241, 241, 248, 255);
        let accent = Theme::PRESETS[self.theme_index].color();

        clear_background(background);

        let width = screen_width();
        let height = screen_height();
        let sidebar_width = (width * 0.205).clamp(210.0, 280.0);
        let pad = (width * 0.027).clamp(22.0, 42.0);

        draw_rectangle(0.0, 0.0, sidebar_width, height, sidebar);
        draw_line(
            sidebar_width,
            0.0,
            sidebar_width,
            height,
            1.0,
            Color::from_rgba(49, 49, 61, 255),
        );

        self.draw_brand(24.0, 34.0, accent, white, muted);

        let nav_x = 16.0;
        let nav_w = sidebar_width - 32.0;
        let nav_h = 48.0;
        let nav_gap = 10.0;
        let mut nav_y = 118.0;

        self.nav_button("DASHBOARD", Screen::Dashboard, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);
        nav_y += nav_h + nav_gap;
        self.nav_button("VERSIONS", Screen::Versions, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);
        nav_y += nav_h + nav_gap;
        self.nav_button("MODS", Screen::Mods, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);
        nav_y += nav_h + nav_gap;
        self.nav_button("RESOURCE PACKS", Screen::ResourcePacks, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);
        nav_y += nav_h + nav_gap;
        self.nav_button("SETTINGS", Screen::Settings, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);
        nav_y += nav_h + nav_gap;
        self.nav_button("ABOUT", Screen::About, nav_x, nav_y, nav_w, nav_h, accent, surface_alt, muted, white);

        draw_text(
            "NEXUSCLIENT  /  JAVA EDITION",
            24.0,
            height - 28.0,
            11.0,
            muted,
        );

        let content_x = sidebar_width + pad;
        let content_w = (width - content_x - pad).max(200.0);
        self.draw_top_bar(content_x, 22.0, content_w, accent, white, muted);

        let content_y = 105.0;
        match self.screen {
            Screen::Dashboard => self.draw_dashboard(content_x, content_y, content_w, height, surface, surface_alt, accent, white, muted),
            Screen::Versions => self.draw_placeholder_page("Versions", "Manage game versions and client profiles.", "Version management is not connected yet.", content_x, content_y, content_w, height, surface, accent, white, muted),
            Screen::Mods => self.draw_placeholder_page("Mods", "Your installed modifications will appear here.", "Mod loading and management are planned for a future milestone.", content_x, content_y, content_w, height, surface, accent, white, muted),
            Screen::ResourcePacks => self.draw_placeholder_page("Resource packs", "Manage textures and visual customizations.", "Resource-pack loading is not implemented yet.", content_x, content_y, content_w, height, surface, accent, white, muted),
            Screen::Settings => self.draw_settings(content_x, content_y, content_w, height, surface, surface_alt, accent, white, muted),
            Screen::About => self.draw_about(content_x, content_y, content_w, height, surface, accent, white, muted),
        }

        if self.show_fps {
            let fps = format!("{} FPS", get_fps());
            let measured = measure_text(&fps, None, 14, 1.0).width;
            draw_text(&fps, width - measured - 18.0, height - 14.0, 14.0, muted);
        }
    }

    fn draw_brand(&self, x: f32, y: f32, accent: Color, white: Color, muted: Color) {
        draw_rectangle(x, y, 42.0, 42.0, accent);
        draw_text("N", x + 11.0, y + 30.0, 29.0, white);
        draw_text("NexusClient", x + 54.0, y + 22.0, 23.0, white);
        draw_text("ANDROID  /  JAVA", x + 55.0, y + 38.0, 10.0, muted);
    }

    fn draw_top_bar(&self, x: f32, y: f32, width: f32, accent: Color, white: Color, muted: Color) {
        draw_text("NEXUSCLIENT", x, y + 15.0, 12.0, accent);

        let profile = "PROFILE  /  DEFAULT";
        let profile_width = measure_text(profile, None, 14, 1.0).width;
        draw_rectangle(
            x + width - profile_width - 28.0,
            y - 2.0,
            profile_width + 28.0,
            34.0,
            Color::from_rgba(30, 30, 40, 255),
        );
        draw_text(profile, x + width - profile_width - 14.0, y + 20.0, 14.0, white);

        let line_y = y + 48.0;
        draw_line(x, line_y, x + width, line_y, 1.0, Color::from_rgba(49, 49, 61, 255));
        let _ = muted;
    }

    fn nav_button(
        &mut self,
        label: &str,
        target: Screen,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        accent: Color,
        surface: Color,
        muted: Color,
        white: Color,
    ) {
        let selected = self.screen == target;
        let hovered = point_in_rect(mouse_position(), x, y, width, height);
        let fill = if selected {
            Color::new(accent.r * 0.24, accent.g * 0.24, accent.b * 0.24, 1.0)
        } else if hovered {
            surface
        } else {
            Color::from_rgba(19, 20, 27, 255)
        };

        draw_rectangle(x, y, width, height, fill);
        if selected {
            draw_rectangle(x, y, 4.0, height, accent);
        }

        let color = if selected { white } else { muted };
        draw_text(label, x + 18.0, y + 30.0, 15.0, color);

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.screen = target;
        }
    }

    fn draw_dashboard(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        surface: Color,
        surface_alt: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text("Welcome back.", x, y + 30.0, 34.0, white);
        draw_text(
            "Your Minecraft Java experience starts here.",
            x,
            y + 58.0,
            16.0,
            muted,
        );

        let card_y = y + 86.0;
        let card_h = (height - card_y - 92.0).clamp(230.0, 340.0);
        draw_rectangle(x, card_y, width, card_h, surface);

        draw_text("READY WHEN YOU ARE", x + 28.0, card_y + 38.0, 12.0, accent);
        draw_text("NexusClient", x + 28.0, card_y + 82.0, 31.0, white);
        draw_text(
            "A custom Minecraft Java client, built in Rust.",
            x + 28.0,
            card_y + 111.0,
            16.0,
            muted,
        );

        let info_x = x + 28.0;
        let info_y = card_y + 145.0;
        let info_w = (width * 0.52).min(470.0);
        draw_rectangle(info_x, info_y, info_w, 74.0, surface_alt);
        draw_text("CLIENT STATUS", info_x + 16.0, info_y + 25.0, 11.0, accent);
        draw_text("Foundation build", info_x + 16.0, info_y + 49.0, 16.0, white);
        draw_text("Interface online  /  Game engine in development", info_x + 16.0, info_y + 66.0, 10.0, muted);

        let play_w = 250.0_f32.min(width - 56.0).max(150.0);
        let play_x = x + 28.0;
        let play_y = card_y + card_h - 64.0;
        self.action_button("PLAY", play_x, play_y, play_w, 44.0, accent, white);

        let profile_x = play_x + play_w + 14.0;
        let profile_w = 170.0_f32.min((width - play_w - 70.0).max(100.0));
        draw_rectangle(profile_x, play_y, profile_w, 44.0, surface_alt);
        draw_text("DEFAULT PROFILE", profile_x + 13.0, play_y + 27.0, 12.0, white);

        let note = "The launcher interface is ready. Minecraft gameplay features are still under development.";
        draw_text(note, x, height - 32.0, 12.0, muted);
    }

    fn draw_placeholder_page(
        &self,
        title: &str,
        subtitle: &str,
        message: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        surface: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text(title, x, y + 30.0, 34.0, white);
        draw_text(subtitle, x, y + 58.0, 16.0, muted);

        let card_y = y + 88.0;
        let card_h = (height - card_y - 60.0).max(130.0);
        draw_rectangle(x, card_y, width, card_h, surface);
        draw_text("COMING IN A FUTURE MILESTONE", x + 24.0, card_y + 42.0, 12.0, accent);
        draw_text(message, x + 24.0, card_y + 82.0, 17.0, white);
    }

    fn draw_settings(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        surface: Color,
        surface_alt: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text("Settings", x, y + 30.0, 34.0, white);
        draw_text("Customize the NexusClient interface.", x, y + 58.0, 16.0, muted);

        let card_y = y + 88.0;
        draw_rectangle(x, card_y, width.min(650.0), 205.0, surface);
        draw_text("APPEARANCE", x + 24.0, card_y + 34.0, 12.0, accent);
        draw_text("Accent color", x + 24.0, card_y + 66.0, 17.0, white);

        let swatch_y = card_y + 88.0;
        let mut swatch_x = x + 24.0;
        for (index, theme) in Theme::PRESETS.iter().enumerate() {
            let color = theme.color();
            draw_rectangle(swatch_x, swatch_y, 42.0, 36.0, color);
            if index == self.theme_index {
                draw_rectangle_lines(swatch_x - 3.0, swatch_y - 3.0, 48.0, 42.0, 2.0, white);
            }
            if point_in_rect(mouse_position(), swatch_x, swatch_y, 42.0, 36.0)
                && is_mouse_button_pressed(MouseButton::Left)
            {
                self.theme_index = index;
                self.save();
            }
            swatch_x += 56.0;
        }

        let fps_y = card_y + 164.0;
        let fps_label = if self.show_fps { "FPS counter: ON" } else { "FPS counter: OFF" };
        draw_text(fps_label, x + 24.0, fps_y, 15.0, white);
        if point_in_rect(mouse_position(), x + 18.0, fps_y - 20.0, 220.0, 28.0)
            && is_mouse_button_pressed(MouseButton::Left)
        {
            self.show_fps = !self.show_fps;
            self.save();
        }

        let _ = (surface_alt, height, accent);
    }

    fn draw_about(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        surface: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text("About NexusClient", x, y + 30.0, 34.0, white);
        draw_text("An independent Minecraft Java client project.", x, y + 58.0, 16.0, muted);

        draw_rectangle(x, y + 88.0, width.min(700.0), 190.0, surface);
        draw_text("NexusClient 0.2.0", x + 24.0, y + 128.0, 22.0, accent);
        draw_text("Built with Rust and Macroquad.", x + 24.0, y + 162.0, 16.0, white);
        draw_text("This build contains the launcher interface only.", x + 24.0, y + 190.0, 14.0, muted);
        draw_text("Minecraft authentication, protocol and gameplay are not implemented yet.", x + 24.0, y + 218.0, 13.0, muted);

        let _ = height;
    }

    fn action_button(&self, label: &str, x: f32, y: f32, width: f32, height: f32, color: Color, text_color: Color) {
        let hovered = point_in_rect(mouse_position(), x, y, width, height);
        let fill = if hovered { lighten(color, 1.10) } else { color };
        draw_rectangle(x, y, width, height, fill);
        let size = 17;
        let measured = measure_text(label, None, size, 1.0);
        draw_text(
            label,
            x + (width - measured.width) / 2.0,
            y + (height + measured.height) / 2.0 - 4.0,
            size as f32,
            text_color,
        );
    }

    fn save(&self) {
        let Some(path) = &self.settings_path else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let saved = SavedSettings {
            theme_index: self.theme_index,
            show_fps: self.show_fps,
        };

        if let Ok(contents) = serde_json::to_string_pretty(&saved) {
            let _ = fs::write(path, contents);
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        Some(PathBuf::from("nexus-settings.json"))
    }

    #[cfg(not(target_os = "android"))]
    {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("nexusclient")
                .join("settings.json")
        })
    }
}

fn point_in_rect(position: (f32, f32), x: f32, y: f32, width: f32, height: f32) -> bool {
    position.0 >= x
        && position.0 <= x + width
        && position.1 >= y
        && position.1 <= y + height
}

fn lighten(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r * factor).min(1.0),
        (color.g * factor).min(1.0),
        (color.b * factor).min(1.0),
        color.a,
    )
}
