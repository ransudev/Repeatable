#[cfg(target_os = "windows")]
fn main() {
    use std::path::PathBuf;
    use image::imageops::FilterType;

    println!("cargo:rerun-if-changed=src/assets/logo.png");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let icon_path = PathBuf::from(out_dir).join("repeatable-icon.ico");

    let image = image::open("src/assets/logo.png")
        .expect("failed to open src/assets/logo.png")
        .resize(256, 256, FilterType::Lanczos3);
    image
        .save(&icon_path)
        .expect("failed to generate .ico icon from logo.png");

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path.to_string_lossy().as_ref());
    res.compile().expect("failed to compile Windows resources");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
