use std::path::Path;
use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm32") {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest_dir);
    let ws_root = manifest.parent().unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let wasm_target_dir = Path::new(&out_dir).join("wasm-target");
    let wasm_built = wasm_target_dir.join("wasm32-unknown-unknown/debug/app.wasm");

    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            wasm_target_dir.to_str().unwrap(),
            "-p",
            "app",
        ])
        .status()
        .expect("Failed to spawn cargo build for WASM target");

    assert!(status.success(), "WASM build failed");

    let dist_dir = ws_root.join("dist");
    std::fs::create_dir_all(&dist_dir).unwrap();

    if wasm_built.exists() {
        let status = Command::new("wasm-bindgen")
            .args([
                "--target",
                "web",
                "--out-dir",
                dist_dir.to_str().unwrap(),
                "--out-name",
                "chess",
                wasm_built.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to run wasm-bindgen");

        assert!(status.success(), "wasm-bindgen failed");
    }

    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Chess</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <script type="module">
        import init from './chess.js';
        init();
    </script>
</body>
</html>"#;

    std::fs::write(dist_dir.join("index.html"), html).unwrap();
    std::fs::copy(ws_root.join("styles.css"), dist_dir.join("styles.css")).unwrap();

    let pieces_dir = ws_root.join("pieces");
    let pieces_out = dist_dir.join("pieces");
    if pieces_dir.exists() {
        std::fs::create_dir_all(&pieces_out).unwrap();
        for entry in std::fs::read_dir(&pieces_dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            std::fs::copy(entry.path(), pieces_out.join(&name)).unwrap();
        }
    }

    let src = ws_root.join("server/src");
    if src.exists() {
        println!("cargo::rerun-if-changed={}", src.display());
    }
    let core_src = ws_root.join("chess/src");
    if core_src.exists() {
        println!("cargo::rerun-if-changed={}", core_src.display());
    }
    let wasm_src = ws_root.join("app/src");
    if wasm_src.exists() {
        println!("cargo::rerun-if-changed={}", wasm_src.display());
    }
    let css = ws_root.join("styles.css");
    if css.exists() {
        println!("cargo::rerun-if-changed={}", css.display());
    }
    let pieces = ws_root.join("pieces");
    if pieces.exists() {
        println!("cargo::rerun-if-changed={}", pieces.display());
    }
    let cargo_toml = manifest.join("Cargo.toml");
    if cargo_toml.exists() {
        println!("cargo::rerun-if-changed={}", cargo_toml.display());
    }
}
