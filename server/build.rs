use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("wasm32") {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // When building via Nix (crate2nix), the frontend WASM and JS bindings
    // are pre-built in a separate derivation.  Just copy them in and skip
    // the inner `cargo build` / `wasm-bindgen` invocations (which would
    // fail in a Nix sandbox because there is no network access).
    if let Ok(nix_frontend_dir) = std::env::var("NIX_FRONTEND_DIR") {
        let src = std::path::Path::new(&nix_frontend_dir);
        let dst = std::path::Path::new(&out_dir);
        copy_dir_all(src, dst).expect("failed to copy Nix frontend assets");
        generate_embedded_module(dst);
        println!("cargo:rerun-if-changed=../frontend/src");
        println!("cargo:rerun-if-changed=../frontend/styles.css");
        println!("cargo:rerun-if-changed=../frontend/pieces");
        println!("cargo:rerun-if-changed=../Cargo.lock");
        return;
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap();
    let frontend_dir = workspace_root.join("frontend");

    // Build the frontend package for WASM using a separate target dir
    // to avoid cargo lock contention with the current build.
    let wasm_target_dir = Path::new(&manifest_dir).join("target-wasm-build");

    let status = std::process::Command::new("cargo")
        .args([
            "build",
            "--package",
            "frontend",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--target-dir",
        ])
        .arg(&wasm_target_dir)
        .current_dir(workspace_root)
        .status()
        .expect("failed to build frontend WASM binary");
    assert!(
        status.success(),
        "frontend WASM build failed — is the wasm32 target installed? (rustup target add wasm32-unknown-unknown)"
    );

    // Run wasm-bindgen to generate JS glue code
    let wasm_file = wasm_target_dir.join("wasm32-unknown-unknown/release/frontend.wasm");

    let status = std::process::Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-dir"])
        .arg(&out_dir)
        .arg(&wasm_file)
        .status()
        .expect("failed to run wasm-bindgen — is wasm-bindgen-cli installed?");
    assert!(status.success(), "wasm-bindgen failed");

    // Copy static assets (CSS + piece SVGs)
    let assets_out = Path::new(&out_dir);

    std::fs::copy(frontend_dir.join("styles.css"), assets_out.join("styles.css")).ok();

    let pieces_src = frontend_dir.join("pieces");
    let pieces_dst = assets_out.join("pieces");
    if pieces_src.exists() {
        std::fs::create_dir_all(&pieces_dst).ok();
        for entry in std::fs::read_dir(&pieces_src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                std::fs::copy(&path, pieces_dst.join(entry.file_name())).ok();
            }
        }
    }

    // Generate index.html with correct script/link tags
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust Chess</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <script type="module">
        import init from '/frontend.js';
        await init('/frontend_bg.wasm');
    </script>
</body>
</html>"#;
    std::fs::write(assets_out.join("index.html"), html).unwrap();

    // Generate embedded.rs module so the server binary can include the
    // assets at compile time via include_bytes!
    generate_embedded_module(assets_out);

    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/styles.css");
    println!("cargo:rerun-if-changed=../frontend/pieces");
    println!("cargo:rerun-if-changed=../Cargo.lock");
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn generate_embedded_module(out_dir: &Path) {
    use std::io::Write;

    let module_path = out_dir.join("embedded.rs");
    let mut f = std::fs::File::create(&module_path).unwrap();

    writeln!(
        f,
        "#[derive(Copy, Clone)]\npub struct Asset {{ pub path: &'static str, pub data: &'static [u8] }}"
    )
    .unwrap();

    let mut all = Vec::new();

    let mut add = |name: &str, file_rel: &str| {
        let var = name
            .trim_start_matches("pieces/")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
            .to_uppercase();
        let var = format!("FILE_{}", var);
        writeln!(
            f,
            "pub static {var}: Asset = Asset {{ path: \"/{name}\", data: include_bytes!(\"{file_rel}\") }};",
        )
        .unwrap();
        all.push(var);
    };

    add("index.html", "index.html");
    add("styles.css", "styles.css");
    add("frontend.js", "frontend.js");
    add("frontend_bg.wasm", "frontend_bg.wasm");

    let pieces_dir = out_dir.join("pieces");
    if pieces_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&pieces_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in &entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            add(
                &format!("pieces/{}", name_str),
                &format!("pieces/{}", name_str),
            );
        }
    }

    writeln!(f, "pub const ALL_ASSETS: &[Asset] = &[").unwrap();
    for var in &all {
        writeln!(f, "    {var},").unwrap();
    }
    writeln!(f, "];").unwrap();
}
