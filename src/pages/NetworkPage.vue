<script setup lang="ts">
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
  CheckCircle2,
  XCircle,
  AlertCircle,
  ChevronRight,
} from 'lucide-vue-next';
import { useNetworkState } from '../composables/useNetworkState';

const {
  activeTab,
  addNetwork,
  cancelEdit,
  copiedHint,
  copyText,
  displayNameOf,
  displayPeers,
  editingId,
  editingLabel,
  expandedPeerKey,
  finishEdit,
  getLatencyColor,
  isLocalPeer,
  isRunning,
  logsBox,
  logsById,
  networks,
  peers,
  peersError,
  peersLoading,
  refreshPeers,
  removeNetwork,
  runningById,
  selectedDisplayName,
  selectedId,
  selectedProfile,
  settingsMessage,
  startEdit,
  togglePeerDetails,
  toggleSelectedNetwork,
} = useNetworkState();

const vFocus = {
  mounted: (el: HTMLElement) => el.focus(),
};

function peerDetailsId(peerKey: string) {
  return `peer-details-${encodeURIComponent(peerKey)}`;
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <span>PHANTOM<b>WIRE</b></span>
      </div>
    </div>

    <div class="sidebar-content">
      <div class="section-label">虚拟网络</div>
      <div class="network-list" role="list">
        <div
          v-for="n in networks"
          :key="n.id"
          class="network-item"
          :class="{ active: n.id === selectedId, running: !!runningById[n.id] }"
          role="listitem"
        >
          <input
            v-if="editingId === n.id"
            v-model="editingLabel"
            class="name-input name-input-row"
            :aria-label="`重命名 ${displayNameOf(n)}`"
            @blur="finishEdit"
            @keyup.enter="finishEdit"
            @keyup.esc="cancelEdit"
            v-focus
          />

          <button
            v-else
            type="button"
            class="network-select"
            :aria-pressed="n.id === selectedId"
            :aria-label="`选择网络 ${displayNameOf(n)}`"
            @click="selectedId = n.id"
          >
            <div class="network-info">
              <Activity v-if="!!runningById[n.id]" class="status-icon pulse" />
              <Network v-else class="status-icon" />
              <span class="name" @dblclick.stop="startEdit(n.id)">{{ displayNameOf(n) }}</span>
            </div>
          </button>

          <button
            v-if="networks.length > 1"
            type="button"
            class="delete-btn"
            :aria-label="`删除网络 ${displayNameOf(n)}`"
            @click.stop="removeNetwork(n.id)"
            :disabled="!!runningById[n.id]"
          >
            <Trash2 :size="14" />
          </button>
        </div>
      </div>

      <button type="button" class="add-network-btn" @click="addNetwork">
        <Plus :size="16" />
        <span>新增虚拟网络</span>
      </button>
    </div>
  </aside>

  <main class="main-container">
    <header class="content-header">
      <div class="header-left">
        <div>
          <h1>{{ selectedDisplayName }}</h1>
          <p class="header-subtitle">EasyTier 虚拟网络控制面板</p>
        </div>
        <div class="status-badge" :class="{ running: isRunning }">
          <CheckCircle2 v-if="isRunning" :size="14" />
          <XCircle v-else :size="14" />
          {{ isRunning ? '在线' : '离线' }}
        </div>
      </div>

      <nav class="content-tabs" role="tablist" aria-label="网络详情视图">
        <button
          id="tab-config"
          type="button"
          role="tab"
          :class="{ active: activeTab === 'config' }"
          :aria-selected="activeTab === 'config'"
          aria-controls="panel-config"
          :tabindex="activeTab === 'config' ? 0 : -1"
          @click="activeTab = 'config'"
        >
          <Settings :size="16" />
          配置
        </button>
        <button
          id="tab-peers"
          type="button"
          role="tab"
          :class="{ active: activeTab === 'peers' }"
          :aria-selected="activeTab === 'peers'"
          aria-controls="panel-peers"
          :tabindex="activeTab === 'peers' ? 0 : -1"
          @click="activeTab = 'peers'"
        >
          <Users :size="16" />
          节点
        </button>
        <button
          id="tab-logs"
          type="button"
          role="tab"
          :class="{ active: activeTab === 'logs' }"
          :aria-selected="activeTab === 'logs'"
          aria-controls="panel-logs"
          :tabindex="activeTab === 'logs' ? 0 : -1"
          @click="activeTab = 'logs'"
        >
          <Activity :size="16" />
          日志
        </button>
      </nav>
    </header>

    <div class="content-body">
      <section
        v-if="activeTab === 'config'"
        id="panel-config"
        class="tab-pane config-pane"
        role="tabpanel"
        aria-labelledby="tab-config"
      >
        <section class="config-section card-section">
          <div class="section-heading">
            <h3>基础设置</h3>
          </div>
          <div class="config-grid">
            <div class="form-group">
              <label :for="`network-name-${selectedId}`">网络名称</label>
              <div class="input-wrapper">
                <input
                  :id="`network-name-${selectedId}`"
                  v-model="selectedProfile().config.network_name"
                  :disabled="isRunning"
                  placeholder="例如: my-net"
                />
              </div>
            </div>
            <div class="form-group">
              <label :for="`network-secret-${selectedId}`">访问密码</label>
              <div class="input-wrapper">
                <input
                  :id="`network-secret-${selectedId}`"
                  v-model="selectedProfile().config.network_secret"
                  type="password"
                  :disabled="isRunning"
                  placeholder="留空则不加密"
                />
              </div>
            </div>
            <div class="form-group full">
              <label :for="`peer-url-${selectedId}`">节点服务器 (Peer URL)</label>
              <div class="input-wrapper">
                <Globe class="field-icon" :size="16" />
                <input
                  :id="`peer-url-${selectedId}`"
                  v-model="selectedProfile().config.peer_url"
                  :disabled="isRunning"
                  placeholder="tcp://host:port"
                />
              </div>
            </div>
            <div class="form-group full">
              <label :for="`hostname-${selectedId}`">设备主机名</label>
              <div class="input-wrapper">
                <Cpu class="field-icon" :size="16" />
                <input
                  :id="`hostname-${selectedId}`"
                  v-model="selectedProfile().config.hostname"
                  :disabled="isRunning"
                  placeholder="默认使用系统主机名"
                />
              </div>
            </div>
          </div>
        </section>

        <section class="config-section card-section">
          <div class="section-heading">
            <h3>网络配置</h3>
          </div>
          <div class="config-grid">
            <div class="form-group">
              <label>IP 分配模式</label>
              <div class="toggle-group">
                <button
                  type="button"
                  :class="{ active: selectedProfile().config.use_dhcp }"
                  @click="selectedProfile().config.use_dhcp = true"
                  :disabled="isRunning"
                >
                  DHCP
                </button>
                <button
                  type="button"
                  :class="{ active: !selectedProfile().config.use_dhcp }"
                  @click="selectedProfile().config.use_dhcp = false"
                  :disabled="isRunning"
                >
                  静态 IP
                </button>
              </div>
            </div>
            <div v-if="!selectedProfile().config.use_dhcp" class="form-group">
              <label :for="`ipv4-${selectedId}`">IPv4 地址</label>
              <div class="input-wrapper">
                <input
                  :id="`ipv4-${selectedId}`"
                  v-model="selectedProfile().config.ipv4"
                  :disabled="isRunning"
                  placeholder="10.126.x.x"
                />
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

        <div class="config-bottom">
          <div class="connect-actions">
            <button
              type="button"
              @click="toggleSelectedNetwork"
              class="connect-btn"
              :class="{ 'btn-stop': isRunning }"
            >
              <Zap v-if="!isRunning" :size="18" />
              <XCircle v-else :size="18" />
              {{ isRunning ? '断开幻影网络' : '开启幻影连接' }}
            </button>
            <p v-if="settingsMessage" class="inline-warning" role="status">{{ settingsMessage }}</p>
          </div>
        </div>
      </section>

      <section
        v-if="activeTab === 'peers'"
        id="panel-peers"
        class="tab-pane peers-pane"
        role="tabpanel"
        aria-labelledby="tab-peers"
      >
        <div class="pane-header">
          <div class="pane-stats" v-if="displayPeers.length">
            共 <b>{{ displayPeers.length }}</b> 个活跃节点
          </div>
          <div class="pane-actions">
            <span v-if="copiedHint" class="copy-success" role="status">{{ copiedHint }}</span>
            <button
              type="button"
              class="icon-btn"
              aria-label="刷新节点列表"
              @click="refreshPeers"
              :disabled="peersLoading || !isRunning"
            >
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
          <article v-for="p in displayPeers" :key="p.peerKey" class="peer-card">
            <div class="peer-header">
              <button
                type="button"
                class="peer-main"
                :aria-expanded="expandedPeerKey === p.peerKey"
                :aria-controls="peerDetailsId(p.peerKey)"
                @click="togglePeerDetails(p.peerKey)"
              >
                <div class="peer-info">
                  <div class="peer-host">
                    <span class="host-name">{{ p.hostname || 'Unknown' }}</span>
                    <span v-if="isLocalPeer(p)" class="local-tag">本机</span>
                  </div>
                  <span class="peer-ip">{{ p.ipv4 || '-' }}</span>
                </div>

                <div class="peer-meta">
                  <div class="peer-latency" :class="getLatencyColor(p.latency_ms)">
                    {{ typeof p.latency_ms === 'number' ? `${Math.round(p.latency_ms)}ms` : '-' }}
                  </div>
                  <ChevronRight :size="18" class="expand-icon" :class="{ rotated: expandedPeerKey === p.peerKey }" />
                </div>
              </button>

              <button
                type="button"
                class="peer-copy-btn"
                :aria-label="p.ipv4 ? `复制 ${p.hostname || '节点'} 的 IPv4 地址` : '无可复制的 IPv4 地址'"
                :disabled="!p.ipv4"
                @click="p.ipv4 && copyText(p.ipv4)"
              >
                <Copy :size="14" />
              </button>
            </div>

            <div v-if="expandedPeerKey === p.peerKey" :id="peerDetailsId(p.peerKey)" class="peer-details-grid">
              <div class="detail-item"><label>网段</label><span>{{ p.cidr || '-' }}</span></div>
              <div class="detail-item"><label>隧道</label><span>{{ p.tunnel_proto || '-' }}</span></div>
              <div class="detail-item"><label>NAT 类型</label><span>{{ p.nat_type || '-' }}</span></div>
              <div class="detail-item"><label>丢包率</label><span>{{ p.loss_rate || '0%' }}</span></div>
              <div class="detail-item"><label>接收流量</label><span>{{ p.rx_bytes || '0 B' }}</span></div>
              <div class="detail-item"><label>发送流量</label><span>{{ p.tx_bytes || '0 B' }}</span></div>
              <div class="detail-item full"><label>版本信息</label><span>{{ p.version || '-' }}</span></div>
            </div>
          </article>
        </div>
      </section>

      <section
        v-if="activeTab === 'logs'"
        id="panel-logs"
        class="tab-pane logs-pane"
        role="tabpanel"
        aria-labelledby="tab-logs"
      >
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
      </section>
    </div>
  </main>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 24px 20px 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  letter-spacing: 0.5px;
  color: var(--text-main);
}

.logo span b {
  color: var(--primary);
  margin-left: 2px;
}

.sidebar-content {
  flex: 1;
  padding: 0 12px 12px;
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
  gap: 6px;
}

.network-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.network-select,
.name-input-row {
  width: 100%;
}

.network-select {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s, box-shadow 0.2s, color 0.2s;
}

.network-select:hover {
  background: #f1f5f9;
}

.network-item.active .network-select {
  background: #ffffff;
  border-color: var(--border-color);
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06);
}

.network-item.running .network-select {
  background: #f0fdf4;
}

.network-item.running.active .network-select {
  background: #ffffff;
  border-color: var(--success);
}

.network-item.active .name {
  color: var(--primary);
  font-weight: 600;
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
  min-width: 0;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.name-input {
  background: white;
  border: 1px solid var(--primary);
  border-radius: 10px;
  padding: 11px 12px;
  font-size: 14px;
  outline: none;
  box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.12);
}

.delete-btn {
  flex-shrink: 0;
  padding: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 8px;
  opacity: 0;
  transition: opacity 0.2s, background 0.2s, color 0.2s, border-color 0.2s;
}

.network-item:hover .delete-btn,
.network-item:focus-within .delete-btn {
  opacity: 1;
}

.delete-btn:hover:not(:disabled) {
  background: #fee2e2;
  border-color: #fecaca;
  color: var(--danger);
}

.delete-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.add-network-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: calc(100% - 24px);
  margin: 12px;
  padding: 12px;
  background: transparent;
  border: 1px dashed var(--border-color);
  border-radius: 10px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s, background 0.2s;
}

.add-network-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
  background: #f0f9ff;
}

.main-container {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
}

.content-header {
  padding: 24px 32px 0;
  border-bottom: 1px solid var(--border-color);
  background: rgba(255, 255, 255, 0.92);
}

.header-left {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.header-left h1 {
  margin: 0;
  font-size: 26px;
  font-weight: 700;
}

.header-subtitle {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-muted);
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  background: #f1f5f9;
  color: var(--text-muted);
  white-space: nowrap;
}

.status-badge.running {
  background: #dcfce7;
  color: var(--success);
}

.content-tabs {
  display: flex;
  gap: 24px;
}

.content-tabs button,
.icon-btn,
.peer-copy-btn {
  background: transparent;
  color: inherit;
}

.content-tabs button {
  border: none;
  border-bottom: 2px solid transparent;
  padding: 8px 4px 12px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: color 0.2s, border-color 0.2s;
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
  max-width: 1040px;
  margin: 0 auto;
}

.card-section {
  background: #ffffff;
  border: 1px solid var(--border-color);
  border-radius: 18px;
  padding: 24px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.04);
}

.config-section {
  margin-bottom: 24px;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.config-section h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
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
  padding: 11px 12px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  font-size: 14px;
  transition: border-color 0.2s, box-shadow 0.2s, background 0.2s;
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
  border-radius: 10px;
  gap: 4px;
}

.toggle-group button {
  flex: 1;
  padding: 8px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s, color 0.2s, box-shadow 0.2s;
}

.toggle-group button.active {
  background: white;
  color: var(--primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  font-weight: 600;
}

.toggle-group button:disabled {
  cursor: not-allowed;
  opacity: 0.65;
}

.switches-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
  margin-top: 24px;
}

.switch-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px;
  background: #f8fafc;
  border: 1px solid var(--border-color);
  border-radius: 14px;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.switch-card:hover:not(.disabled),
.switch-card:focus-within:not(.disabled) {
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
  margin-top: 2px;
  font-size: 11px;
}

.config-bottom {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.connect-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.connect-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-width: 220px;
  padding: 14px 28px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 999px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s, transform 0.2s, box-shadow 0.2s, border-color 0.2s, color 0.2s;
  box-shadow: 0 4px 12px rgba(14, 165, 233, 0.3);
}

.connect-btn:hover:not(:disabled) {
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

.connect-btn.btn-stop:hover:not(:disabled) {
  background: #fff1f1;
  border-color: #fecaca;
}

.connect-btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  transform: none;
  box-shadow: none;
}

.inline-warning {
  margin: 12px 0 0;
  font-size: 13px;
  color: #b45309;
}

.pane-header,
.pane-actions,
.peer-header,
.peer-meta {
  display: flex;
  align-items: center;
}

.pane-header {
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
}

.pane-actions {
  gap: 12px;
}

.pane-stats {
  font-size: 14px;
  color: var(--text-muted);
}

.copy-success {
  font-size: 12px;
  color: var(--success);
  background: #ecfdf5;
  padding: 4px 8px;
  border-radius: 999px;
}

.icon-btn,
.peer-copy-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-muted);
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s, background 0.2s;
}

.icon-btn {
  padding: 8px;
}

.icon-btn:hover:not(:disabled),
.peer-copy-btn:hover:not(:disabled) {
  border-color: var(--primary);
  color: var(--primary);
  background: #f0f9ff;
}

.icon-btn:disabled,
.peer-copy-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  color: var(--text-muted);
  text-align: center;
}

.empty-state p,
.error-state p {
  margin-top: 16px;
  font-size: 15px;
}

.peer-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.peer-card {
  border: 1px solid var(--border-color);
  border-radius: 16px;
  background: white;
  overflow: hidden;
}

.peer-header {
  gap: 12px;
  padding: 10px 12px 10px 16px;
}

.peer-main {
  flex: 1;
  min-width: 0;
  padding: 6px 4px 6px 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: color 0.2s;
}

.peer-main:hover .host-name {
  color: var(--primary);
}

.peer-info {
  min-width: 0;
}

.peer-host {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.host-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
  font-size: 15px;
}

.local-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: #e0f2fe;
  color: var(--primary);
  border-radius: 999px;
  font-weight: 700;
  flex-shrink: 0;
}

.peer-ip {
  display: inline-block;
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 13px;
  user-select: text;
}

.peer-meta {
  flex-shrink: 0;
  gap: 14px;
}

.peer-latency {
  font-family: monospace;
  font-weight: 600;
  font-size: 14px;
}

.expand-icon {
  color: var(--text-muted);
  transition: transform 0.2s;
}

.expand-icon.rotated {
  transform: rotate(90deg);
}

.peer-copy-btn {
  flex-shrink: 0;
  padding: 8px;
}

.peer-details-grid {
  padding: 16px 20px;
  background: #f8fafc;
  border-top: 1px solid var(--border-color);
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.detail-item.full {
  grid-column: span 3;
}

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
  user-select: text;
}

.logs-pane {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.log-container {
  flex: 1;
  background: #0f172a;
  border-radius: 16px;
  padding: 20px;
  color: #e2e8f0;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  overflow-y: auto;
}

.log-line {
  margin-bottom: 4px;
  display: flex;
  gap: 12px;
  line-height: 1.55;
}

.log-time {
  color: #64748b;
  flex-shrink: 0;
}

.log-content {
  word-break: break-all;
  user-select: text;
}

.text-emerald-500 { color: #10b981; }
.text-amber-500 { color: #f59e0b; }
.text-rose-500 { color: #f43f5e; }
.text-gray-400 { color: #94a3b8; }

.network-select:focus-visible,
.delete-btn:focus-visible,
.add-network-btn:focus-visible,
.content-tabs button:focus-visible,
.toggle-group button:focus-visible,
.connect-btn:focus-visible,
.icon-btn:focus-visible,
.peer-main:focus-visible,
.peer-copy-btn:focus-visible,
.switch-card input:focus-visible,
.name-input:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 2px;
}

@media (max-width: 1200px) {
  .config-grid,
  .peer-details-grid {
    grid-template-columns: 1fr;
  }

  .form-group.full,
  .detail-item.full {
    grid-column: span 1;
  }
}

@media (max-width: 980px) {
  .sidebar {
    width: 224px;
  }

  .content-header {
    padding: 20px 24px 0;
  }

  .content-body {
    padding: 24px;
  }

  .header-left {
    flex-direction: column;
    align-items: flex-start;
  }

  .content-tabs {
    gap: 18px;
  }
}

@media (max-width: 760px) {
  .sidebar {
    width: 208px;
  }

  .content-header {
    padding: 18px 20px 0;
  }

  .content-body {
    padding: 20px;
  }

  .card-section {
    padding: 20px;
  }

  .peer-header {
    align-items: flex-start;
  }

  .peer-main {
    align-items: flex-start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .pulse,
  .spin {
    animation: none;
  }

  .network-select,
  .delete-btn,
  .add-network-btn,
  .content-tabs button,
  .toggle-group button,
  .switch-card,
  .connect-btn,
  .icon-btn,
  .peer-main,
  .peer-copy-btn,
  .expand-icon,
  .host-name,
  .input-wrapper input {
    transition: none;
  }

  .connect-btn:hover:not(:disabled) {
    transform: none;
  }
}
</style>
