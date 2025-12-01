use std::path::PathBuf;

fn main() {
    slint_build::compile("ui/setup-window.slint").expect("Failed to compile Slint UI");

    // 仅在启用 embed_msi 功能时尝试内嵌 MSI。
    let embed = std::env::var("CARGO_FEATURE_EMBED_MSI").is_ok();
    if !embed {
        return;
    }

    let default_msi = format!(
        "{}/../target/wix/Oriv-{}-x86_64.msi",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into())
    );
    let source = std::env::var("ORIV_MSI_PATH").unwrap_or(default_msi);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let out_msi = out_dir.join("oriv.msi");

    std::fs::create_dir_all(&out_dir).expect("create OUT_DIR failed");
    std::fs::copy(&source, &out_msi).unwrap_or_else(|e| {
        panic!(
            "无法复制 MSI 文件用于内嵌，检查 ORIV_MSI_PATH（当前值: {}）: {}",
            source, e
        )
    });

    println!("cargo:rerun-if-env-changed=ORIV_MSI_PATH");
    println!("cargo:rerun-if-changed={}", source);
    println!("cargo:rustc-env=ORIV_MSI_EMBED_PATH={}", out_msi.display());
}
