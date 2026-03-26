<script setup lang="ts">
import NetworkPage from './pages/NetworkPage.vue';
</script>

<template>
  <div class="phantom-app">
    <NetworkPage />
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
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-main);
  background: var(--bg-main);
  overflow: hidden;
  user-select: none;
}

button,
input,
textarea,
select {
  font: inherit;
}

input,
textarea {
  width: 100%;
}

pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.phantom-app {
  display: flex;
  height: 100vh;
  width: 100vw;
  background: #f1f5f9;
}

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
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
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

.content-tabs button,
.icon-btn {
  background: transparent;
  cursor: pointer;
}

.content-tabs button {
  border: none;
  border-bottom: 2px solid transparent;
  padding: 8px 4px 12px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-muted);
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
  max-width: 1040px;
  margin: 0 auto;
}

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
  gap: 4px;
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
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
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

.config-bottom {
  margin-top: 40px;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.connect-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.connect-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-width: 220px;
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

.connect-btn:disabled,
.connect-btn.disabled {
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
.pane-actions {
  display: flex;
  align-items: center;
}

.pane-header {
  justify-content: space-between;
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
  padding: 2px 8px;
  border-radius: 4px;
}

.icon-btn {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-muted);
  padding: 6px;
}

.icon-btn:hover:not(:disabled) {
  border-color: var(--primary);
  color: var(--primary);
}

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

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
  color: #e2e8f0;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  overflow-y: auto;
}

.log-line {
  margin-bottom: 4px;
  display: flex;
  gap: 12px;
  line-height: 1.5;
}

.log-time { color: #64748b; flex-shrink: 0; }
.log-content { word-break: break-all; }

.text-emerald-500 { color: #10b981; }
.text-amber-500 { color: #f59e0b; }
.text-rose-500 { color: #f43f5e; }
.text-gray-400 { color: #94a3b8; }

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
</style>
