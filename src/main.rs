mod app;
mod theme;

use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "NexusClient".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = app::NexusApp::load();

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
