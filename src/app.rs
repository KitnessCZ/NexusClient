
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
    pointer: (f32, f32),
    pointer_pressed: bool,
    notice: Option<String>,
}

impl NexusApp {
    pub fn load() -> Self {
        let settings_path = settings_path();

        let saved = settings_path
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| {
                serde_json::from_str::<SavedSettings>(&contents).ok()
            })
            .unwrap_or_default();

        Self {
            screen: Screen::Dashboard,
            theme_index: saved.theme_index.min(Theme::PRESETS.len() - 1),
            show_fps: saved.show_fps,
            settings_path,
            pointer: (0.0, 0.0),
            pointer_pressed: false,
            notice: None,
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Dashboard;
            self.notice = None;
        }

        if is_key_pressed(KeyCode::F3) {
            self.show_fps = !self.show_fps;
            self.save();
        }
    }

    pub fn draw(&mut self) {
        let physical_width = screen_width();
        let physical_height = screen_height();

        // Use a 1280x720 design space in landscape and 720x1280 in portrait.
        let scale = if physical_width >= physical_height {
            (physical_width / 1280.0).min(physical_height / 720.0)
        } else {
            (physical_width / 720.0).min(physical_height / 1280.0)
        }
        .max(0.1);

        let width = physical_width / scale;
        let height = physical_height / scale;
        let portrait = width < height;

        // All UI drawing uses logical coordinates. Macroquad scales them
        // to the actual display size through this camera.
        let camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            width,
            height,
        ));
        set_camera(&camera);

        let mouse = mouse_position();
        self.pointer = (mouse.0 / scale, mouse.1 / scale);
        self.pointer_pressed = is_mouse_button_pressed(MouseButton::Left);

        let background = Color::from_rgba(15, 16, 21, 255);
        let sidebar = Color::from_rgba(19, 20, 27, 255);
        let surface = Color::from_rgba(27, 28, 37, 255);
        let surface_alt = Color::from_rgba(35, 36, 46, 255);
        let muted = Color::from_rgba(157, 159, 176, 255);
        let white = Color::from_rgba(241, 241, 248, 255);
        let accent = Theme::PRESETS[self.theme_index].color();

        clear_background(background);

        if portrait {
            self.draw_portrait(
                width,
                height,
                surface,
                surface_alt,
                accent,
                white,
                muted,
            );
        } else {
            self.draw_landscape(
                width,
                height,
                sidebar,
                surface,
                surface_alt,
                accent,
                white,
                muted,
            );
        }

        if self.show_fps {
            let fps = format!("{} FPS", get_fps());
            let measured = measure_text(&fps, None, 14, 1.0).width;
            draw_text(
                &fps,
                width - measured - 18.0,
                height - 14.0,
                14.0,
                muted,
            );
        }
    }

    fn draw_landscape(
        &mut self,
        width: f32,
        height: f32,
        sidebar: Color,
        surface: Color,
        surface_alt: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        let sidebar_width = (width * 0.205).clamp(220.0, 280.0);
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
        let nav_width = sidebar_width - 32.0;
        let nav_height = 48.0;
        let nav_gap = 10.0;
        let mut nav_y = 118.0;

        self.nav_button("DASHBOARD", Screen::Dashboard, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);
        nav_y += nav_height + nav_gap;
        self.nav_button("VERSIONS", Screen::Versions, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);
        nav_y += nav_height + nav_gap;
        self.nav_button("MODS", Screen::Mods, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);
        nav_y += nav_height + nav_gap;
        self.nav_button("RESOURCE PACKS", Screen::ResourcePacks, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);
        nav_y += nav_height + nav_gap;
        self.nav_button("SETTINGS", Screen::Settings, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);
        nav_y += nav_height + nav_gap;
        self.nav_button("ABOUT", Screen::About, nav_x, nav_y, nav_width, nav_height, accent, surface_alt, muted, white);

        draw_text(
            "NEXUSCLIENT  /  JAVA EDITION",
            24.0,
            height - 28.0,
            11.0,
            muted,
        );

        let content_x = sidebar_width + pad;
        let content_width = (width - content_x - pad).max(200.0);

        self.draw_top_bar(content_x, 22.0, content_width, accent, white);

        self.draw_current_screen(
            content_x,
            105.0,
            content_width,
            height,
            surface,
            surface_alt,
            accent,
            white,
            muted,
        );
    }

    fn draw_portrait(
        &mut self,
        width: f32,
        height: f32,
        surface: Color,
        surface_alt: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        // In portrait, navigation becomes a two-row touch-friendly grid.
        self.draw_brand(22.0, 22.0, accent, white, muted);

        let nav_x = 16.0;
        let nav_y = 92.0;
        let gap = 8.0;
        let button_width = (width - 2.0 * nav_x - 2.0 * gap) / 3.0;
        let button_height = 48.0;

        let items = [
            ("HOME", Screen::Dashboard),
            ("VERSIONS", Screen::Versions),
            ("MODS", Screen::Mods),
            ("PACKS", Screen::ResourcePacks),
            ("SETTINGS", Screen::Settings),
            ("ABOUT", Screen::About),
        ];

        for (index, (label, target)) in items.iter().enumerate() {
            let column = index % 3;
            let row = index / 3;
            let x = nav_x + column as f32 * (button_width + gap);
            let y = nav_y + row as f32 * (button_height + gap);

            self.nav_button(
                label,
                *target,
                x,
                y,
                button_width,
                button_height,
                accent,
                surface_alt,
                muted,
                white,
            );
        }

        draw_line(
            16.0,
            204.0,
            width - 16.0,
            204.0,
            1.0,
            Color::from_rgba(49, 49, 61, 255),
        );

        self.draw_current_screen(
            20.0,
            224.0,
            width - 40.0,
            height,
            surface,
            surface_alt,
            accent,
            white,
            muted,
        );
    }

    fn draw_current_screen(
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
        match self.screen {
            Screen::Dashboard => self.draw_dashboard(
                x, y, width, height, surface, surface_alt, accent, white, muted,
            ),
            Screen::Versions => self.draw_placeholder_page(
                "Versions",
                "Manage game versions and client profiles.",
                "Version management is not connected yet.",
                x, y, width, height, surface, accent, white, muted,
            ),
            Screen::Mods => self.draw_placeholder_page(
                "Mods",
                "Your installed modifications will appear here.",
                "Mod loading and management are planned for a future milestone.",
                x, y, width, height, surface, accent, white, muted,
            ),
            Screen::ResourcePacks => self.draw_placeholder_page(
                "Resource packs",
                "Manage textures and visual customizations.",
                "Resource-pack loading is not implemented yet.",
                x, y, width, height, surface, accent, white, muted,
            ),
            Screen::Settings => self.draw_settings(
                x, y, width, height, surface, accent, white, muted,
            ),
            Screen::About => self.draw_about(
                x, y, width, height, surface, accent, white, muted,
            ),
        }
    }

    fn draw_brand(&self, x: f32, y: f32, accent: Color, white: Color, muted: Color) {
        draw_rectangle(x, y, 42.0, 42.0, accent);
        draw_text("N", x + 11.0, y + 30.0, 29.0, white);
        draw_text("NexusClient", x + 54.0, y + 22.0, 23.0, white);
        draw_text("ANDROID  /  JAVA", x + 55.0, y + 38.0, 10.0, muted);
    }

    fn draw_top_bar(
        &self,
        x: f32,
        y: f32,
        width: f32,
        accent: Color,
        white: Color,
    ) {
        draw_text("NEXUSCLIENT", x, y + 15.0, 12.0, accent);

        let profile = "PROFILE  /  DEFAULT";
        let profile_width = measure_text(profile, None, 14, 1.0).width;
        let profile_x = x + width - profile_width - 28.0;

        self.simple_button(
            profile,
            profile_x,
            y - 2.0,
            profile_width + 28.0,
            38.0,
            Color::from_rgba(30, 30, 40, 255),
            white,
            14,
        );

        if self.pointer_pressed
            && point_in_rect(self.pointer, profile_x, y - 2.0, profile_width + 28.0, 38.0)
        {
            self.notice = Some("Profile selection is not implemented yet.".to_owned());
        }

        draw_line(
            x,
            y + 48.0,
            x + width,
            y + 48.0,
            1.0,
            Color::from_rgba(49, 49, 61, 255),
        );
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
        let hovered = point_in_rect(self.pointer, x, y, width, height);

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

        let font_size = if width < 150.0 { 12.0 } else { 15.0 };
        let color = if selected { white } else { muted };
        let measured = measure_text(label, None, font_size as u16, 1.0);

        draw_text(
            label,
            x + (width - measured.width) / 2.0,
            y + (height + measured.height) / 2.0 - 3.0,
            font_size,
            color,
        );

        if hovered && self.pointer_pressed {
            self.screen = target;
            self.notice = None;
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
        let portrait = width < height;

        draw_text(
            "Welcome back.",
            x,
            y + 34.0,
            if portrait { 30.0 } else { 34.0 },
            white,
        );

        draw_text(
            "Your Minecraft Java experience starts here.",
            x,
            y + 62.0,
            15.0,
            muted,
        );

        let card_y = y + 88.0;
        let available_height = (height - card_y - 54.0).max(260.0);
        let card_height = if portrait {
            available_height.min(430.0)
        } else {
            available_height.clamp(250.0, 340.0)
        };

        draw_rectangle(x, card_y, width, card_height, surface);

        let inner_x = x + 24.0;
        draw_text("READY WHEN YOU ARE", inner_x, card_y + 38.0, 12.0, accent);
        draw_text("NexusClient", inner_x, card_y + 82.0, 30.0, white);

        draw_text(
            "A custom Minecraft Java client, built in Rust.",
            inner_x,
            card_y + 110.0,
            15.0,
            muted,
        );

        let info_y = card_y + 138.0;
        let info_width = (width - 48.0).min(470.0);

        draw_rectangle(
            inner_x,
            info_y,
            info_width,
            74.0,
            surface_alt,
        );

        draw_text("CLIENT STATUS", inner_x + 14.0, info_y + 24.0, 11.0, accent);
        draw_text("Foundation build", inner_x + 14.0, info_y + 47.0, 16.0, white);
        draw_text(
            "Interface online  /  Game engine in development",
            inner_x + 14.0,
            info_y + 65.0,
            10.0,
            muted,
        );

        let button_y = card_y + card_height - 62.0;
        let button_width = (width - 48.0).min(250.0).max(150.0);

        if self.action_button(
            "PLAY",
            inner_x,
            button_y,
            button_width,
            46.0,
            accent,
            white,
        ) {
            self.notice = Some(
                "The Minecraft game runtime is not ready yet.".to_owned(),
            );
        }

        let profile_x = inner_x + button_width + 12.0;
        let profile_width = (width - button_width - 84.0).clamp(110.0, 180.0);

        self.simple_button(
            "DEFAULT PROFILE",
            profile_x,
            button_y,
            profile_width,
            46.0,
            surface_alt,
            white,
            12,
        );

        if self.pointer_pressed
            && point_in_rect(
                self.pointer,
                profile_x,
                button_y,
                profile_width,
                46.0,
            )
        {
            self.notice = Some("Profile selection is not implemented yet.".to_owned());
        }

        if let Some(notice) = &self.notice {
            draw_text(
                notice,
                x,
                card_y + card_height + 28.0,
                13.0,
                accent,
            );
        } else {
            draw_text(
                "The launcher is ready. Minecraft gameplay is still under development.",
                x,
                card_y + card_height + 28.0,
                12.0,
                muted,
            );
        }
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
        draw_text(title, x, y + 34.0, 32.0, white);
        draw_text(subtitle, x, y + 62.0, 15.0, muted);

        let card_y = y + 90.0;
        let card_height = (height - card_y - 60.0).max(150.0);

        draw_rectangle(x, card_y, width, card_height, surface);
        draw_text(
            "COMING IN A FUTURE MILESTONE",
            x + 24.0,
            card_y + 42.0,
            12.0,
            accent,
        );
        draw_text(message, x + 24.0, card_y + 82.0, 16.0, white);
    }

    fn draw_settings(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        _height: f32,
        surface: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text("Settings", x, y + 34.0, 32.0, white);
        draw_text(
            "Customize the NexusClient interface.",
            x,
            y + 62.0,
            15.0,
            muted,
        );

        let card_y = y + 90.0;
        let card_width = width.min(650.0);

        draw_rectangle(x, card_y, card_width, 230.0, surface);
        draw_text("APPEARANCE", x + 24.0, card_y + 34.0, 12.0, accent);
        draw_text("Accent color", x + 24.0, card_y + 66.0, 16.0, white);

        let swatch_y = card_y + 86.0;
        let mut swatch_x = x + 24.0;

        for (index, theme) in Theme::PRESETS.iter().enumerate() {
            let color = theme.color();
            let swatch_width = 42.0;
            let swatch_height = 42.0;

            draw_rectangle(
                swatch_x,
                swatch_y,
                swatch_width,
                swatch_height,
                color,
            );

            if index == self.theme_index {
                draw_rectangle_lines(
                    swatch_x - 3.0,
                    swatch_y - 3.0,
                    swatch_width + 6.0,
                    swatch_height + 6.0,
                    2.0,
                    white,
                );
            }

            if self.pointer_pressed
                && point_in_rect(
                    self.pointer,
                    swatch_x,
                    swatch_y,
                    swatch_width,
                    swatch_height,
                )
            {
                self.theme_index = index;
                self.save();
            }

            swatch_x += 58.0;
        }

        let fps_y = card_y + 174.0;
        let fps_label = if self.show_fps {
            "FPS counter: ON"
        } else {
            "FPS counter: OFF"
        };

        self.simple_button(
            fps_label,
            x + 18.0,
            fps_y - 24.0,
            220.0,
            42.0,
            Color::from_rgba(35, 36, 46, 255),
            white,
            14,
        );

        if self.pointer_pressed
            && point_in_rect(self.pointer, x + 18.0, fps_y - 24.0, 220.0, 42.0)
        {
            self.show_fps = !self.show_fps;
            self.save();
        }
    }

    fn draw_about(
        &self,
        x: f32,
        y: f32,
        width: f32,
        _height: f32,
        surface: Color,
        accent: Color,
        white: Color,
        muted: Color,
    ) {
        draw_text("About NexusClient", x, y + 34.0, 32.0, white);
        draw_text(
            "An independent Minecraft Java client project.",
            x,
            y + 62.0,
            15.0,
            muted,
        );

        draw_rectangle(x, y + 90.0, width.min(700.0), 200.0, surface);
        draw_text("NexusClient 0.2.0", x + 24.0, y + 130.0, 22.0, accent);
        draw_text("Built with Rust and Macroquad.", x + 24.0, y + 164.0, 15.0, white);
        draw_text(
            "This build contains the launcher interface only.",
            x + 24.0,
            y + 192.0,
            14.0,
            muted,
        );
        draw_text(
            "Minecraft authentication, protocol and gameplay are not implemented yet.",
            x + 24.0,
            y + 220.0,
            12.0,
            muted,
        );
    }

    fn action_button(
        &self,
        label: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        text_color: Color,
    ) -> bool {
        let hovered = point_in_rect(self.pointer, x, y, width, height);
        let fill = if hovered {
            lighten(color, 1.10)
        } else {
            color
        };

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

        hovered && self.pointer_pressed
    }

    fn simple_button(
        &self,
        label: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        text_color: Color,
        font_size: u16,
    ) {
        let hovered = point_in_rect(self.pointer, x, y, width, height);
        let fill = if hovered {
            lighten(color, 1.15)
        } else {
            color
        };

        draw_rectangle(x, y, width, height, fill);

        let measured = measure_text(label, None, font_size, 1.0);

        draw_text(
            label,
            x + (width - measured.width) / 2.0,
            y + (height + measured.height) / 2.0 - 3.0,
            font_size as f32,
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

fn point_in_rect(
    position: (f32, f32),
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> bool {
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
