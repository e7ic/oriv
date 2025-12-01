use std::{collections::HashMap, path::PathBuf};

fn main() {
    let library = HashMap::from([(
        "lucide".to_string(),
        PathBuf::from(lucide_slint::get_slint_file_path().to_string()),
    )]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");

    // Windows 平台配置
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app-icon.ico");
        res.compile().unwrap();
    }

    // macOS 平台配置
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
    }
}