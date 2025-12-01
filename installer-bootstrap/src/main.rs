#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui {
    slint::include_modules!();
}

use anyhow::{anyhow, Context, Result};
use slint::{ComponentHandle, SharedString, Weak};
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Seek};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use ui::SetupWindow;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;

const MSI_FILE_NAME: &str = concat!("Oriv-", env!("CARGO_PKG_VERSION"), "-x86_64.msi");

#[cfg(feature = "embed_msi")]
static MSI_BYTES: &[u8] = include_bytes!(env!("ORIV_MSI_EMBED_PATH"));

#[derive(Clone)]
enum MsiSource {
    Embedded(&'static [u8]),
    File(PathBuf),
}

fn to_wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

fn is_elevated() -> bool {
    unsafe {
        let mut handle: HANDLE = 0;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) == 0 {
            return false;
        }
        let mut elev = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len = 0;
        let ok = GetTokenInformation(
            handle,
            TokenElevation,
            &mut elev as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        ok != 0 && elev.TokenIsElevated != 0
    }
}

fn relaunch_elevated() -> Result<()> {
    let exe = std::env::current_exe().context("无法获取当前可执行路径")?;
    let exe_w = to_wide(exe.as_os_str());
    let verb = to_wide(OsStr::new("runas"));
    let res = unsafe { ShellExecuteW(0, verb.as_ptr(), exe_w.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    if res <= 32 {
        Err(anyhow!(
            "需要管理员权限运行（ShellExecuteW 返回 {}）。请右键“以管理员身份运行”。",
            res
        ))
    } else {
        std::process::exit(0);
    }
}

fn default_install_dir() -> PathBuf {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    Path::new(&pf).join("Oriv")
}

#[allow(unreachable_code)]
fn resolve_msi_source() -> Result<MsiSource> {
    #[cfg(feature = "embed_msi")]
    {
        return Ok(MsiSource::Embedded(MSI_BYTES));
    }

    if let Ok(env_path) = std::env::var("ORIV_MSI_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Ok(MsiSource::File(p));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(MSI_FILE_NAME);
            if candidate.exists() {
                return Ok(MsiSource::File(candidate));
            }
            let simple = dir.join("Oriv.msi");
            if simple.exists() {
                return Ok(MsiSource::File(simple));
            }
        }
    }

    let dev = Path::new("target").join("wix").join(MSI_FILE_NAME);
    if dev.exists() {
        return Ok(MsiSource::File(dev));
    }

    Err(anyhow!(
        "未找到 MSI，请先运行 `cargo wix` 或设置环境变量 ORIV_MSI_PATH 指向 MSI 文件。"
    ))
}

fn materialize_msi(src: &MsiSource, staging_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(staging_dir)?;
    let target = staging_dir.join("oriv.msi");
    match src {
        MsiSource::Embedded(bytes) => fs::write(&target, bytes)?,
        MsiSource::File(path) => {
            fs::copy(path, &target)
                .with_context(|| format!("复制 MSI 文件失败: {}", path.display()))?;
        }
    };
    Ok(target)
}

use std::os::windows::process::CommandExt; // Added import

fn run_msiexec(msi: &Path, installdir: &Path, log_path: &Path) -> Result<i32> {
    // Ensure no trailing backslash which can escape quotes
    let dir_str = installdir.to_string_lossy();
    let clean_dir = if dir_str.ends_with('\\') {
        &dir_str[..dir_str.len() - 1]
    } else {
        &dir_str
    };

    println!("Executing: msiexec /i {:?} INSTALLDIR=\"{}\" /qn /norestart /l*vx {:?}", msi, clean_dir, log_path);

    let status = Command::new("msiexec")
        .arg("/i")
        .arg(msi)
        // Use raw_arg to pass the property exactly as formatted, bypassing Rust's auto-quoting.
        // This ensures msiexec receives INSTALLDIR="Path" correctly.
        .raw_arg(format!("INSTALLDIR=\"{}\"", clean_dir))
        .arg("/qn")
        .arg("/norestart")
        .arg("/l*vx")
        .arg(log_path)
        .status()
        .context("调用 msiexec 失败")?;
    Ok(status.code().unwrap_or(-1))
}

fn spawn_log_watcher(log_path: PathBuf, stop: Arc<AtomicBool>, ui: Weak<SetupWindow>) {
    thread::spawn(move || {
        let mut pos: u64 = 0;
        while !stop.load(Ordering::SeqCst) {
            if let Ok(mut f) = fs::File::open(&log_path) {
                if f.seek(std::io::SeekFrom::Start(pos)).is_ok() {
                    let mut buf = String::new();
                    if f.read_to_string(&mut buf).is_ok() {
                        pos += buf.len() as u64;
                        let mut progress = 0.1;
                        if buf.contains("InstallValidate") {
                            progress = 0.2;
                        }
                        if buf.contains("InstallFiles") {
                            progress = 0.5;
                        }
                        if buf.contains("WriteRegistryValues") {
                            progress = 0.7;
                        }
                        if buf.contains("InstallFinalize") {
                            progress = 0.9;
                        }
                        let text = if buf.contains("error") || buf.contains("Error") {
                            "检测到安装日志中出现错误".to_string()
                        } else {
                            "安装进行中...".to_string()
                        };
                        let _ = slint::invoke_from_event_loop({
                            let ui = ui.clone();
                            move || {
                                if let Some(u) = ui.upgrade() {
                                    u.set_progress(progress as f32);
                                    u.set_status_text(SharedString::from(text.clone()));
                                }
                            }
                        });
                    }
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn main() -> Result<()> {
    if !is_elevated() {
        relaunch_elevated()?;
    }

    let msi_source = resolve_msi_source()?;
    let default_dir = default_install_dir();
    let staging_dir = std::env::temp_dir().join("oriv_installer");
    let log_path = staging_dir.join("install.log");
    let log_path_shared = Arc::new(Mutex::new(log_path.clone()));
    let busy = Arc::new(AtomicBool::new(false));

    let ui = SetupWindow::new()?;
    ui.set_install_path(SharedString::from(default_dir.to_string_lossy().into_owned()));
    ui.set_status_text("就绪，点击安装".into());
    ui.set_progress(0.0);
    ui.set_installing(false);

    let ui_handle = ui.as_weak();
    let msi_source_arc = Arc::new(msi_source);
    let staging_dir_arc = Arc::new(staging_dir);

    // 浏览选择路径
    ui.on_browse(move || {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("选择安装目录")
            .pick_folder()
        {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_install_path(SharedString::from(dir.to_string_lossy().into_owned()));
            }
        }
    });

    let ui_handle = ui.as_weak();
    let busy_install = busy.clone();
    let msi_source_install = msi_source_arc.clone();
    let staging_dir_install = staging_dir_arc.clone();
    let log_path_install = log_path_shared.clone();

    ui.on_install(move |path| {
        if busy_install.swap(true, Ordering::SeqCst) {
            return;
        }

        let installdir = PathBuf::from(path.as_str());
        let ui_inner = ui_handle.clone();
        let msi_src = msi_source_install.clone();
        let staging = staging_dir_install.clone();
        let log_path_arc = log_path_install.clone();
        let busy_flag = busy_install.clone();

        thread::spawn(move || {
            let res = (|| -> Result<()> {
                fs::create_dir_all(staging.as_ref())?;
                let log = log_path_arc
                    .lock()
                    .map(|p| p.clone())
                    .unwrap_or_else(|_| staging.join("install.log"));
                if log.exists() {
                    let _ = fs::remove_file(&log);
                }

                let _ = slint::invoke_from_event_loop({
                    let ui = ui_inner.clone();
                    move || {
                        if let Some(u) = ui.upgrade() {
                            u.set_installing(true);
                            u.set_status_text("准备安装...".into());
                            u.set_progress(0.05);
                        }
                    }
                });

                let msi_path = materialize_msi(&msi_src, staging.as_ref())?;
                let stop = Arc::new(AtomicBool::new(false));
                spawn_log_watcher(log.clone(), stop.clone(), ui_inner.clone());

                let code = run_msiexec(&msi_path, &installdir, &log)?;

                stop.store(true, Ordering::SeqCst);

                if code == 0 {
                    slint::invoke_from_event_loop({
                        let ui = ui_inner.clone();
                        move || {
                            if let Some(u) = ui.upgrade() {
                                u.set_progress(1.0);
                                u.set_status_text("安装完成".into());
                                u.set_installing(false);
                            }
                        }
                    })
                    .ok();
                } else {
                    slint::invoke_from_event_loop({
                        let ui = ui_inner.clone();
                        move || {
                            if let Some(u) = ui.upgrade() {
                                u.set_status_text(
                                    SharedString::from(format!("安装失败，退出码 {}", code)),
                                );
                                u.set_installing(false);
                            }
                        }
                    })
                    .ok();
                    return Err(anyhow!("msiexec 退出码 {}", code));
                }

                Ok(())
            })();

            if let Err(e) = res {
                let _ = slint::invoke_from_event_loop({
                    let ui = ui_inner.clone();
                    move || {
                        if let Some(u) = ui.upgrade() {
                            u.set_status_text(SharedString::from(format!("错误: {e}")));
                            u.set_installing(false);
                        }
                    }
                });
            }

            busy_flag.store(false, Ordering::SeqCst);
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_open_log(move || {
        if let Ok(path) = log_path_shared.lock() {
            let path_buf = path.clone();
            if path_buf.exists() {
                let _ = Command::new("explorer").arg(&path_buf).spawn();
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("日志文件不存在".into());
            }
        }
    });

    ui.on_quit(move || {
        std::process::exit(0);
    });

    ui.run()?;
    Ok(())
}
