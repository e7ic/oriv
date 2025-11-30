// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod proxy;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use i_slint_backend_winit::WinitWindowAccessor;
use i_slint_backend_winit::winit::window::ResizeDirection;
use slint::{Model, VecModel, SharedString};
use proxy::server::ProxyServer;
use proxy::server::Rule;
use proxy::ca::CertificateAuthority;

mod ui {
    slint::include_modules!();
}
use ui::*;

/// 设置 Windows DPI 感知以改善字体渲染
#[cfg(target_os = "windows")]
fn set_dpi_awareness() {
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE};

    unsafe {
        // 设置进程为每监视器 DPI 感知
        // 这样可以确保在高 DPI 显示器上字体清晰
        let _ = SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Windows 平台：设置 DPI 感知
    #[cfg(target_os = "windows")]
    set_dpi_awareness();

    // 强制使用 Skia 渲染器以获得最佳字体质量
    // Skia 是 Google Chrome 使用的渲染引擎
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-skia");
    }

    let ui = AppWindow::new()?;
    
    // Create channel for logs
    let (log_sender, mut log_receiver) = tokio::sync::mpsc::channel(100);
    
    let proxy_server = Arc::new(ProxyServer::new(log_sender));
    let proxy_running = Arc::new(Mutex::new(false));

    // Logs Model
    let logs_model = Rc::new(VecModel::default());
    ui.set_logs(logs_model.clone().into());

    // Handle incoming logs
    let ui_handle = ui.as_weak();
    tokio::spawn(async move {
        while let Some(event) = log_receiver.recv().await {
            let ui_handle = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() {
                    let logs_model = ui.get_logs();
                    if let Some(vec_model) = logs_model.as_any().downcast_ref::<VecModel<LogEntry>>() {
                        vec_model.insert(0, LogEntry {
                            time: SharedString::from(&event.time),
                            method: SharedString::from(&event.method),
                            protocol: SharedString::from(&event.protocol),
                            url: SharedString::from(&event.url),
                            status: SharedString::from(&event.status),
                            status_code: event.status_code,
                        });
                        // Keep only last 1000 logs
                        if vec_model.row_count() > 1000 {
                            vec_model.remove(1000);
                        }
                    }
                }
            });
        }
    });

    let logs_model_clone = logs_model.clone();
    ui.on_clear_logs(move || {
        while logs_model_clone.row_count() > 0 {
            logs_model_clone.remove(0);
        }
    });

    // 检测并设置平台类型
    #[cfg(target_os = "macos")]
    {
        ui.set_is_macos(true);

        // 在 macOS 上配置原生标题栏样式
        // 注意：这需要在窗口创建后立即调用
        configure_macos_titlebar(&ui);
    }

    #[cfg(not(target_os = "macos"))]
    {
        ui.set_is_macos(false);
        // 注意：窗口装饰已通过 app-window.slint 中的 no-frame: true 移除
    }

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

    // 开始调整窗口大小
    ui.on_start_resize_window({
        let ui_handle = ui.as_weak();
        move |direction: i32| {
            let ui = ui_handle.unwrap();

            // 将整数方向转换为 ResizeDirection 枚举
            let resize_direction = match direction {
                0 => ResizeDirection::East,       // 右
                1 => ResizeDirection::North,      // 上
                2 => ResizeDirection::NorthEast,  // 右上
                3 => ResizeDirection::NorthWest,  // 左上
                4 => ResizeDirection::South,      // 下
                5 => ResizeDirection::SouthEast,  // 右下
                6 => ResizeDirection::SouthWest,  // 左下
                7 => ResizeDirection::West,       // 左
                _ => return,  // 无效方向，忽略
            };

            ui.window().with_winit_window(|winit_window| {
                let _ = winit_window.drag_resize_window(resize_direction);
            });
        }
    });

    // 规则管理
    let rules_model = Rc::new(VecModel::default());
    ui.set_rules(rules_model.clone().into());

    let proxy_server_clone = proxy_server.clone();
    let rules_model_clone = rules_model.clone();
    ui.on_add_rule(move |domain, target, protocol| {
        let id = uuid::Uuid::new_v4().to_string();
        let rule = ProxyRule {
            id: SharedString::from(&id),
            domain,
            target,
            protocol,
            enabled: true,
        };
        rules_model_clone.push(rule);
        
        update_backend_rules(&proxy_server_clone, &rules_model_clone);
    });

    let proxy_server_clone = proxy_server.clone();
    let rules_model_clone = rules_model.clone();
    ui.on_remove_rule(move |id| {
        let mut index_to_remove = None;
        for (i, rule) in rules_model_clone.iter().enumerate() {
            if rule.id == id {
                index_to_remove = Some(i);
                break;
            }
        }
        if let Some(i) = index_to_remove {
            rules_model_clone.remove(i);
            update_backend_rules(&proxy_server_clone, &rules_model_clone);
        }
    });

    // 证书管理
    let ca = Arc::new(CertificateAuthority::new());
    let certs_model = Rc::new(VecModel::default());
    ui.set_certificates(certs_model.clone().into());

    // Load existing CA if available
    if let Some(info) = ca.load_ca_info() {
        certs_model.push(CertInfo {
            id: SharedString::from(&info.id),
            domain: SharedString::from(&info.domain),
            r#type: SharedString::from(&info.type_),
            issuer: SharedString::from(&info.issuer),
            validity: SharedString::from(&info.validity),
        });
    }

    let ca_clone = ca.clone();
    let certs_model_clone = certs_model.clone();
    ui.on_generate_ca(move || {
        match ca_clone.generate_ca_cert() {
            Ok(info) => {
                while certs_model_clone.row_count() > 0 {
                    certs_model_clone.remove(0);
                }
                
                certs_model_clone.push(CertInfo {
                    id: SharedString::from(&info.id),
                    domain: SharedString::from(&info.domain),
                    r#type: SharedString::from(&info.type_),
                    issuer: SharedString::from(&info.issuer),
                    validity: SharedString::from(&info.validity),
                });
                println!("CA Certificate generated successfully");
            }
            Err(e) => eprintln!("Failed to generate CA: {}", e),
        }
    });

    let ca_clone = ca.clone();
    ui.on_open_cert_dir(move || {
        let path = ca_clone.get_cert_dir();
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(path)
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(path)
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    });

    let certs_model_clone = certs_model.clone();
    ui.on_delete_cert(move |id| {
        let mut index_to_remove = None;
        for (i, cert) in certs_model_clone.iter().enumerate() {
            if cert.id == id {
                index_to_remove = Some(i);
                break;
            }
        }
        if let Some(i) = index_to_remove {
            certs_model_clone.remove(i);
        }
    });

    let ca_clone = ca.clone();
    ui.on_export_cert(move |_id| {
        let cert_path = ca_clone.get_cert_dir().join("ca.crt");
        if !cert_path.exists() {
            eprintln!("Certificate file not found: {:?}", cert_path);
            return;
        }

        let task = rfd::AsyncFileDialog::new()
            .set_title("Export CA Certificate")
            .set_file_name("Oriv_CA.crt")
            .add_filter("Certificate", &["crt", "pem"])
            .save_file();

        let cert_path = cert_path.clone();
        tokio::spawn(async move {
            if let Some(file) = task.await {
                match std::fs::copy(&cert_path, file.path()) {
                    Ok(_) => println!("Certificate exported to: {:?}", file.path()),
                    Err(e) => eprintln!("Failed to export certificate: {}", e),
                }
            }
        });
    });

    // 切换代理
    ui.on_toggle_proxy({
        let proxy_server = proxy_server.clone();
        let proxy_running = proxy_running.clone();
        let ui_handle = ui.as_weak();
        
        move |enable: bool| {
            let mut running = proxy_running.lock().unwrap();
            if enable && !*running {
                let server = proxy_server.clone();
                let ui = ui_handle.unwrap();
                
                // Read configuration from UI
                let port_str = ui.get_http_port();
                let port = port_str.parse::<u16>().unwrap_or(8080);
                
                *running = true;
                tokio::spawn(async move {
                    if let Err(e) = server.start(port).await {
                        eprintln!("Proxy server error: {}", e);
                    }
                });
                println!("Proxy started on port {}", port);
            } else if !enable && *running {
                proxy_server.stop();
                *running = false;
                println!("Proxy stopped");
            }
            
            // Update UI state
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_proxy_running(*running);
            }
        }
    });

    ui.run()?;

    Ok(())
}

fn update_backend_rules(server: &Arc<ProxyServer>, model: &Rc<VecModel<ProxyRule>>) {
    let mut rules = Vec::new();
    for rule in model.iter() {
        rules.push(Rule {
            id: rule.id.to_string(),
            domain: rule.domain.to_string(),
            target: rule.target.to_string(),
            protocol: rule.protocol.to_string(),
            enabled: rule.enabled,
        });
    }
    server.update_rules(rules);
}

/// 配置 macOS 窗口的原生标题栏样式
#[cfg(target_os = "macos")]
fn configure_macos_titlebar(ui: &AppWindow) {
    use i_slint_backend_winit::winit::platform::macos::WindowExtMacOS;

    ui.window().with_winit_window(|winit_window| {
        // 设置标题栏为透明，允许内容延伸到标题栏区域
        winit_window.set_titlebar_transparent(true);

        // 隐藏标题文字（但保留按钮）
        winit_window.set_title_hidden(true);

        // 允许通过窗口背景拖动（补充自定义标题栏的拖动功能）
        winit_window.set_movable_by_window_background(true);
    });
}