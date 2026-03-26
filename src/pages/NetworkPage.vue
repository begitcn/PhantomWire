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
  expandedPeerIndex,
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
              v-model="editingLabel"
              class="name-input"
              @blur="finishEdit"
              @keyup.enter="finishEdit"
              @keyup.esc="cancelEdit"
              v-focus
            />
            <span v-else class="name" @dblclick="startEdit(n.id)">{{ displayNameOf(n) }}</span>
          </div>

          <button
            v-if="networks.length > 1"
            class="delete-btn"
            @click.stop="removeNetwork(n.id)"
            :disabled="!!runningById[n.id]"
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
  </aside>

  <main class="main-container">
    <header class="content-header">
      <div class="header-left">
        <h1>{{ selectedDisplayName }}</h1>
        <div class="status-badge" :class="{ running: isRunning }">
          <CheckCircle2 v-if="isRunning" :size="14" />
          <XCircle v-else :size="14" />
          {{ isRunning ? '在线' : '离线' }}
        </div>
      </div>

      <nav class="content-tabs">
        <button :class="{ active: activeTab === 'config' }" @click="activeTab = 'config'">
          <Settings :size="16" />
          配置
        </button>
        <button :class="{ active: activeTab === 'peers' }" @click="activeTab = 'peers'">
          <Users :size="16" />
          节点
        </button>
        <button :class="{ active: activeTab === 'logs' }" @click="activeTab = 'logs'">
          <Activity :size="16" />
          日志
        </button>
      </nav>
    </header>

    <div class="content-body">
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
                <button :class="{ active: selectedProfile().config.use_dhcp }" @click="selectedProfile().config.use_dhcp = true" :disabled="isRunning">DHCP</button>
                <button :class="{ active: !selectedProfile().config.use_dhcp }" @click="selectedProfile().config.use_dhcp = false" :disabled="isRunning">静态 IP</button>
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

        <div class="config-bottom">
          <div class="connect-actions">
            <button
              @click="toggleSelectedNetwork"
              class="connect-btn"
              :class="{ 'btn-stop': isRunning }"
            >
              <Zap v-if="!isRunning" :size="18" />
              <XCircle v-else :size="18" />
              {{ isRunning ? '断开幻影网络' : '开启幻影连接' }}
            </button>
            <p v-if="settingsMessage" class="inline-warning">{{ settingsMessage }}</p>
          </div>
        </div>
      </div>

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
                <ChevronRight :size="18" class="expand-icon" :class="{ rotated: expandedPeerIndex === idx }" />
              </div>
            </div>

            <div v-if="expandedPeerIndex === idx" class="peer-details-grid">
              <div class="detail-item"><label>网段</label><span>{{ p.cidr || '-' }}</span></div>
              <div class="detail-item"><label>隧道</label><span>{{ p.tunnel_proto || '-' }}</span></div>
              <div class="detail-item"><label>NAT 类型</label><span>{{ p.nat_type || '-' }}</span></div>
              <div class="detail-item"><label>丢包率</label><span>{{ p.loss_rate || '0%' }}</span></div>
              <div class="detail-item"><label>接收流量</label><span>{{ p.rx_bytes || '0 B' }}</span></div>
              <div class="detail-item"><label>发送流量</label><span>{{ p.tx_bytes || '0 B' }}</span></div>
              <div class="detail-item full"><label>版本信息</label><span>{{ p.version || '-' }}</span></div>
            </div>
          </div>
        </div>
      </div>

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
</template>
