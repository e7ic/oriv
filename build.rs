fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app-icon.ico");
        res.compile().unwrap();
    }
}