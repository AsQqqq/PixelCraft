//! The app icon, everywhere Windows, macOS and Linux show one (as the crafting apps do).
//!
//! - **Executable:** `build.rs` embeds `assets/app-icon/pixelcraft.ico` (16–256 px) as icon
//!   resource 1, which Explorer, Alt+Tab and an unpinned taskbar button fall back to.
//! - **Window:** [`window_icon`] sets the title-bar and taskbar icon at runtime (`WM_SETICON` on
//!   Windows, `_NET_WM_ICON` on X11, the Dock when running unbundled on macOS).
//! - **macOS app bundle:** the Dock shows the bundle's `pixelcraft.icns`, which macOS renders in
//!   its own style; a runtime PNG would replace it with a flat picture, so it isn't set there.
//!
//! All of them are rendered from `assets/app-icon/pixelcraft.svg` by `packaging/icons.sh`.

/// Window, taskbar and (when running unbundled) Dock icon. macOS gets the padded 1024 px render
/// on Apple's icon grid; elsewhere the tighter 256 px hicolor render reads better at small sizes.
pub fn window_icon() -> egui::IconData {
    match eframe::icon_data::from_png_bytes(PNG) {
        Ok(icon) => icon,
        Err(e) => {
            log::warn!("app icon: {e}");
            egui::IconData::default()
        }
    }
}

/// Whether the window gets [`window_icon`] at runtime: everywhere except inside a macOS app
/// bundle, whose Dock icon is the bundle's `.icns`. `exe` is the running executable.
pub fn sets_runtime_icon(exe: Option<&std::path::Path>) -> bool {
    !(cfg!(target_os = "macos") && exe.is_some_and(in_app_bundle))
}

/// Whether `exe` is an app bundle's executable: `…/Name.app/Contents/MacOS/binary`.
fn in_app_bundle(exe: &std::path::Path) -> bool {
    let name = |p: Option<&std::path::Path>| p.and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned());
    let macos = exe.parent();
    let contents = macos.and_then(std::path::Path::parent);
    let bundle = contents.and_then(std::path::Path::parent);
    name(macos).as_deref() == Some("MacOS")
        && name(contents).as_deref() == Some("Contents")
        && name(bundle).is_some_and(|n| n.len() > 4 && n.to_ascii_lowercase().ends_with(".app"))
}

#[cfg(target_os = "macos")]
const PNG: &[u8] = include_bytes!("../../../assets/app-icon/pixelcraft-1024.png");
#[cfg(not(target_os = "macos"))]
const PNG: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/ai.storyteller.pixelcraft.png");

#[cfg(test)]
mod tests {
    use super::*;

    const ICO: &[u8] = include_bytes!("../../../assets/app-icon/pixelcraft.ico");

    #[test]
    fn the_window_icon_decodes() {
        let icon = window_icon();
        assert!(icon.width >= 256 && icon.width == icon.height);
        assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
    }

    #[test]
    fn the_ico_has_every_windows_size() {
        assert_eq!(&ICO[..4], &[0, 0, 1, 0]);
        let count = u16::from_le_bytes([ICO[4], ICO[5]]) as usize;
        let sizes: Vec<u32> = (0..count).map(|i| ICO[6 + 16 * i]).map(|b| if b == 0 { 256 } else { u32::from(b) }).collect();
        for s in [16, 24, 32, 48, 256] {
            assert!(sizes.contains(&s), "missing {s} px: {sizes:?}");
        }
    }

    #[test]
    fn only_a_macos_app_bundle_skips_the_runtime_icon() {
        let bundled = std::path::Path::new("/Applications/PixelCraft.app/Contents/MacOS/pixelcraft");
        let loose = std::path::Path::new("/Users/me/PixelCraft/target/release/pixelcraft");
        assert!(sets_runtime_icon(Some(loose)));
        assert!(sets_runtime_icon(None));
        assert_eq!(sets_runtime_icon(Some(bundled)), !cfg!(target_os = "macos"));
    }
}
