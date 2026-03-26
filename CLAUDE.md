# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Frontend / Tauri
- `npm install` — install JS dependencies
- `npm run tauri dev` — run the full desktop app in development mode
- `npm run build` — frontend typecheck + production web build (`vue-tsc --noEmit && vite build`)
- `npm run tauri build` — build the production desktop bundle
- `npm run preview` — preview the built frontend only

### Rust / native
- `cargo check --manifest-path src-tauri/Cargo.toml` — check the Rust/Tauri backend
- `cargo test --manifest-path src-tauri/Cargo.toml` — run Rust tests
- `cargo test --manifest-path src-tauri/Cargo.toml <test_name>` — run a single Rust test by filter

### Notes
- There is currently **no dedicated lint script** in `package.json`.
- There is currently **no frontend test setup** in this repository.
- Tauri dev expects the Vite dev server at `http://localhost:1420` and `vite.config.ts` uses a fixed port `1420` with HMR on `1421` when `TAURI_DEV_HOST` is set.

## Architecture

### Big picture
PhantomWire is a **Tauri 2 desktop app** with a **Vue 3 + TypeScript + Vite** frontend and a **Rust backend** that mainly orchestrates bundled EasyTier binaries.

The app is effectively a single-page desktop control panel for managing multiple virtual network profiles:
- frontend stores profile/config state in `localStorage`
- frontend calls Rust commands through Tauri `invoke(...)`
- Rust launches and manages EasyTier sidecar processes
- Rust emits log/status events back to the UI
- frontend listens for those events and refreshes the display

### Frontend structure
- `src/main.ts` mounts the Vue app.
- `src/App.vue` is the top-level shell and currently renders a single page.
- `src/pages/NetworkPage.vue` contains the main UI for:
  - network profile list
  - configuration form
  - peer list
  - logs view
- `src/composables/useNetworkState.ts` is the core frontend state layer. It holds most app behavior:
  - network/profile persistence
  - selected tab/profile state
  - Tauri `invoke(...)` calls
  - Tauri event listeners for logs and running state
  - peer polling while the peers tab is open

There is **no router** and **no dedicated store library**. Most frontend behavior is centralized in `useNetworkState.ts`.

### Backend / native structure
- `src-tauri/src/main.rs` is the binary entrypoint.
- `src-tauri/src/lib.rs` builds the Tauri app and handles:
  - shared `AppState`
  - system tray menu
  - hide-on-close behavior
  - autostart integration on Windows
  - command registration
- `src-tauri/src/easytier.rs` contains the EasyTier integration:
  - build command arguments for `easytier-core`
  - start/stop sidecar processes
  - reserve per-network RPC ports
  - query peers through `easytier-cli`
  - parse CLI JSON into frontend-friendly peer data
  - emit `et-log` and `et-status` events

### Runtime data flow
The important frontend/backend boundary is:

Frontend invokes:
- `start_easytier_core`
- `stop_easytier_core`
- `query_easytier_peers`
- `get_launch_on_login_status`
- `was_launched_from_autostart`

Rust emits:
- `et-log`
- `et-status`

`useNetworkState.ts` is the main place to inspect when changing UX or application behavior, because it is where UI state, persistence, backend commands, and event subscriptions come together.

### Process management model
Rust `AppState` tracks:
- running child processes by network ID
- allocated RPC ports by network ID

Each virtual network profile can map to a running EasyTier instance. Peer inspection is not implemented in Rust directly; Rust runs the bundled `easytier-cli` sidecar against that profile's local RPC portal and parses the JSON result.

### Windows-specific behavior
This repository is Windows-oriented:
- autostart is implemented through the HKCU `Run` registry key in `src-tauri/src/lib.rs`
- closing the window hides it instead of exiting
- the tray menu includes show/autostart/quit actions
- `src-tauri/build.rs` embeds `src-tauri/admin.manifest` during build
- `ensure_windows_sidecar_dlls(...)` copies `Packet.dll` and `wintun.dll` from bundled resources to the executable directory when needed

### Bundled binaries and resources
`src-tauri/tauri.conf.json` bundles these runtime dependencies:

External sidecars:
- `bin/easytier-core`
- `bin/easytier-cli`

Resources:
- `bin/Packet.dll`
- `bin/wintun.dll`

Be careful when changing Tauri bundle config or sidecar names/paths: the Rust backend assumes these sidecars/resources exist and are available at runtime.

## Repository-specific guidance
- Treat this as a **single-page desktop app**, not a multi-route web app.
- Prefer making behavioral changes in `src/composables/useNetworkState.ts` first, then adjust `src/pages/NetworkPage.vue` for presentation.
- Keep the Tauri command/event contract in sync between `useNetworkState.ts`, `src-tauri/src/lib.rs`, and `src-tauri/src/easytier.rs`.
- If changing how EasyTier is launched, check both:
  - sidecar permissions in `src-tauri/capabilities/default.json`
  - bundle declarations in `src-tauri/tauri.conf.json`
- The README is still the default Tauri/Vue template and does not describe the actual app architecture.
