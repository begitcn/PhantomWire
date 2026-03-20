<script setup lang="ts">
import { ref, onMounted, reactive, watch, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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
        // 兼容旧数据：如果缺少 label，则使用 network_name
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

    copiedHint.value = `已复制 ${text}`;
    setTimeout(() => {
      copiedHint.value = null;
    }, 1200);
  } catch (e) {
    copiedHint.value = "复制失败";
    setTimeout(() => {
      copiedHint.value = null;
    }, 1200);
  }
}

watch([selectedId, isRunning], () => {
  if (peersTimer) {
    clearInterval(peersTimer);
    peersTimer = null;
  }
  refreshPeers();
  if (isRunning.value) {
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
  if (idx >= 0) networks.value.splice(idx, 1);
  if (selectedId.value === id) selectedId.value = networks.value[0]?.id || "";
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
const vFocus = {
  mounted: (el: HTMLElement) => el.focus()
};
</script>

<template>
  <div class="phantom-ui">
    <header>
      <div class="logo">PHANTOM<span>WIRE</span></div>
      <div class="status" :class="{ running: isRunning }">
        {{ isRunning ? '已接入' : '离线' }}
      </div>
    </header>

    <div class="networks">
      <div class="networks-head">
        <div class="networks-title">虚拟网络</div>
        <button class="net-btn" @click="addNetwork">新增</button>
      </div>
      <div class="net-list">
        <div
          v-for="n in networks"
          :key="n.id"
          class="net-item"
          :class="{ active: n.id === selectedId, running: !!runningById[n.id] }"
          @click="selectedId = n.id"
        >
          <input
            v-if="editingId === n.id"
            v-model="n.label"
            class="net-edit-input"
            @blur="finishEdit"
            @keyup.enter="finishEdit"
            v-focus
          />
          <span v-else class="net-name" @dblclick="startEdit(n.id)">{{ n.label }}</span>
          <button class="net-del" @click.stop="removeNetwork(n.id)" :disabled="!!runningById[n.id]">×</button>
        </div>
      </div>
    </div>

    <div class="form">
      <div class="row">
        <div class="cell">
          <label>网络名称 (Easytier Name)</label>
          <input v-model="selectedProfile().config.network_name" :disabled="isRunning" />
        </div>
        <div class="cell">
          <label>网络密码</label>
          <input v-model="selectedProfile().config.network_secret" type="password" :disabled="isRunning" />
        </div>
      </div>

      <div class="row">
        <div class="cell">
          <label>IP 模式</label>
          <button @click="selectedProfile().config.use_dhcp = !selectedProfile().config.use_dhcp" class="mode-btn" :disabled="isRunning">
            {{ selectedProfile().config.use_dhcp ? 'DHCP (自动获取)' : '静态 IPv4' }}
          </button>
        </div>
        <div class="cell">
          <label>指定 IP</label>
          <input v-model="selectedProfile().config.ipv4" :disabled="isRunning || selectedProfile().config.use_dhcp" placeholder="10.x.x.x" />
        </div>
      </div>

      <div class="row full">
        <label>节点服务器 (Peer URL)</label>
        <input v-model="selectedProfile().config.peer_url" :disabled="isRunning" />
      </div>

      <div class="row full">
        <label>设备主机名</label>
        <input v-model="selectedProfile().config.hostname" :disabled="isRunning" />
      </div>

      <div class="switches">
        <div class="sw-item" v-for="tag in ['is_private', 'latency_first', 'magic_dns']" :key="tag">
          <input type="checkbox" v-model="(selectedProfile().config as any)[tag]" :disabled="isRunning" :id="tag" />
          <label :for="tag">{{ ({ is_private: '启用私有模式', latency_first: '开启延迟优先模式', magic_dns: '启用魔法DNS' } as any)[tag] }}</label>
        </div>
      </div>

      <button @click="toggle" class="main-btn" :class="{ 'btn-stop': isRunning }">
        {{ isRunning ? '断开幻影网络' : '开启幻影连接' }}
      </button>
    </div>

    <!-- <div class="console-wrap">
      <div class="console-toolbar">
        <button class="console-btn" @click="clearLogs">清空日志</button>
      </div>
      <div ref="logsBox" class="console">
        <div v-for="(l, i) in currentLogs()" :key="i" class="line">{{ l }}</div>
      </div>
    </div> -->

    <div class="peers">
      <div class="peers-head">
        <div class="peers-title">组网节点</div>
        <div class="peers-actions">
          <div v-if="copiedHint" class="copy-hint">{{ copiedHint }}</div>
          <button class="peer-btn" @click="refreshPeers" :disabled="peersLoading">{{ peersLoading ? '刷新中' : '刷新' }}</button>
        </div>
      </div>

      <div v-if="peersError" class="peer-error">{{ peersError }}</div>
      <div class="peer-table">
        <div class="peer-row peer-row-head">
          <div>主机名</div>
          <div>IPv4</div>
          <div>连接方式</div>
          <div>延迟</div>
        </div>

        <div v-if="!peersLoading && peers.length === 0" class="peer-empty">暂无节点</div>
        <div v-for="(p, idx) in displayPeers" :key="`${p.hostname || ''}_${p.ipv4 || ''}_${idx}`" class="peer-item">
          <div class="peer-row peer-row-main" :class="{ 'peer-row-local': isLocalPeer(p) }" @click="togglePeerDetails(idx)">
            <div class="peer-host">{{ p.hostname || '-' }}</div>
            <div class="peer-ip">
              <span
                class="peer-ip-text"
                :class="{ 'peer-ip-clickable': !!p.ipv4 }"
                @click.stop="p.ipv4 && copyText(p.ipv4)"
              >
                {{ p.ipv4 || '-' }}
              </span>
            </div>
            <div class="peer-cost">{{ p.cost || '-' }}</div>
            <div class="peer-lat">{{ typeof p.latency_ms === 'number' ? `${Math.round(p.latency_ms)}ms` : '-' }}</div>
          </div>

          <div v-if="expandedPeerIndex === idx" class="peer-details">
            <div class="peer-detail"><span>网段</span><span>{{ p.cidr || '-' }}</span></div>
            <div class="peer-detail"><span>隧道</span><span>{{ p.tunnel_proto || '-' }}</span></div>
            <div class="peer-detail"><span>NAT</span><span>{{ p.nat_type || '-' }}</span></div>
            <div class="peer-detail"><span>丢包</span><span>{{ p.loss_rate || '-' }}</span></div>
            <div class="peer-detail"><span>接收</span><span>{{ p.rx_bytes || '-' }}</span></div>
            <div class="peer-detail"><span>发送</span><span>{{ p.tx_bytes || '-' }}</span></div>
            <div class="peer-detail"><span>版本</span><span>{{ p.version || '-' }}</span></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
:root { color-scheme: light; }
body { background: #f6f8fa; color: #111827; font-family: 'Segoe UI', system-ui, sans-serif; margin: 0; }
.phantom-ui { height: 100vh; display: flex; flex-direction: column; padding: 20px; box-sizing: border-box; gap: 15px; }
header { display: flex; justify-content: space-between; align-items: center; }
.logo { font-size: 1.5rem; font-weight: 900; letter-spacing: 2px; }
.logo span { color: #00d2ff; }
.status { font-size: 12px; padding: 4px 12px; border-radius: 20px; background: #ffffff; color: #6b7280; border: 1px solid #e5e7eb; }
.status.running { color: #16a34a; background: #dcfce7; border: 1px solid #86efac; }

.form { display: flex; flex-direction: column; gap: 12px; }
.networks { display: flex; flex-direction: column; gap: 8px; }
.networks-head { display: flex; justify-content: space-between; align-items: center; }
.networks-title { font-weight: 700; font-size: 12px; color: #374151; }
.net-btn { background: #ffffff; border: 1px solid #e5e7eb; padding: 6px 10px; border-radius: 8px; color: #111827; cursor: pointer; font-size: 12px; }
.net-btn:hover { border-color: #00d2ff; box-shadow: 0 0 0 2px #00d2ff22; }
.net-list { display: flex; gap: 8px; overflow-x: auto; padding-bottom: 2px;padding-top: 2px; }
.net-item { display: flex; align-items: center; gap: 10px; background: #ffffff; border: 1px solid #e5e7eb; padding: 10px 12px; border-radius: 12px; cursor: pointer; min-width: 140px; justify-content: space-between; position: relative; transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.net-item:hover { border-color: #00d2ff; background: #f0f9ff; }
.net-item.active { border-color: #00d2ff; background: linear-gradient(135deg, #ffffff 0%, #f0f9ff 100%); box-shadow: 0 4px 12px rgba(0, 210, 255, 0.15); transform: translateY(-1px); }
.net-item.active .net-name { color: #0088cc; font-weight: 800; }
.net-item.running { border-color: #16a34a; border-width: 1.5px; }
.net-item.running.active { border-color: #00d2ff; box-shadow: 0 4px 12px rgba(22, 163, 74, 0.1), 0 0 0 1px #00d2ff; }
.net-edit-input { border: 1px solid #00d2ff; background: #ffffff; padding: 2px 6px; border-radius: 4px; font-size: 13px; width: 85%; outline: none; box-shadow: 0 0 0 2px rgba(0, 210, 255, 0.1); }
.net-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; color: #4b5563; font-weight: 500; }
.net-del { background: transparent; border: none; color: #9ca3af; cursor: pointer; font-size: 16px; line-height: 1; padding: 0 2px; }
.net-del:disabled { opacity: 0.4; cursor: not-allowed; }
.row { display: flex; gap: 12px; }
.cell { flex: 1; display: flex; flex-direction: column; gap: 4px; }
.full { flex-direction: column; gap: 4px; }
label { font-size: 11px; color: #6b7280; font-weight: bold; text-align: left;}
input { background: #ffffff; border: 1px solid #e5e7eb; padding: 10px; border-radius: 6px; color: #111827; outline: none; transition: 0.2s; }
input:focus { border-color: #00d2ff; box-shadow: 0 0 0 2px #00d2ff22; }
input:disabled { opacity: 0.5; cursor: not-allowed; }
.mode-btn { background: #ffffff; border: 1px solid #e5e7eb; padding: 10px; border-radius: 6px; color: #111827; cursor: pointer; }
.mode-btn:disabled { opacity: 0.5; }

.switches { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; }
.sw-item { display: flex; align-items: center; gap: 6px; font-size: 10px; }

.main-btn { background: #00d2ff; color: #0b1220; border: none; padding: 12px; border-radius: 6px; font-weight: bold; cursor: pointer; transition: 0.2s; margin-top: 5px; }
.main-btn:hover { transform: translateY(-1px); box-shadow: 0 4px 12px #00d2ff44; }
.main-btn.btn-stop { background: #fee2e2; color: #dc2626; border: 1px solid #fecaca; }

.console-wrap { flex: 1; display: flex; flex-direction: column; min-height: 0; }
.console-toolbar { display: flex; justify-content: flex-end; margin-bottom: 8px; }
.console-btn { background: #ffffff; border: 1px solid #e5e7eb; padding: 6px 10px; border-radius: 8px; color: #111827; cursor: pointer; font-size: 12px; }
.console-btn:hover { border-color: #00d2ff; box-shadow: 0 0 0 2px #00d2ff22; }

.console { flex: 1; background: #ffffff; border: 1px solid #e5e7eb; border-radius: 6px; padding: 10px; font-family: 'Consolas', monospace; font-size: 11px; overflow-y: auto; color: #374151; text-align: left;}
.line { border-bottom: 1px solid #f3f4f6; padding: 2px 0; word-break: break-all; }

.peers { background: #ffffff; border: 1px solid #e5e7eb; border-radius: 10px; padding: 12px; }
.peers-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.peers-title { font-weight: 700; font-size: 12px; color: #374151; }
.peers-actions { display: flex; align-items: center; gap: 10px; }
.copy-hint { font-size: 12px; color: #16a34a; background: #dcfce7; border: 1px solid #86efac; padding: 4px 8px; border-radius: 999px; }
.peer-btn { background: #ffffff; border: 1px solid #e5e7eb; padding: 6px 10px; border-radius: 8px; color: #111827; cursor: pointer; font-size: 12px; }
.peer-btn:disabled { opacity: 0.6; cursor: not-allowed; }
.peer-btn:hover { border-color: #00d2ff; box-shadow: 0 0 0 2px #00d2ff22; }

.peer-error { color: #dc2626; font-size: 12px; margin-bottom: 8px; word-break: break-all; }
.peer-table { display: flex; flex-direction: column; gap: 6px; }
.peer-item { display: flex; flex-direction: column; gap: 6px; }
.peer-row { display: grid; grid-template-columns: 1fr 1fr 1fr 90px; gap: 10px; padding: 8px 10px; border: 1px solid #f3f4f6; border-radius: 8px; font-size: 12px; color: #111827; }
.peer-row-local { background: #eff6ff; border-color: #93c5fd; }
.peer-row-main { cursor: pointer; }
.peer-row-main:hover { border-color: #00d2ff; box-shadow: 0 0 0 2px #00d2ff22; }
.peer-row-head { background: #f9fafb; font-weight: 700; color: #374151; }
.peer-empty { font-size: 12px; color: #6b7280; padding: 8px 2px; }
.peer-host, .peer-cost, .peer-lat { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.peer-ip { display: flex; align-items: center; min-width: 0; }
.peer-ip-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.peer-ip-clickable { color: #0284c7; font-weight: 700; cursor: pointer; }
.peer-ip-clickable:hover { text-decoration: underline; }

.peer-details { border: 1px solid #f3f4f6; border-radius: 8px; padding: 10px; display: grid; grid-template-columns: 1fr 1fr; gap: 8px 12px; font-size: 12px; color: #374151; }
.peer-detail { display: flex; justify-content: space-between; gap: 10px; }
.peer-detail span:first-child { color: #6b7280; }
.peer-detail span:last-child { color: #111827; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>