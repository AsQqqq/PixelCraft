//! Windows only: embed the app icon and version info (VERSIONINFO) into `pixelcraft.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PIXELCRAFT_REQUIRE_WINRES=1` (for
//! release builds) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/pixelcraft.ico");
    println!("cargo:rerun-if-env-changed=PIXELCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/pixelcraft.ico")
        .set("ProductName", "PixelCraft")
        .set("FileDescription", "PixelCraft pixel-art editor")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the PixelCraft contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "pixelcraft.exe")
        .set("InternalName", "pixelcraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("PIXELCRAFT_REQUIRE_WINRES").is_some() {
            println!("cargo:warning=embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=pixelcraft.exe built without icon/version resources: {e}");
    }
}
