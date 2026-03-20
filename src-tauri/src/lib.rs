use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

fn ensure_windows_sidecar_dlls(app: &AppHandle) -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }

    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("current_exe 获取失败: {}", e))?
        .parent()
        .ok_or_else(|| "无法获取 exe 目录".to_string())?
        .to_path_buf();

    for dll in ["Packet.dll", "wintun.dll"] {
        let dst = exe_dir.join(dll);
        if !dst.exists() {
            let resource_path = format!("bin/{}", dll);
            let src = app.path().resolve(&resource_path, tauri::path::BaseDirectory::Resource)
                .map_err(|e| format!("解析资源路径 {} 失败: {}", resource_path, e))?;
            
            if src.exists() {
                fs::copy(&src, &dst).map_err(|e| format!("复制 {} 失败: {}", dll, e))?;
            }
        }
    }

    Ok(())
}

struct AppState {
    children: Mutex<HashMap<String, CommandChild>>,
    rpc_ports: Mutex<HashMap<String, u16>>,
}

#[derive(Serialize, Clone)]
struct ETLogEvent {
    network_id: String,
    line: String,
}

#[derive(Serialize, Clone)]
struct ETStatusEvent {
    network_id: String,
    running: bool,
}

fn is_auth_failure_line(line: &str) -> bool {
    let l = line.to_lowercase();
    l.contains("auth") && l.contains("fail")
        || l.contains("authentication failed")
        || l.contains("unauthorized")
        || l.contains("forbidden")
        || l.contains("invalid secret")
        || l.contains("invalid network")
        || line.contains("认证失败")
        || line.contains("鉴权失败")
}

#[derive(Deserialize, Debug)]
struct ETConfig {
    network_name: String,
    network_secret: String,
    peer_url: String,
    hostname: String,
    use_dhcp: bool,
    ipv4: String,
    is_private: bool,
    latency_first: bool,
    magic_dns: bool,
}

fn reserve_rpc_port(state: &State<AppState>, network_id: &str) -> Result<u16, String> {
    let mut rpc_ports = state.rpc_ports.lock().unwrap();
    if let Some(p) = rpc_ports.get(network_id).copied() {
        return Ok(p);
    }

    let base: u16 = 15888;
    let mut offset: u16 = 0;
    for b in network_id.as_bytes() {
        offset = offset.wrapping_add(*b as u16);
    }

    for i in 0..512u16 {
        let port = base.wrapping_add((offset.wrapping_add(i)) % 2000);
        if port < 1024 {
            continue;
        }
        if rpc_ports.values().any(|&p| p == port) {
            continue;
        }
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            rpc_ports.insert(network_id.to_string(), port);
            return Ok(port);
        }
    }

    Err("无法为 rpc-portal 分配可用端口".to_string())
}

#[tauri::command]
async fn start_easytier_core(app: AppHandle, state: State<'_, AppState>, network_id: String, config: ETConfig) -> Result<String, String> {
    {
        let children = state.children.lock().unwrap();
        if children.contains_key(&network_id) {
            return Err("EasyTier 已在运行中".into());
        }
    }

    let rpc_port = reserve_rpc_port(&state, &network_id)?;
    let rpc_portal = format!("127.0.0.1:{}", rpc_port);

    // 严格匹配用户测试成功的命令行参数
    let mut args = Vec::new();
    args.push("--instance-name".into()); args.push(network_id.clone());
    args.push("--rpc-portal".into()); args.push(rpc_portal.clone());
    args.push("--network-name".into()); args.push(config.network_name);
    args.push("--network-secret".into()); args.push(config.network_secret);

    if !config.peer_url.is_empty() {
        args.push("--peers".into()); args.push(config.peer_url);
    }

    if !config.hostname.is_empty() {
        args.push("--hostname".into()); args.push(config.hostname);
    }

    if config.use_dhcp {
        args.push("--dhcp".into());
    } else if !config.ipv4.is_empty() {
        args.push("--ipv4".into()); args.push(config.ipv4);
    }

    if config.is_private {
        args.push("--private-mode".into()); args.push("true".into());
    }

    if config.latency_first {
        args.push("--latency-first".into());
    }

    if config.magic_dns {
        args.push("--accept-dns".into()); args.push("true".into());
    }

    let args_log = format!("easytier-core {}", args.join(" "));
    let _ = app.emit(
        "et-log",
        ETLogEvent {
            network_id: network_id.clone(),
            line: format!(">>> 启动参数: {}", args_log),
        },
    );

    let _ = app.emit(
        "et-log",
        ETLogEvent {
            network_id: network_id.clone(),
            line: format!(">>> 管理接口: {} (easytier-cli -p {} node/peer/route ...)", rpc_portal, rpc_portal),
        },
    );

    ensure_windows_sidecar_dlls(&app)?;

    // 启动 Sidecar (Tauri 会自动隐藏 CMD 窗口)
    let sidecar_command = app.shell()
        .sidecar("easytier-core")
        .map_err(|e| format!("Sidecar 错误: {}", e))?
        .args(&args);

    let (mut rx, child) = match sidecar_command.spawn() {
        Ok(v) => v,
        Err(e) => {
            let _ = state.rpc_ports.lock().unwrap().remove(&network_id);
            return Err(format!("启动失败: {}", e));
        }
    };

    {
        let mut children = state.children.lock().unwrap();
        children.insert(network_id.clone(), child);
    }

    let _ = app.emit(
        "et-status",
        ETStatusEvent {
            network_id: network_id.clone(),
            running: true,
        },
    );

    // 日志实时回传
    tauri::async_runtime::spawn(async move {
        let nid = network_id.clone();
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let log = String::from_utf8_lossy(&line).to_string();
                    if is_auth_failure_line(&log) {
                        let _ = app.emit(
                            "et-log",
                            ETLogEvent {
                                network_id: nid.clone(),
                                line: "[auth-failed] 检测到认证失败，正在停止进程".to_string(),
                            },
                        );
                        let state: State<AppState> = app.state();
                        if let Some(c) = state.children.lock().unwrap().remove(&nid) {
                            let _ = c.kill();
                        }
                        let _ = state.rpc_ports.lock().unwrap().remove(&nid);
                        let _ = app.emit(
                            "et-status",
                            ETStatusEvent {
                                network_id: nid.clone(),
                                running: false,
                            },
                        );
                    }

                    let _ = app.emit(
                        "et-log",
                        ETLogEvent {
                            network_id: nid.clone(),
                            line: log,
                        },
                    );
                }
                CommandEvent::Stderr(line) => {
                    let log = String::from_utf8_lossy(&line).to_string();

                    if is_auth_failure_line(&log) {
                        let _ = app.emit(
                            "et-log",
                            ETLogEvent {
                                network_id: nid.clone(),
                                line: "[auth-failed] 检测到认证失败，正在停止进程".to_string(),
                            },
                        );
                        let state: State<AppState> = app.state();
                        if let Some(c) = state.children.lock().unwrap().remove(&nid) {
                            let _ = c.kill();
                        }
                        let _ = state.rpc_ports.lock().unwrap().remove(&nid);
                        let _ = app.emit(
                            "et-status",
                            ETStatusEvent {
                                network_id: nid.clone(),
                                running: false,
                            },
                        );
                    }

                    let _ = app.emit(
                        "et-log",
                        ETLogEvent {
                            network_id: nid.clone(),
                            line: format!("[stderr] {}", log),
                        },
                    );
                }
                CommandEvent::Terminated(payload) => {
                    let _ = app.emit(
                        "et-log",
                        ETLogEvent {
                            network_id: nid.clone(),
                            line: format!("[terminated] {:?}", payload),
                        },
                    );

                    let state: State<AppState> = app.state();
                    let _ = state.children.lock().unwrap().remove(&nid);
                    let _ = state.rpc_ports.lock().unwrap().remove(&nid);
                    let _ = app.emit(
                        "et-status",
                        ETStatusEvent {
                            network_id: nid.clone(),
                            running: false,
                        },
                    );
                }
                CommandEvent::Error(err) => {
                    let _ = app.emit(
                        "et-log",
                        ETLogEvent {
                            network_id: nid.clone(),
                            line: format!("[event-error] {}", err),
                        },
                    );

                    let state: State<AppState> = app.state();
                    let _ = state.children.lock().unwrap().remove(&nid);
                    let _ = state.rpc_ports.lock().unwrap().remove(&nid);
                    let _ = app.emit(
                        "et-status",
                        ETStatusEvent {
                            network_id: nid.clone(),
                            running: false,
                        },
                    );
                }
                _ => {}
            }
        }
    });

    Ok(format!("服务指令下发成功: {}", args_log))
}

#[tauri::command]
async fn stop_easytier_core(app: AppHandle, state: State<'_, AppState>, network_id: String) -> Result<String, String> {
    let mut children = state.children.lock().unwrap();
    if let Some(child) = children.remove(&network_id) {
        let _ = child.kill();
        let _ = state.rpc_ports.lock().unwrap().remove(&network_id);
        let _ = app.emit(
            "et-status",
            ETStatusEvent {
                network_id,
                running: false,
            },
        );
        Ok("服务已停止".into())
    } else {
        Err("未发现运行中的进程".into())
    }
}

#[derive(Serialize, Clone)]
struct ETPeerEntry {
    hostname: Option<String>,
    ipv4: Option<String>,
    ipv6: Option<String>,
    cost: Option<String>,
    latency_ms: Option<f64>,
    loss_rate: Option<String>,
    rx_bytes: Option<String>,
    tx_bytes: Option<String>,
    tunnel_proto: Option<String>,
    nat_type: Option<String>,
    version: Option<String>,
    cidr: Option<String>,
    raw: serde_json::Value,
}

fn find_first_str(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn find_first_f64(v: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        let x = v.get(*k)?;
        if let Some(n) = x.as_f64() {
            return Some(n);
        }
        if let Some(n) = x.as_i64() {
            return Some(n as f64);
        }
        if let Some(n) = x.as_u64() {
            return Some(n as f64);
        }
        if let Some(s) = x.as_str() {
            let s = s.trim();
            if s == "-" {
                continue;
            }
            let s = s.strip_suffix("ms").unwrap_or(s).trim();
            if let Ok(n) = s.parse::<f64>() {
                return Some(n);
            }
        }
    }
    None
}

fn find_cost(v: &serde_json::Value) -> Option<String> {
    if let Some(s) = find_first_str(v, &["cost", "link_cost", "conn_type", "link_type", "transport", "path", "tunnel"]) {
        return Some(s);
    }
    if let Some(n) = v.get("cost").and_then(|x| x.as_i64()) {
        return Some(n.to_string());
    }
    if let Some(n) = v.get("cost").and_then(|x| x.as_u64()) {
        return Some(n.to_string());
    }
    if let Some(n) = v.get("cost").and_then(|x| x.as_f64()) {
        return Some(format!("{}", n));
    }
    if let Some(peer) = v.get("peer") {
        return find_cost(peer);
    }
    None
}

fn parse_peers_json(v: serde_json::Value) -> Vec<ETPeerEntry> {
    let arr = if v.is_array() {
        v.as_array().cloned().unwrap_or_default()
    } else if let Some(a) = v.get("peers").and_then(|x| x.as_array()) {
        a.clone()
    } else if let Some(a) = v.get("data").and_then(|x| x.as_array()) {
        a.clone()
    } else {
        vec![v]
    };

    arr.into_iter()
        .map(|item| {
            let hostname = find_first_str(&item, &["hostname", "name", "peer_hostname"])
                .or_else(|| item.get("peer").and_then(|p| find_first_str(p, &["hostname", "name"])));

            let ipv4 = find_first_str(&item, &["ipv4", "ip", "ip_addr", "vpn_ipv4"])
                .or_else(|| item.get("peer").and_then(|p| find_first_str(p, &["ipv4", "ip"])));

            let ipv6 = find_first_str(&item, &["ipv6", "vpn_ipv6"])
                .or_else(|| item.get("peer").and_then(|p| find_first_str(p, &["ipv6"])));

            let cost = find_cost(&item);

            let loss_rate = find_first_str(&item, &["loss_rate", "loss", "loss(%)", "loss_percent"]);
            let rx_bytes = find_first_str(&item, &["rx_bytes", "rx", "rx_kb", "rx_bytes_total"]);
            let tx_bytes = find_first_str(&item, &["tx_bytes", "tx", "tx_kb", "tx_bytes_total"]);
            let tunnel_proto = find_first_str(&item, &["tunnel_proto", "tunnel", "tunnel_protocol", "proto"]);
            let nat_type = find_first_str(&item, &["nat_type", "nat", "natType"]);
            let version = find_first_str(&item, &["version", "ver"]);
            let cidr = find_first_str(&item, &["cidr", "ipv4_cidr", "vpn_cidr"]);

            let latency_ms = find_first_f64(&item, &["lat_ms", "latency_ms", "latency", "lat", "lat(ms)", "rtt_ms", "rtt", "avg_rtt_ms", "avg_rtt"])
                .or_else(|| item.get("stats").and_then(|s| find_first_f64(s, &["lat_ms", "latency_ms", "latency", "lat", "lat(ms)", "rtt_ms", "rtt", "avg_rtt_ms", "avg_rtt"])))
                .or_else(|| item.get("link").and_then(|s| find_first_f64(s, &["lat_ms", "latency_ms", "lat", "lat(ms)", "rtt_ms", "rtt"])))
                .or_else(|| item.get("peer").and_then(|p| p.get("stats")).and_then(|s| find_first_f64(s, &["lat_ms", "latency_ms", "lat", "lat(ms)", "rtt_ms", "rtt"])));

            ETPeerEntry {
                hostname,
                ipv4,
                ipv6,
                cost,
                latency_ms,
                loss_rate,
                rx_bytes,
                tx_bytes,
                tunnel_proto,
                nat_type,
                version,
                cidr,
                raw: item,
            }
        })
        .collect()
}

#[tauri::command]
async fn query_easytier_peers(app: AppHandle, state: State<'_, AppState>, network_id: String) -> Result<Vec<ETPeerEntry>, String> {
    let rpc_port = state
        .rpc_ports
        .lock()
        .unwrap()
        .get(&network_id)
        .copied()
        .ok_or_else(|| "该网络未运行或未分配 rpc-portal".to_string())?;
    let rpc_portal = format!("127.0.0.1:{}", rpc_port);

    let out = app.shell()
        .sidecar("easytier-cli")
        .map_err(|e| format!("Sidecar easytier-cli 错误: {}", e))?
        .args(["-p", &rpc_portal, "-o", "json", "peer"])
        .output()
        .await
        .map_err(|e| format!("执行 easytier-cli 失败: {}", e))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        return Err(format!("easytier-cli 返回失败: {}{}", stdout, stderr));
    }

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析 easytier-cli JSON 失败: {}", e))?;
    Ok(parse_peers_json(json))
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState { children: Mutex::new(HashMap::new()), rpc_ports: Mutex::new(HashMap::new()) })
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 托盘菜单配置
            let quit_i = MenuItem::with_id(app, "quit", "彻底退出", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        let state: State<AppState> = app.state();
                        let mut children = state.children.lock().unwrap();
                        for (_, c) in children.drain() {
                            let _ = c.kill();
                        }
                        let _ = state.rpc_ports.lock().unwrap().drain();
                        app.exit(0);
                    }
                    "show" => { if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); } }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().unwrap();
            }
        })
        .invoke_handler(tauri::generate_handler![start_easytier_core, stop_easytier_core, query_easytier_peers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}