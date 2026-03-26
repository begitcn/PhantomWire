import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type ETConfig = {
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

export type NetworkProfile = {
  id: string;
  label: string;
  config: ETConfig;
};

type ETLogEvent = { network_id: string; line: string };
type ETStatusEvent = { network_id: string; running: boolean };

type AppSettings = {
  selectedProfileId: string;
  launchOnLogin: boolean;
  autoConnectOnLaunch: boolean;
  runningProfileIds: string[];
};

export type ETPeerEntry = {
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
    network_name: 'default',
    network_secret: '',
    peer_url: 'tcp://public.easytier.top:11010',
    hostname: 'default',
    use_dhcp: true,
    ipv4: '10.126.1.100',
    is_private: false,
    latency_first: false,
    magic_dns: false,
  };
}

function normalizeLabel(label: string) {
  return label.trim();
}

function resolveDisplayName(profile: Pick<NetworkProfile, 'label' | 'config'>) {
  return normalizeLabel(profile.label) || normalizeLabel(profile.config.network_name) || '未命名网络';
}

function loadNetworks(): NetworkProfile[] {
  const raw = localStorage.getItem('pw_networks');
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length) {
        return parsed.map((item) => ({
          ...item,
          label: typeof item.label === 'string' ? item.label : '',
          config: {
            ...defaultConfig(),
            ...(item.config || {}),
          },
        }));
      }
    } catch {
      // ignore broken local state
    }
  }
  return [{ id: newId(), label: 'default', config: defaultConfig() }];
}

function loadAppSettings(networks: NetworkProfile[]): AppSettings {
  const fallbackId = networks[0]?.id || newId();
  const raw = localStorage.getItem('pw_app_settings');
  if (raw) {
    try {
      const parsed = JSON.parse(raw);
      const runningProfileIds = Array.isArray(parsed.runningProfileIds)
        ? parsed.runningProfileIds.filter((id: unknown): id is string => typeof id === 'string')
        : (typeof parsed.lastRunningProfileId === 'string' && parsed.lastRunningProfileId ? [parsed.lastRunningProfileId] : []);
      return {
        selectedProfileId: typeof parsed.selectedProfileId === 'string' ? parsed.selectedProfileId : fallbackId,
        launchOnLogin: !!parsed.launchOnLogin,
        autoConnectOnLaunch: parsed.autoConnectOnLaunch === undefined ? !!parsed.launchOnLogin : !!parsed.autoConnectOnLaunch,
        runningProfileIds,
      };
    } catch {
      // ignore broken local state
    }
  }
  return {
    selectedProfileId: fallbackId,
    launchOnLogin: false,
    autoConnectOnLaunch: false,
    runningProfileIds: [],
  };
}

export function useNetworkState() {
  const networks = ref<NetworkProfile[]>(loadNetworks());
  const appSettings = ref<AppSettings>(loadAppSettings(networks.value));
  const selectedId = ref<string>(appSettings.value.selectedProfileId || networks.value[0]?.id || newId());
  const activeTab = ref<'config' | 'peers' | 'logs'>('config');
  const runningById = reactive<Record<string, boolean>>({});
  const logsById = reactive<Record<string, string[]>>({});
  const peers = ref<ETPeerEntry[]>([]);
  const peersLoading = ref(false);
  const peersError = ref<string | null>(null);
  const expandedPeerIndex = ref<number | null>(null);
  const copiedHint = ref<string | null>(null);
  const editingId = ref<string | null>(null);
  const editingLabel = ref('');
  const isRunning = ref(false);
  const logsBox = ref<HTMLElement | null>(null);
  const launchOnLogin = ref(false);
  const autoConnectOnLaunch = ref(false);
  const runningProfileIds = ref(appSettings.value.runningProfileIds.filter((id) => networks.value.some((item) => item.id === id)));
  const settingsMessage = ref<string | null>(null);
  let peersTimer: ReturnType<typeof setInterval> | null = null;
  const unlistenFns: Array<() => void> = [];
  let autoConnectAttempted = false;

  function setMessage(message: string | null) {
    settingsMessage.value = message;
  }

  function selectedProfile(): NetworkProfile {
    let profile = networks.value.find((item) => item.id === selectedId.value);
    if (!profile) {
      const created: NetworkProfile = { id: selectedId.value, label: defaultConfig().network_name, config: defaultConfig() };
      networks.value.push(created);
      profile = created;
    }
    return profile;
  }

  function displayNameOf(profile: NetworkProfile) {
    return resolveDisplayName(profile);
  }

  function rememberRunningNetwork(networkId: string) {
    if (!runningProfileIds.value.includes(networkId)) {
      runningProfileIds.value = [...runningProfileIds.value, networkId];
    }
  }

  function forgetRunningNetwork(networkId: string) {
    if (runningProfileIds.value.includes(networkId)) {
      runningProfileIds.value = runningProfileIds.value.filter((id) => id !== networkId);
    }
  }

  const selectedDisplayName = computed(() => displayNameOf(selectedProfile()));

  function scrollLogsToBottom() {
    nextTick(() => {
      const box = logsBox.value;
      if (box) {
        box.scrollTop = box.scrollHeight;
      }
    });
  }

  function startEdit(id: string) {
    const profile = networks.value.find((item) => item.id === id);
    editingId.value = id;
    editingLabel.value = profile?.label || '';
  }

  function cancelEdit() {
    editingId.value = null;
    editingLabel.value = '';
  }

  function finishEdit() {
    if (!editingId.value) return;
    const profile = networks.value.find((item) => item.id === editingId.value);
    if (profile) {
      profile.label = normalizeLabel(editingLabel.value);
    }
    cancelEdit();
  }

  function isLocalPeer(peer: ETPeerEntry): boolean {
    return (peer.hostname || '').trim().toLowerCase() === 'local';
  }

  const displayPeers = computed(() => {
    const list = [...peers.value];
    list.sort((left, right) => {
      const leftLocal = isLocalPeer(left);
      const rightLocal = isLocalPeer(right);
      if (leftLocal !== rightLocal) {
        return leftLocal ? -1 : 1;
      }
      const leftLatency = typeof left.latency_ms === 'number' && Number.isFinite(left.latency_ms)
        ? left.latency_ms
        : Number.POSITIVE_INFINITY;
      const rightLatency = typeof right.latency_ms === 'number' && Number.isFinite(right.latency_ms)
        ? right.latency_ms
        : Number.POSITIVE_INFINITY;
      if (leftLatency !== rightLatency) {
        return leftLatency - rightLatency;
      }
      return (left.hostname || '').localeCompare(right.hostname || '');
    });
    return list;
  });

  async function refreshPeers() {
    if (!isRunning.value) {
      peers.value = [];
      peersError.value = null;
      return;
    }
    peersLoading.value = true;
    peersError.value = null;
    try {
      const result = await invoke<ETPeerEntry[]>('query_easytier_peers', { networkId: selectedId.value });
      peers.value = Array.isArray(result) ? result : [];
    } catch (error) {
      peersError.value = String(error);
      peers.value = [];
    } finally {
      peersLoading.value = false;
    }
  }

  function togglePeerDetails(index: number) {
    expandedPeerIndex.value = expandedPeerIndex.value === index ? null : index;
  }

  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedHint.value = '已复制';
    } catch {
      copiedHint.value = '失败';
    }
    setTimeout(() => {
      copiedHint.value = null;
    }, 1200);
  }

  function addNetwork() {
    const id = newId();
    networks.value.push({ id, label: '新网络', config: defaultConfig() });
    selectedId.value = id;
  }

  function removeNetwork(id: string) {
    if (runningById[id]) {
      return;
    }
    const index = networks.value.findIndex((item) => item.id === id);
    if (index >= 0) {
      networks.value.splice(index, 1);
      forgetRunningNetwork(id);
      if (selectedId.value === id) {
        selectedId.value = networks.value[0]?.id || '';
      }
    }
  }

  async function startNetwork(networkId: string) {
    const profile = networks.value.find((item) => item.id === networkId);
    if (!profile) {
      throw new Error('未找到对应的网络配置');
    }
    logsById[networkId] = ['>>> 正在尝试建立幻影连接...'];
    const result = await invoke<string>('start_easytier_core', { networkId, config: profile.config });
    logsById[networkId].push(`>>> 后端反馈: ${result}`);
    runningById[networkId] = true;
    rememberRunningNetwork(networkId);
    if (networkId === selectedId.value) {
      isRunning.value = true;
    }
  }

  async function stopNetwork(networkId: string) {
    await invoke('stop_easytier_core', { networkId });
    runningById[networkId] = false;
    forgetRunningNetwork(networkId);
    if (networkId === selectedId.value) {
      isRunning.value = false;
    }
    if (!logsById[networkId]) {
      logsById[networkId] = [];
    }
    logsById[networkId].push('>>> 服务已手动停止');
  }

  async function toggleSelectedNetwork() {
    const networkId = selectedId.value;
    if (!networkId) return;
    setMessage(null);
    try {
      if (isRunning.value) {
        await stopNetwork(networkId);
      } else {
        await startNetwork(networkId);
      }
    } catch (error) {
      if (!logsById[networkId]) {
        logsById[networkId] = [];
      }
      logsById[networkId].push(`>>> 操作失败: ${error}`);
      setMessage(String(error));
    }
  }

  function getLatencyColor(ms: number | null | undefined) {
    if (ms === null || ms === undefined) return 'text-gray-400';
    if (ms < 50) return 'text-emerald-500';
    if (ms < 150) return 'text-amber-500';
    return 'text-rose-500';
  }

  watch(networks, (value) => {
    localStorage.setItem('pw_networks', JSON.stringify(value));
  }, { deep: true });

  watch([selectedId, launchOnLogin, autoConnectOnLaunch, runningProfileIds], () => {
    appSettings.value = {
      selectedProfileId: selectedId.value,
      launchOnLogin: launchOnLogin.value,
      autoConnectOnLaunch: autoConnectOnLaunch.value,
      runningProfileIds: runningProfileIds.value,
    };
    localStorage.setItem('pw_app_settings', JSON.stringify(appSettings.value));
  }, { deep: true });

  watch(selectedId, () => {
    isRunning.value = !!runningById[selectedId.value];
    scrollLogsToBottom();
  });

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

  onMounted(async () => {
    unlistenFns.push(await listen<ETLogEvent>('et-log', (event) => {
      const { network_id, line } = event.payload;
      if (!logsById[network_id]) {
        logsById[network_id] = [];
      }
      logsById[network_id].push(line);
      if (network_id === selectedId.value) {
        scrollLogsToBottom();
      }
    }));

    unlistenFns.push(await listen<ETStatusEvent>('et-status', (event) => {
      const { network_id, running } = event.payload;
      runningById[network_id] = running;
      if (running) {
        rememberRunningNetwork(network_id);
      } else {
        forgetRunningNetwork(network_id);
      }
      if (network_id === selectedId.value) {
        isRunning.value = running;
      }
      scrollLogsToBottom();
    }));

    launchOnLogin.value = await invoke<boolean>('get_launch_on_login_status');
    const launchedFromAutostart = await invoke<boolean>('was_launched_from_autostart');
    autoConnectOnLaunch.value = appSettings.value.autoConnectOnLaunch || launchOnLogin.value;

    const targetNetworkIds = runningProfileIds.value.filter((id) => networks.value.some((item) => item.id === id));
    const fallbackNetworkId = selectedId.value && !targetNetworkIds.length ? selectedId.value : '';
    const networkIdsToRestore = targetNetworkIds.length ? targetNetworkIds : (fallbackNetworkId ? [fallbackNetworkId] : []);

    if (launchedFromAutostart && !autoConnectAttempted && autoConnectOnLaunch.value && networkIdsToRestore.length) {
      autoConnectAttempted = true;
      for (const networkId of networkIdsToRestore) {
        if (runningById[networkId]) {
          continue;
        }
        selectedId.value = networkId;
        await startNetwork(networkId);
      }
    }
  });

  onBeforeUnmount(() => {
    if (peersTimer) {
      clearInterval(peersTimer);
      peersTimer = null;
    }
    unlistenFns.splice(0).forEach((fn) => fn());
  });

  return {
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
    launchOnLogin,
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
    toggleSelectedNetwork,
    togglePeerDetails,
  };
}
