pub mod engine;
pub mod theme;

mod gpu_app;

pub fn run_desktop() {
    gpu_app::run_desktop();
}

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    gpu_app::run_android(app);
}