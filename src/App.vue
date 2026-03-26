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

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  width: 100%;
  height: 100%;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-main);
  background: #f1f5f9;
  overflow: hidden;
}

button,
input,
textarea,
select {
  font: inherit;
}

button {
  color: inherit;
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
  width: 100%;
  height: 100%;
  background: #f1f5f9;
}
</style>
