use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::TcpListener;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[derive(Deserialize, Debug, Clone)]
pub struct ETConfig {
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

#[derive(Serialize, Clone)]
pub struct ETPeerEntry {
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
    raw: Value,
}

fn is_auth_failure_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    (lower.contains("auth") && lower.contains("fail"))
        || lower.contains("authentication failed")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("invalid secret")
        || lower.contains("invalid network")
        || line.contains("认证失败")
        || line.contains("鉴权失败")
}

fn reserve_rpc_port(state: &State<AppState>, network_id: &str) -> Result<u16, String> {
    let mut rpc_ports = state.rpc_ports.lock().unwrap();
    if let Some(port) = rpc_ports.get(network_id).copied() {
        return Ok(port);
    }

    let base: u16 = 15888;
    let mut offset: u16 = 0;
    for byte in network_id.as_bytes() {
        offset = offset.wrapping_add(*byte as u16);
    }

    for index in 0..512u16 {
        let port = base.wrapping_add((offset.wrapping_add(index)) % 2000);
        if port < 1024 || rpc_ports.values().any(|&existing| existing == port) {
            continue;
        }
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            rpc_ports.insert(network_id.to_string(), port);
            return Ok(port);
        }
    }

    Err("无法为 rpc-portal 分配可用端口".to_string())
}

fn build_easytier_args(network_id: &str, config: &ETConfig, rpc_portal: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();
    args.push("--instance-name".into());
    args.push(network_id.to_string());

    if let Some(rpc_portal) = rpc_portal {
        args.push("--rpc-portal".into());
        args.push(rpc_portal.to_string());
    }

    args.push("--network-name".into());
    args.push(config.network_name.clone());
    args.push("--network-secret".into());
    args.push(config.network_secret.clone());

    if !config.peer_url.trim().is_empty() {
        args.push("--peers".into());
        args.push(config.peer_url.trim().to_string());
    }
    if !config.hostname.trim().is_empty() {
        args.push("--hostname".into());
        args.push(config.hostname.trim().to_string());
    }
    if config.use_dhcp {
        args.push("--dhcp".into());
    } else if !config.ipv4.trim().is_empty() {
        args.push("--ipv4".into());
        args.push(config.ipv4.trim().to_string());
    }
    if config.is_private {
        args.push("--private-mode".into());
        args.push("true".into());
    }
    if config.latency_first {
        args.push("--latency-first".into());
    }
    if config.magic_dns {
        args.push("--accept-dns".into());
        args.push("true".into());
    }

    args
}

#[tauri::command]
pub async fn start_easytier_core(
    app: AppHandle,
    state: State<'_, AppState>,
    network_id: String,
    config: ETConfig,
) -> Result<String, String> {
    {
        let children = state.children.lock().unwrap();
        if children.contains_key(&network_id) {
            return Err("EasyTier 已在运行中".into());
        }
    }

    let rpc_port = reserve_rpc_port(&state, &network_id)?;
    let rpc_portal = format!("127.0.0.1:{}", rpc_port);
    let args = build_easytier_args(&network_id, &config, Some(&rpc_portal));

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

    crate::ensure_windows_sidecar_dlls(&app)?;

    let sidecar_command = app
        .shell()
        .sidecar("easytier-core")
        .map_err(|error| format!("Sidecar 错误: {}", error))?
        .args(&args);

    let (mut rx, child) = match sidecar_command.spawn() {
        Ok(value) => value,
        Err(error) => {
            let _ = state.rpc_ports.lock().unwrap().remove(&network_id);
            return Err(format!("启动失败: {}", error));
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
                        if let Some(child) = state.children.lock().unwrap().remove(&nid) {
                            let _ = child.kill();
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
                        if let Some(child) = state.children.lock().unwrap().remove(&nid) {
                            let _ = child.kill();
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
                CommandEvent::Error(error) => {
                    let _ = app.emit(
                        "et-log",
                        ETLogEvent {
                            network_id: nid.clone(),
                            line: format!("[event-error] {}", error),
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
pub async fn stop_easytier_core(
    app: AppHandle,
    state: State<'_, AppState>,
    network_id: String,
) -> Result<String, String> {
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

fn find_first_str(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(*key).and_then(|item| item.as_str()) {
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn find_first_f64(value: &Value, keys: &[&str]) -> Option<f64> {
    for key in keys {
        let item = value.get(*key)?;
        if let Some(number) = item.as_f64() {
            return Some(number);
        }
        if let Some(number) = item.as_i64() {
            return Some(number as f64);
        }
        if let Some(number) = item.as_u64() {
            return Some(number as f64);
        }
        if let Some(text) = item.as_str() {
            let trimmed = text.trim();
            if trimmed == "-" {
                continue;
            }
            let cleaned = trimmed.strip_suffix("ms").unwrap_or(trimmed).trim();
            if let Ok(number) = cleaned.parse::<f64>() {
                return Some(number);
            }
        }
    }
    None
}

fn find_cost(value: &Value) -> Option<String> {
    if let Some(cost) = find_first_str(value, &["cost", "link_cost", "conn_type", "link_type", "transport", "path", "tunnel"]) {
        return Some(cost);
    }
    if let Some(number) = value.get("cost").and_then(|item| item.as_i64()) {
        return Some(number.to_string());
    }
    if let Some(number) = value.get("cost").and_then(|item| item.as_u64()) {
        return Some(number.to_string());
    }
    if let Some(number) = value.get("cost").and_then(|item| item.as_f64()) {
        return Some(number.to_string());
    }
    if let Some(peer) = value.get("peer") {
        return find_cost(peer);
    }
    None
}

fn parse_peers_json(value: Value) -> Vec<ETPeerEntry> {
    let items = if value.is_array() {
        value.as_array().cloned().unwrap_or_default()
    } else if let Some(array) = value.get("peers").and_then(|item| item.as_array()) {
        array.clone()
    } else if let Some(array) = value.get("data").and_then(|item| item.as_array()) {
        array.clone()
    } else {
        vec![value]
    };

    items
        .into_iter()
        .map(|item| ETPeerEntry {
            hostname: find_first_str(&item, &["hostname", "name", "peer_hostname"])
                .or_else(|| item.get("peer").and_then(|peer| find_first_str(peer, &["hostname", "name"]))),
            ipv4: find_first_str(&item, &["ipv4", "ip", "ip_addr", "vpn_ipv4"])
                .or_else(|| item.get("peer").and_then(|peer| find_first_str(peer, &["ipv4", "ip"]))),
            ipv6: find_first_str(&item, &["ipv6", "vpn_ipv6"])
                .or_else(|| item.get("peer").and_then(|peer| find_first_str(peer, &["ipv6"]))),
            cost: find_cost(&item),
            loss_rate: find_first_str(&item, &["loss_rate", "loss", "loss(%)", "loss_percent"]),
            rx_bytes: find_first_str(&item, &["rx_bytes", "rx", "rx_kb", "rx_bytes_total"]),
            tx_bytes: find_first_str(&item, &["tx_bytes", "tx", "tx_kb", "tx_bytes_total"]),
            tunnel_proto: find_first_str(&item, &["tunnel_proto", "tunnel", "tunnel_protocol", "proto"]),
            nat_type: find_first_str(&item, &["nat_type", "nat", "natType"]),
            version: find_first_str(&item, &["version", "ver"]),
            cidr: find_first_str(&item, &["cidr", "ipv4_cidr", "vpn_cidr"]),
            latency_ms: find_first_f64(&item, &["lat_ms", "latency_ms", "latency", "lat", "lat(ms)", "rtt_ms", "rtt", "avg_rtt_ms", "avg_rtt"])
                .or_else(|| item.get("stats").and_then(|stats| find_first_f64(stats, &["lat_ms", "latency_ms", "latency", "lat", "lat(ms)", "rtt_ms", "rtt", "avg_rtt_ms", "avg_rtt"])))
                .or_else(|| item.get("link").and_then(|link| find_first_f64(link, &["lat_ms", "latency_ms", "lat", "lat(ms)", "rtt_ms", "rtt"])))
                .or_else(|| item.get("peer").and_then(|peer| peer.get("stats")).and_then(|stats| find_first_f64(stats, &["lat_ms", "latency_ms", "lat", "lat(ms)", "rtt_ms", "rtt"]))),
            raw: item,
        })
        .collect()
}

#[tauri::command]
pub async fn query_easytier_peers(
    app: AppHandle,
    state: State<'_, AppState>,
    network_id: String,
) -> Result<Vec<ETPeerEntry>, String> {
    let rpc_port = state
        .rpc_ports
        .lock()
        .unwrap()
        .get(&network_id)
        .copied()
        .ok_or_else(|| "该网络未运行或未分配 rpc-portal".to_string())?;
    let rpc_portal = format!("127.0.0.1:{}", rpc_port);

    let output = app
        .shell()
        .sidecar("easytier-cli")
        .map_err(|error| format!("Sidecar easytier-cli 错误: {}", error))?
        .args(["-p", &rpc_portal, "-o", "json", "peer"])
        .output()
        .await
        .map_err(|error| format!("执行 easytier-cli 失败: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(format!("easytier-cli 返回失败: {}{}", stdout, stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|error| format!("解析 easytier-cli JSON 失败: {}", error))?;
    Ok(parse_peers_json(json))
}
