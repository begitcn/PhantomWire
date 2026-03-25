<script setup lang="ts">
import { ref, onMounted, reactive, watch, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { 
  Plus, 
  Trash2, 
  Settings, 
  Network, 
  Users, 
  Activity, 
  Copy, 
  RefreshCw, 
  Shield, 
  Zap, 
  Globe,
  Cpu,
  Info,
  CheckCircle2,
  XCircle,
  AlertCircle
} from "lucide-vue-next";

type ETConfig = {
  network_name: string;
  network_secret: string;
  peer_url: string;
  hostname: string;
  use_dhcp: boolean;
  ipv4: string;
  is_private: boolean;
  latency_first: boolean;
  magic_dns: boolean;
};

type NetworkProfile = {
  id: string;
  label: string;
  config: ETConfig;
};

type ETLogEvent = { network_id: string; line: string };
type ETStatusEvent = { network_id: string; running: boolean };

type ETPeerEntry = {
  hostname?: string | null;
  ipv4?: string | null;
  ipv6?: string | null;
  cost?: string | null;
  latency_ms?: number | null;
  loss_rate?: string | null;
  rx_bytes?: string | null;
  tx_bytes?: string | null;
  tunnel_proto?: string | null;
  nat_type?: string | null;
  version?: string | null;
  cidr?: string | null;
  raw: any;
};

function newId() {
  return `${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

function defaultConfig(): ETConfig {
  return {
    network_name: "default",
    network_secret: "",
    peer_url: "tcp://public.easytier.top:11010",
    hostname: "default",
    use_dhcp: true,
    ipv4: "10.126.1.100",
    is_private: false,
    latency_first: false,
    magic_dns: false,
  };
}

function loadNetworks(): NetworkProfile[] {
  const raw = localStorage.getItem("pw_networks");
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length) {
        return parsed.map(n => ({
          ...n,
          label: n.label || n.config.network_name
        }));
      }
    } catch { /* ignore */ }
  }

  const migrated: ETConfig = {
    network_name: localStorage.getItem("et_name") || "default",
    network_secret: localStorage.getItem("et_secret") || "",
    peer_url: localStorage.getItem("et_peer") || "tcp://public.easytier.top:11010",
    hostname: localStorage.getItem("et_host") || "default",
    use_dhcp: localStorage.getItem("et_dhcp") !== "false",
    ipv4: localStorage.getItem("et_ipv4") || "10.126.1.100",
    is_private: localStorage.getItem("et_priv") === "true",
    latency_first: localStorage.getItem("et_lat") === "true",
    magic_dns: localStorage.getItem("et_dns") === "true",
  };

  return [{ id: newId(), label: migrated.network_name, config: migrated }];
}

const networks = ref<NetworkProfile[]>(loadNetworks());
const selectedId = ref<string>(networks.value[0]?.id || newId());
const activeTab = ref<'config' | 'peers' | 'logs'>('config');

function selectedProfile(): NetworkProfile {
  let p = networks.value.find(n => n.id === selectedId.value);
  if (!p) {
    const created: NetworkProfile = { id: selectedId.value, label: defaultConfig().network_name, config: defaultConfig() };
    networks.value.push(created);
    p = created;
  }
  return p;
}

const runningById = reactive<Record<string, boolean>>({});
const logsById = reactive<Record<string, string[]>>({});
const peers = ref<ETPeerEntry[]>([]);
const peersLoading = ref(false);
const peersError = ref<string | null>(null);
let peersTimer: any = null;
const expandedPeerIndex = ref<number | null>(null);
const copiedHint = ref<string | null>(null);
const editingId = ref<string | null>(null);

function startEdit(id: string) {
  editingId.value = id;
}

function finishEdit() {
  editingId.value = null;
}

function isLocalPeer(p: ETPeerEntry): boolean {
  const h = (p.hostname || "").trim().toLowerCase();
  return h === "local";
}

const displayPeers = computed(() => {
  const list = [...peers.value];
  list.sort((a, b) => {
    const al = isLocalPeer(a);
    const bl = isLocalPeer(b);
    if (al !== bl) return al ? -1 : 1;

    const la = typeof a.latency_ms === "number" && Number.isFinite(a.latency_ms) ? a.latency_ms : Number.POSITIVE_INFINITY;
    const lb = typeof b.latency_ms === "number" && Number.isFinite(b.latency_ms) ? b.latency_ms : Number.POSITIVE_INFINITY;
    if (la !== lb) return la - lb;

    const ha = (a.hostname || "").toString();
    const hb = (b.hostname || "").toString();
    return ha.localeCompare(hb);
  });
  return list;
});

const isRunning = ref(false);
const logsBox = ref<HTMLElement | null>(null);

function scrollLogsToBottom() {
  nextTick(() => {
    const box = logsBox.value;
    if (box) box.scrollTop = box.scrollHeight;
  });
}

watch(networks, (nv) => {
  localStorage.setItem("pw_networks", JSON.stringify(nv));
}, { deep: true });

watch(selectedId, () => {
  isRunning.value = !!runningById[selectedId.value];
  scrollLogsToBottom();
});

onMounted(async () => {
  await listen<ETLogEvent>("et-log", (e) => {
    const { network_id, line } = e.payload;
    if (!logsById[network_id]) logsById[network_id] = [];
    logsById[network_id].push(line);
    if (network_id === selectedId.value) scrollLogsToBottom();
  });

  await listen<ETStatusEvent>("et-status", (e) => {
    const { network_id, running } = e.payload;
    runningById[network_id] = !!running;
    if (network_id === selectedId.value) isRunning.value = !!running;
    scrollLogsToBottom();
  });

  refreshPeers();
});

async function refreshPeers() {
  if (!isRunning.value && activeTab.value === 'peers') return;
  peersError.value = null;
  peersLoading.value = true;
  try {
    const nid = selectedId.value;
    const res = await invoke<ETPeerEntry[]>("query_easytier_peers", { networkId: nid });
    peers.value = Array.isArray(res) ? res : [];
  } catch (e: any) {
    peersError.value = `${e}`;
    peers.value = [];
  } finally {
    peersLoading.value = false;
  }
}

function togglePeerDetails(idx: number) {
  expandedPeerIndex.value = expandedPeerIndex.value === idx ? null : idx;
}

async function copyText(text: string) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }

    copiedHint.value = `已复制`;
    setTimeout(() => {
      copiedHint.value = null;
    }, 1200);
  } catch (e) {
    copiedHint.value = "失败";
    setTimeout(() => {
      copiedHint.value = null;
    }, 1200);
  }
}

watch([selectedId, isRunning, activeTab], () => {
  if (peersTimer) {
    clearInterval(peersTimer);
    peersTimer = null;
  }
  if (isRunning.value && activeTab.value === 'peers') {
    refreshPeers();
    peersTimer = setInterval(() => {
      refreshPeers();
    }, 5000);
  }
});

function addNetwork() {
  const id = newId();
  networks.value.push({ id, label: "新网络", config: defaultConfig() });
  selectedId.value = id;
}

function removeNetwork(id: string) {
  if (runningById[id]) return;
  const idx = networks.value.findIndex(n => n.id === id);
  if (idx >= 0) {
    networks.value.splice(idx, 1);
    if (selectedId.value === id) {
      selectedId.value = networks.value[0]?.id || "";
    }
  }
}

async function toggle() {
  const profile = selectedProfile();
  const nid = profile.id;
  const cfg = profile.config;

  if (isRunning.value) {
    await invoke("stop_easytier_core", { networkId: nid });
    isRunning.value = false;
    if (!logsById[nid]) logsById[nid] = [];
    logsById[nid].push(">>> 服务已手动停止");
  } else {
    try {
      logsById[nid] = [">>> 正在尝试建立幻影连接..."];
      const res = await invoke("start_easytier_core", { networkId: nid, config: cfg });
      logsById[nid].push(`>>> 后端反馈: ${res}`);
      isRunning.value = true;
    } catch (e) {
      if (!logsById[nid]) logsById[nid] = [];
      logsById[nid].push(`>>> 启动失败: ${e}`);
    }
  }
}

function getLatencyColor(ms: number | null | undefined) {
  if (ms === null || ms === undefined) return 'text-gray-400';
  if (ms < 50) return 'text-emerald-500';
  if (ms < 150) return 'text-amber-500';
  return 'text-rose-500';
}

const vFocus = {
  mounted: (el: HTMLElement) => el.focus()
};
</script>

<template>
  <div class="phantom-app">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <div class="logo">
          <Globe class="logo-icon" />
          <span>PHANTOM<b>WIRE</b></span>
        </div>
      </div>

      <div class="sidebar-content">
        <div class="section-label">虚拟网络</div>
        <div class="network-list">
          <div
            v-for="n in networks"
            :key="n.id"
            class="network-item"
            :class="{ active: n.id === selectedId, running: !!runningById[n.id] }"
            @click="selectedId = n.id"
          >
            <div class="network-info">
              <Activity v-if="!!runningById[n.id]" class="status-icon pulse" />
              <Network v-else class="status-icon" />
              
              <input
                v-if="editingId === n.id"
                v-model="n.label"
                class="name-input"
                @blur="finishEdit"
                @keyup.enter="finishEdit"
                v-focus
              />
              <span v-else class="name" @dblclick="startEdit(n.id)">{{ n.label }}</span>
            </div>
            
            <button 
              class="delete-btn" 
              @click.stop="removeNetwork(n.id)" 
              :disabled="!!runningById[n.id]"
              v-if="networks.length > 1"
            >
              <Trash2 :size="14" />
            </button>
          </div>
        </div>

        <button class="add-network-btn" @click="addNetwork">
          <Plus :size="16" />
          <span>新增虚拟网络</span>
        </button>
      </div>

      <div class="sidebar-footer">
        <div class="app-info">
          <Info :size="14" />
          <span>v0.1.0</span>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-container">
      <header class="content-header">
        <div class="header-left">
          <h1>{{ selectedProfile().label }}</h1>
          <div class="status-badge" :class="{ running: isRunning }">
            <CheckCircle2 v-if="isRunning" :size="14" />
            <XCircle v-else :size="14" />
            {{ isRunning ? '在线' : '离线' }}
          </div>
        </div>

        <nav class="content-tabs">
          <button 
            :class="{ active: activeTab === 'config' }" 
            @click="activeTab = 'config'"
          >
            <Settings :size="16" />
            配置
          </button>
          <button 
            :class="{ active: activeTab === 'peers' }" 
            @click="activeTab = 'peers'"
          >
            <Users :size="16" />
            节点
          </button>
          <button 
            :class="{ active: activeTab === 'logs' }" 
            @click="activeTab = 'logs'"
          >
            <Activity :size="16" />
            日志
          </button>
        </nav>
      </header>

      <div class="content-body">
        <!-- Config Tab -->
        <div v-if="activeTab === 'config'" class="tab-pane config-pane">
          <section class="config-section">
            <h3>基础设置</h3>
            <div class="config-grid">
              <div class="form-group">
                <label>网络名称</label>
                <div class="input-wrapper">
                  <input v-model="selectedProfile().config.network_name" :disabled="isRunning" placeholder="例如: my-net" />
                </div>
              </div>
              <div class="form-group">
                <label>访问密码</label>
                <div class="input-wrapper">
                  <input v-model="selectedProfile().config.network_secret" type="password" :disabled="isRunning" placeholder="留空则不加密" />
                </div>
              </div>
              <div class="form-group full">
                <label>节点服务器 (Peer URL)</label>
                <div class="input-wrapper">
                  <Globe class="field-icon" :size="16" />
                  <input v-model="selectedProfile().config.peer_url" :disabled="isRunning" placeholder="tcp://host:port" />
                </div>
              </div>
              <div class="form-group full">
                <label>设备主机名</label>
                <div class="input-wrapper">
                  <Cpu class="field-icon" :size="16" />
                  <input v-model="selectedProfile().config.hostname" :disabled="isRunning" placeholder="默认使用系统主机名" />
                </div>
              </div>
            </div>
          </section>

          <section class="config-section">
            <h3>网络配置</h3>
            <div class="config-grid">
              <div class="form-group">
                <label>IP 分配模式</label>
                <div class="toggle-group">
                  <button 
                    :class="{ active: selectedProfile().config.use_dhcp }" 
                    @click="selectedProfile().config.use_dhcp = true"
                    :disabled="isRunning"
                  >DHCP</button>
                  <button 
                    :class="{ active: !selectedProfile().config.use_dhcp }" 
                    @click="selectedProfile().config.use_dhcp = false"
                    :disabled="isRunning"
                  >静态 IP</button>
                </div>
              </div>
              <div class="form-group" v-if="!selectedProfile().config.use_dhcp">
                <label>IPv4 地址</label>
                <div class="input-wrapper">
                  <input v-model="selectedProfile().config.ipv4" :disabled="isRunning" placeholder="10.126.x.x" />
                </div>
              </div>
            </div>
            
            <div class="switches-grid">
              <label class="switch-card" :class="{ disabled: isRunning }">
                <div class="switch-info">
                  <Shield :size="18" />
                  <div>
                    <div class="switch-title">私有模式</div>
                    <div class="switch-desc">不响应其他节点的公共发现</div>
                  </div>
                </div>
                <input type="checkbox" v-model="selectedProfile().config.is_private" :disabled="isRunning" />
              </label>

              <label class="switch-card" :class="{ disabled: isRunning }">
                <div class="switch-info">
                  <Zap :size="18" />
                  <div>
                    <div class="switch-title">延迟优先</div>
                    <div class="switch-desc">自动选择延迟最低的路径</div>
                  </div>
                </div>
                <input type="checkbox" v-model="selectedProfile().config.latency_first" :disabled="isRunning" />
              </label>

              <label class="switch-card" :class="{ disabled: isRunning }">
                <div class="switch-info">
                  <Globe :size="18" />
                  <div>
                    <div class="switch-title">魔法 DNS</div>
                    <div class="switch-desc">通过主机名访问组网内设备</div>
                  </div>
                </div>
                <input type="checkbox" v-model="selectedProfile().config.magic_dns" :disabled="isRunning" />
              </label>
            </div>
          </section>

          <div class="action-footer">
            <button 
              @click="toggle" 
              class="connect-btn" 
              :class="{ 'btn-stop': isRunning }"
            >
              <RefreshCw v-if="false" class="spin" :size="18" />
              <Zap v-else-if="!isRunning" :size="18" />
              <XCircle v-else :size="18" />
              {{ isRunning ? '断开幻影网络' : '开启幻影连接' }}
            </button>
          </div>
        </div>

        <!-- Peers Tab -->
        <div v-if="activeTab === 'peers'" class="tab-pane peers-pane">
          <div class="pane-header">
            <div class="pane-stats" v-if="peers.length">
              共 <b>{{ peers.length }}</b> 个活跃节点
            </div>
            <div class="pane-actions">
              <span v-if="copiedHint" class="copy-success">{{ copiedHint }}</span>
              <button class="icon-btn" @click="refreshPeers" :disabled="peersLoading || !isRunning">
                <RefreshCw :class="{ spin: peersLoading }" :size="16" />
              </button>
            </div>
          </div>

          <div v-if="!isRunning" class="empty-state">
            <AlertCircle :size="48" />
            <p>请先启动幻影网络以查看节点</p>
          </div>
          
          <div v-else-if="peersError" class="error-state">
            <AlertCircle :size="32" />
            <p>{{ peersError }}</p>
          </div>

          <div v-else-if="peers.length === 0" class="empty-state">
            <Users :size="48" />
            <p>暂无其他节点在线</p>
          </div>

          <div v-else class="peer-list">
            <div v-for="(p, idx) in displayPeers" :key="idx" class="peer-card">
              <div class="peer-main" @click="togglePeerDetails(idx)">
                <div class="peer-info">
                  <div class="peer-host">
                    <span class="host-name">{{ p.hostname || 'Unknown' }}</span>
                    <span v-if="isLocalPeer(p)" class="local-tag">本机</span>
                  </div>
                  <div class="peer-ip-wrap" @click.stop="p.ipv4 && copyText(p.ipv4)">
                    <span class="peer-ip">{{ p.ipv4 || '-' }}</span>
                    <Copy :size="12" class="copy-icon" />
                  </div>
                </div>
                
                <div class="peer-meta">
                  <div class="peer-latency" :class="getLatencyColor(p.latency_ms)">
                    {{ typeof p.latency_ms === 'number' ? `${Math.round(p.latency_ms)}ms` : '-' }}
                  </div>
                  <ChevronRight 
                    :size="18" 
                    class="expand-icon" 
                    :class="{ rotated: expandedPeerIndex === idx }" 
                  />
                </div>
              </div>

              <div v-if="expandedPeerIndex === idx" class="peer-details-grid">
                <div class="detail-item">
                  <label>网段</label>
                  <span>{{ p.cidr || '-' }}</span>
                </div>
                <div class="detail-item">
                  <label>隧道</label>
                  <span>{{ p.tunnel_proto || '-' }}</span>
                </div>
                <div class="detail-item">
                  <label>NAT 类型</label>
                  <span>{{ p.nat_type || '-' }}</span>
                </div>
                <div class="detail-item">
                  <label>丢包率</label>
                  <span>{{ p.loss_rate || '0%' }}</span>
                </div>
                <div class="detail-item">
                  <label>接收流量</label>
                  <span>{{ p.rx_bytes || '0 B' }}</span>
                </div>
                <div class="detail-item">
                  <label>发送流量</label>
                  <span>{{ p.tx_bytes || '0 B' }}</span>
                </div>
                <div class="detail-item full">
                  <label>版本信息</label>
                  <span>{{ p.version || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Logs Tab -->
        <div v-if="activeTab === 'logs'" class="tab-pane logs-pane">
          <div ref="logsBox" class="log-container">
            <div v-if="!(logsById[selectedId] && logsById[selectedId].length)" class="empty-state">
              <Activity :size="48" />
              <p>暂无运行日志</p>
            </div>
            <div v-for="(l, i) in logsById[selectedId]" :key="i" class="log-line">
              <span class="log-time">[{{ new Date().toLocaleTimeString() }}]</span>
              <span class="log-content">{{ l }}</span>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
:root {
  --sidebar-width: 260px;
  --primary: #0ea5e9;
  --primary-hover: #0284c7;
  --bg-sidebar: #f8fafc;
  --bg-main: #ffffff;
  --border-color: #e2e8f0;
  --text-main: #0f172a;
  --text-muted: #64748b;
  --danger: #ef4444;
  --success: #10b981;
}

* { box-sizing: border-box; }

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-main);
  background: var(--bg-main);
  overflow: hidden;
  user-select: none;
}

.phantom-app {
  display: flex;
  height: 100vh;
  width: 100vw;
}

/* Sidebar Styles */
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 24px 20px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  letter-spacing: 0.5px;
  color: var(--text-main);
}

.logo-icon {
  color: var(--primary);
}

.logo span b {
  color: var(--primary);
  margin-left: 2px;
}

.sidebar-content {
  flex: 1;
  padding: 0 12px;
  overflow-y: auto;
}

.section-label {
  padding: 0 12px 8px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted);
  letter-spacing: 0.05em;
}

.network-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.network-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.network-item:hover {
  background: #f1f5f9;
}

.network-item.active {
  background: #ffffff;
  border-color: var(--border-color);
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

.network-item.active .name {
  color: var(--primary);
  font-weight: 600;
}

.network-item.running {
  background: #f0fdf4;
}

.network-item.running.active {
  background: #ffffff;
  border-color: var(--success);
}

.network-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.status-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.network-item.running .status-icon {
  color: var(--success);
}

.pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

.name {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.name-input {
  background: white;
  border: 1px solid var(--primary);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 14px;
  width: 100%;
  outline: none;
}

.delete-btn {
  padding: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.network-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: #fee2e2;
  color: var(--danger);
}

.add-network-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: calc(100% - 24px);
  margin: 12px;
  padding: 10px;
  background: transparent;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.add-network-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
  background: #f0f9ff;
}

.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.app-info {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

/* Main Container Styles */
.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
}

.content-header {
  padding: 20px 32px 0;
  border-bottom: 1px solid var(--border-color);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}

.header-left h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  background: #f1f5f9;
  color: var(--text-muted);
}

.status-badge.running {
  background: #dcfce7;
  color: var(--success);
}

.content-tabs {
  display: flex;
  gap: 24px;
}

.content-tabs button {
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  padding: 8px 4px 12px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.content-tabs button:hover {
  color: var(--text-main);
}

.content-tabs button.active {
  color: var(--primary);
  border-bottom-color: var(--primary);
}

.content-body {
  flex: 1;
  overflow-y: auto;
  padding: 32px;
}

.tab-pane {
  max-width: 800px;
  margin: 0 auto;
}

/* Config Pane */
.config-section {
  margin-bottom: 32px;
}

.config-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-muted);
  margin: 0 0 16px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group.full {
  grid-column: span 2;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-main);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.field-icon {
  position: absolute;
  left: 12px;
  color: var(--text-muted);
}

.input-wrapper input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  transition: all 0.2s;
}

.input-wrapper .field-icon + input {
  padding-left: 36px;
}

.input-wrapper input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.1);
}

.input-wrapper input:disabled {
  background: #f8fafc;
  cursor: not-allowed;
}

.toggle-group {
  display: flex;
  background: #f1f5f9;
  padding: 4px;
  border-radius: 8px;
}

.toggle-group button {
  flex: 1;
  padding: 6px;
  border: none;
  background: transparent;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-group button.active {
  background: white;
  color: var(--primary);
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  font-weight: 600;
}

.switches-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-top: 24px;
}

.switch-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: #f8fafc;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.switch-card:hover:not(.disabled) {
  border-color: var(--primary);
  background: #f0f9ff;
}

.switch-card.disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.switch-info {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-muted);
}

.switch-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
}

.switch-desc {
  font-size: 11px;
}

.action-footer {
  margin-top: 40px;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: center;
}

.connect-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-width: 200px;
  padding: 14px 28px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 4px 12px rgba(14, 165, 233, 0.3);
}

.connect-btn:hover {
  background: var(--primary-hover);
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(14, 165, 233, 0.4);
}

.connect-btn.btn-stop {
  background: white;
  color: var(--danger);
  border: 1px solid #fee2e2;
  box-shadow: none;
}

.connect-btn.btn-stop:hover {
  background: #fff1f1;
  border-color: #fecaca;
}

/* Peers Pane */
.pane-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.pane-stats {
  font-size: 14px;
  color: var(--text-muted);
}

.pane-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.copy-success {
  font-size: 12px;
  color: var(--success);
  background: #ecfdf5;
  padding: 2px 8px;
  border-radius: 4px;
}

.icon-btn {
  padding: 6px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.icon-btn:hover:not(:disabled) {
  border-color: var(--primary);
  color: var(--primary);
}

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

.empty-state, .error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  color: var(--text-muted);
  text-align: center;
}

.empty-state p { margin-top: 16px; font-size: 15px; }

.peer-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.peer-card {
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: white;
  overflow: hidden;
}

.peer-main {
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s;
}

.peer-main:hover {
  background: #f8fafc;
}

.peer-host {
  display: flex;
  align-items: center;
  gap: 8px;
}

.host-name {
  font-weight: 600;
  font-size: 15px;
}

.local-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: #e0f2fe;
  color: var(--primary);
  border-radius: 4px;
  font-weight: 700;
}

.peer-ip-wrap {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: copy;
}

.copy-icon { opacity: 0; transition: opacity 0.2s; }
.peer-ip-wrap:hover .copy-icon { opacity: 1; }

.peer-meta {
  display: flex;
  align-items: center;
  gap: 16px;
}

.peer-latency {
  font-family: monospace;
  font-weight: 600;
  font-size: 14px;
}

.expand-icon {
  color: var(--text-muted);
  transition: transform 0.3s;
}

.expand-icon.rotated { transform: rotate(90deg); }

.peer-details-grid {
  padding: 16px 20px;
  background: #f8fafc;
  border-top: 1px solid var(--border-color);
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item.full { grid-column: span 3; }

.detail-item label {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 600;
}

.detail-item span {
  font-size: 13px;
  color: var(--text-main);
  word-break: break-all;
}

/* Logs Pane */
.logs-pane {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.log-container {
  flex: 1;
  background: #0f172a;
  border-radius: 12px;
  padding: 20px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  overflow-y: auto;
  color: #e2e8f0;
}

.log-line {
  margin-bottom: 4px;
  display: flex;
  gap: 12px;
  line-height: 1.5;
}

.log-time { color: #64748b; flex-shrink: 0; }
.log-content { word-break: break-all; }

/* Utilities */
.text-emerald-500 { color: #10b981; }
.text-amber-500 { color: #f59e0b; }
.text-rose-500 { color: #f43f5e; }
.text-gray-400 { color: #94a3b8; }
</style>