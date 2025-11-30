fn main() {
    let mut library_paths = std::collections::HashMap::new();
    // Guessing the API
    let path = lucide_slint::get_slint_file_path(); 
    library_paths.insert("lucide".to_string(), std::path::PathBuf::from(path));
    
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library_paths);
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

    // Linux 平台配置
    #[cfg(target_os = "linux")]
    {
    }
}