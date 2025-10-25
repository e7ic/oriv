fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");

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
        // macOS 应用图标和元数据由 Cargo.toml 中的 [package.metadata.bundle] 配置
        // 使用 cargo bundle 或 cargo-bundle 工具打包时会自动处理
        // 开发构建时，图标通过 Window::icon 属性在运行时设置

        // 未来可以在这里添加自定义的 macOS 构建配置
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
    }

    // Linux 平台配置
    #[cfg(target_os = "linux")]
    {
        // Linux 平台的图标和桌面文件由打包工具处理
        // 可以使用 cargo-deb 或 cargo-appimage 等工具
    }
}