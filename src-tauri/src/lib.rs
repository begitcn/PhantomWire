use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};
use tauri_plugin_shell::process::CommandChild;
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

mod easytier;

#[derive(Default)]
pub struct AppState {
    pub children: Mutex<HashMap<String, CommandChild>>,
    pub rpc_ports: Mutex<HashMap<String, u16>>,
}

pub fn ensure_windows_sidecar_dlls(app: &AppHandle) -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }

    let exe_dir = std::env::current_exe()
        .map_err(|error| format!("current_exe 获取失败: {}", error))?
        .parent()
        .ok_or_else(|| "无法获取 exe 目录".to_string())?
        .to_path_buf();

    for dll in ["Packet.dll", "wintun.dll"] {
        let destination = exe_dir.join(dll);
        if !destination.exists() {
            let resource_path = format!("bin/{}", dll);
            let source = app
                .path()
                .resolve(&resource_path, tauri::path::BaseDirectory::Resource)
                .map_err(|error| format!("解析资源路径 {} 失败: {}", resource_path, error))?;
            if source.exists() {
                fs::copy(&source, &destination).map_err(|error| format!("复制 {} 失败: {}", dll, error))?;
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
const WINDOWS_AUTOSTART_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
fn windows_autostart_command() -> Result<String, String> {
    let exe_path = std::env::current_exe().map_err(|error| format!("获取程序路径失败: {}", error))?;
    Ok(format!("\"{}\" --autostart", exe_path.to_string_lossy()))
}

#[cfg(target_os = "windows")]
fn is_windows_autostart_enabled(value_name: &str) -> bool {
    let expected_command = match windows_autostart_command() {
        Ok(command) => command,
        Err(_) => return false,
    };

    let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey(WINDOWS_AUTOSTART_RUN_KEY) {
        Ok(key) => key,
        Err(_) => return false,
    };

    match key.get_value::<String, _>(value_name) {
        Ok(command) => command == expected_command,
        Err(_) => false,
    }
}

#[cfg(target_os = "windows")]
fn set_windows_autostart_enabled(value_name: &str, enabled: bool) -> Result<(), String> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(WINDOWS_AUTOSTART_RUN_KEY)
        .map_err(|error| format!("打开启动项注册表失败: {}", error))?;

    if enabled {
        let command = windows_autostart_command()?;
        key.set_value(value_name, &command)
            .map_err(|error| format!("写入启动项失败: {}", error))
    } else {
        match key.delete_value(value_name) {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("移除启动项失败: {}", error)),
        }
    }
}

fn is_autostart_enabled(value_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        return is_windows_autostart_enabled(value_name);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = value_name;
        false
    }
}

fn set_autostart_enabled(value_name: &str, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return set_windows_autostart_enabled(value_name, enabled);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = value_name;
        let _ = enabled;
        Ok(())
    }
}

#[tauri::command]
fn get_launch_on_login_status(app: AppHandle) -> bool {
    is_autostart_enabled(&app.package_info().name)
}

#[tauri::command]
fn was_launched_from_autostart() -> bool {
    std::env::args().any(|arg| arg == "--autostart")
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let autostart_value_name = app.package_info().name.clone();
            let autostart_enabled = is_autostart_enabled(&autostart_value_name);
            let autostart_item = CheckMenuItem::with_id(app, "autostart", "开机自启", true, autostart_enabled, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "彻底退出", true, None::<&str>)?;
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &autostart_item, &quit_item])?;

            let autostart_item_handle = autostart_item.clone();
            let autostart_value_name_handle = autostart_value_name.clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        let state: State<AppState> = app.state();
                        let mut children = state.children.lock().unwrap();
                        for (_, child) in children.drain() {
                            let _ = child.kill();
                        }
                        let _ = state.rpc_ports.lock().unwrap().drain();
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "autostart" => {
                        let target = autostart_item_handle.is_checked().unwrap_or(false);
                        let _ = set_autostart_enabled(&autostart_value_name_handle, target);
                        let _ = autostart_item_handle.set_checked(is_autostart_enabled(&autostart_value_name_handle));
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            if std::env::args().any(|arg| arg == "--autostart") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_launch_on_login_status,
            was_launched_from_autostart,
            easytier::start_easytier_core,
            easytier::stop_easytier_core,
            easytier::query_easytier_peers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
