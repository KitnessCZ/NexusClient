# NexusClient

NexusClient is a Rust-first project targeting Minecraft Java Edition, with an Android ARM64 application target.

**Current milestone:** interactive application shell with a red default theme, theme presets, settings persistence, FPS display, and placeholder navigation. It is not yet a Minecraft-compatible game client. There is no world renderer, Minecraft protocol implementation, Microsoft login, or playable singleplayer/multiplayer in this milestone.

## Current implementation

- Rust application using Macroquad.
- Android APK packaging metadata for `aarch64-linux-android`.
- Responsive main menu, settings screen, about screen.
- Six accent-color presets; red is the default.
- FPS counter toggle (F3).
- Saves theme and FPS preference.
- Mouse input; touch and external input behavior must be verified on target devices.

## Build on Linux

Install Rust, Android Studio (SDK), and the Android NDK. Then install the Android Rust target and cargo-apk:

```bash
rustup update stable
rustup target add aarch64-linux-android
cargo install cargo-apk
```

Set `ANDROID_HOME` to your Android SDK path and `ANDROID_NDK_HOME` to your NDK path. For example, adjust these paths to your installation:

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<installed-ndk-version>"
```

Build the APK:

```bash
cargo apk build --release
```

The generated APK is normally placed under `target/android-artifacts/`. Check the command output for the exact path.

For a desktop development preview:

```bash
cargo run
```

## Android compatibility

The package metadata requests Android API 21 as the minimum and targets ARM64. Actual compatibility depends on the selected NDK, graphics driver, Android version, and Macroquad/miniquad backend support. Android 5 devices are old and often have limited graphics-driver support; test on real hardware before promising support.

## Planned milestones

1. Application foundation and packaging.
2. 3D renderer and block/chunk rendering.
3. World generation and player physics.
4. Minecraft Java protocol and multiplayer.
5. Microsoft authentication and offline-mode support where permitted by servers.
6. Resource packs, shaders, skins, HUD, settings and input customization.
7. Mod-loading architecture.

## Licensing and game assets

This repository contains no proprietary Minecraft code or assets. A compatible client must use assets and trademarks lawfully and must not distribute Mojang/Microsoft assets without permission. Users should supply assets from a legitimately obtained game installation where applicable.


## Build an APK using only an Android phone (GitHub Actions)

1. Sign in to GitHub in your phone browser.
2. Create a new **public or private** repository named `NexusClient`.
3. Upload the contents of this project ZIP to the repository (including the hidden `.github` folder). If the GitHub mobile upload interface hides dot-folders, use GitHub's web editor to create `.github/workflows/android-apk.yml` and paste the workflow from this repository.
4. Open the repository's **Actions** tab.
5. Select **Build NexusClient APK** and tap **Run workflow**. A push to `main` also starts a build automatically.
6. Wait for the workflow to finish successfully.
7. Open the completed workflow run, scroll to **Artifacts**, and download `NexusClient-Android-ARM64`.
8. Extract the downloaded artifact ZIP and install the APK on your phone. Android may ask you to allow installs from your browser or file manager.

The workflow uses Macroquad's documented Android Docker build route and packages only the ARM64 target configured in `Cargo.toml`. See https://macroquad.rs/articles/android/ for the upstream build instructions.

If the workflow fails, open the failed run and copy the first red error section. A failed build does not produce an installable APK.
