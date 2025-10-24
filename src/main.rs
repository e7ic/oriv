// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use i_slint_backend_winit::WinitWindowAccessor;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    // 最小化窗口
    ui.on_minimize_window({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.window().set_minimized(true);
        }
    });

    // 最大化/恢复窗口
    ui.on_maximize_window({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let window = ui.window();
            let is_maximized = window.is_maximized();
            window.set_maximized(!is_maximized);
            !is_maximized  // 返回新的最大化状态
        }
    });

    // 关闭窗口
    ui.on_close_window({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.window().hide().ok();
            std::process::exit(0);
        }
    });

    // 开始拖动窗口
    ui.on_start_drag_window({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.window().with_winit_window(|winit_window| {
                let _ = winit_window.drag_window();
            });
        }
    });

    // 切换主题
    ui.on_toggle_theme({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let current_mode = ui.get_dark_mode();
            ui.set_dark_mode(!current_mode);
        }
    });

    ui.run()?;

    Ok(())
}