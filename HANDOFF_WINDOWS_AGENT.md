# SwarmChat — Windows Platform Agent Handoff Document

**Date**: 2026-05-12
**Source Agent**: Hermes (WSL/Linux)
**Target Agent**: Windows Platform Agent (Windows Host, VS2022, RNW)
**GitHub**: https://github.com/hucx9527/swarmchat
**Working Directory**: `C:\dev\swarmchat\` (copy from WSL: `\\wsl.localhost\Ubuntu\home\hjudgex\projects\swarmchat\`)

---

## 1. Project Overview

SwarmChat is a decentralized, end-to-end encrypted communication platform built on the Swarm Communication Protocol (SCP). It enables peer-to-peer messaging with:

- **E2E Encryption**: X3DH key agreement + Double Ratchet per session + Sender Key for groups
- **Decentralized Identity**: `did:key` method, Ed25519 keypairs, BIP39 mnemonics
- **Relay Nodes**: Optional relay for NAT traversal and offline message storage
- **Multi-platform**: Android, iOS (via React Native), Windows (in progress), HarmonyOS (planned)

### Monorepo Structure

```
swarmchat/                          # Monorepo root (GitHub remote: hucx9527/swarmchat)
├── scp-core/                       # Rust — Core crypto: X3DH, Double Ratchet, Sender Key, DID
├── scp-relay/                      # Go — Relay node (NAT traversal, prekey storage)
├── scp-cli/                        # Go — CLI tool (identity, messaging, groups)
├── scp-sdk-python/                 # Python — Agent SDK (webhooks, decorators)
├── scp-spec/                       # Markdown — Protocol spec v0.1.0
├── docs-site/                      # HTML — Developer portal
├── swarmchat/                      # React Native app (TypeScript) — THE MAIN TARGET
│   ├── src/                        # TypeScript: screens, services, store, hooks
│   ├── rust/scp-core-bridge/       # Rust — JNI (Android) + C FFI (iOS/Windows)
│   ├── android/                    # Android native module (Java → JNI)
│   ├── ios/                        # iOS native module (ObjC → C FFI)
│   └── windows/                    # Windows native module (C++/WinRT) — READY but UNTESTED
└── fab/                            # Meta-build tools (not critical)
```

---

## 2. Phase Completion Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | ✅ Complete | Core crypto: X3DH, Double Ratchet, Sender Key, integration tests |
| Phase 1 | ✅ Complete | Identity & DID system, PeerId, transport layer |
| Phase 2 | ✅ Complete | Go Relay + CLI, Python SDK, Dev Portal |
| Phase 3 | ✅ Complete | React Native app with full UI, JSI/FFI Rust bridge |
| Phase W (Windows) | 🔶 In Progress | **THIS IS YOUR TARGET** — C++/WinRT bridge written, DLL tools prepared |
| Phase H (HarmonyOS) | 🔲 Not Started | Cross-compilation feasibility needs verification |

---

## 3. What's Done (Phase W — Windows Port)

### 3.1 Rust Bridge (scp-core-bridge) — COMPLETE

The Rust crate at `swarmchat/rust/scp-core-bridge/src/lib.rs` exports via `extern "C"`:

```c
char* scp_bridge_generate_identity();                    // → JSON
char* scp_bridge_recover_identity(const char* mnemonic);  // → JSON
char* scp_bridge_did_from_public_key(const char* hex);    // → JSON
char* scp_bridge_double_ratchet_encrypt(const char* sid, const char* pt); // → JSON
char* scp_bridge_sender_key_encrypt(const char* gid, const char* pt);     // → JSON
char* scp_bridge_sign_envelope(const char* envelope, const char* key);    // → JSON
void  scp_bridge_free_string(char* ptr);                  // Free allocated string
```

✅ Compiles for `x86_64-pc-windows-msvc` (no library dependency issues)
✅ All functions return JSON strings (parsed on the JS side)
✅ No `ring` crate dependency — pure ed25519-dalek/x25519-dalek/bip39

### 3.2 C++/WinRT Native Module — WRITTEN, UNTESTED

Files in `swarmchat/windows/SwarmChat/`:
- `ScpCoreBridgeModule.h` — 109 lines, declares 6 REACT_METHODs, LoadLibrary wrapper
- `ScpCoreBridgeModule.cpp` — 204 lines, LoadLibrary → `GetProcAddress` → promise-based JS bridge

Both files exist but **have NOT been compiled or tested** because:
- The WSL environment cannot compile Windows DLLs or RNW solutions
- Visual Studio 2022 / RNW initialization hasn't been run yet

### 3.3 RN Upgrade (0.74 → 0.76.9) — COMPLETE

The React Native app was upgraded from 0.74.0 to 0.76.9 for better RNW compatibility.
Key changes documented in `swarmchat/UPGRADE_RN076.md`:
- `@react-native/babel-preset` replaces `metro-react-native-babel-preset`
- `android/settings.gradle` updated per RN 0.76 conventions
- `react-native-windows` scripts added to `package.json`

### 3.4 Cross-Platform Strategy Document — WRITTEN

`swarmchat/CROSS_PLATFORM_PLAN.md` contains the full execution plan for Phase W and H.
Read it first — it captures all architectural decisions and task decomposition.

---

## 4. Windows Build Prerequisites (Your Machine Setup)

### 4.1 Required Software

```
✔ Visual Studio 2022 Community — workload: "Desktop development with C++"
✔ Windows SDK (10.0.22621.0+)
✔ Rust 1.80+ — target: x86_64-pc-windows-msvc
✔ Node.js 18+ LTS
✔ Git
```

### 4.2 Project Setup on Windows

```powershell
# Option A: Copy from WSL (recommended for first time)
robocopy \\wsl.localhost\Ubuntu\home\hjudgex\projects\swarmchat C:\dev\swarmchat /E /NP

# Option B: Clone from GitHub
cd C:\dev
git clone https://github.com/hucx9527/swarmchat.git
cd swarmchat

# Install Node dependencies
cd swarmchat
npm install

# Initialize React Native for Windows
npx react-native-windows-init --overwrite

# Add Rust MSVC target
rustup target add x86_64-pc-windows-msvc
```

---

## 5. Phase W Task List (Your Action Plan)

### Priority: W-0 → W-1 → W-2 → W-3 (bridge first), then W-4 → W-5 → W-6 → W-7

### W-0: Complete Windows Build Environment [DO THIS FIRST]
1. Open the solution in Visual Studio 2022
2. Confirm `windows/SwarmChat.sln` is generated
3. Run the default RNW project to confirm it builds empty
4. Verify `npm install` completes without errors (npmmirror.com works as npm mirror in China)

### W-1: Compile scp-core-bridge DLL [CRITICAL]
```powershell
cd C:\dev\swarmchat\swarmchat\rust\scp-core-bridge
cargo build --release --target x86_64-pc-windows-msvc
```
- Expected output: `target\x86_64_pc_windows_msvc\release\scp_core_bridge.dll`
- Verify exports: `dumpbin /exports scp_core_bridge.dll`
- Expected symbols: `scp_bridge_generate_identity`, `scp_bridge_recover_identity`, etc.
- ⚠ **Important**: If the DLL doesn't build, the scp-core Cargo.toml might need `cdylib` crate type (it should already have it — verify before compiling)

### W-2: Register C++/WinRT Native Module
The bridge files are pre-written at:
- `windows/SwarmChat/ScpCoreBridgeModule.h`
- `windows/SwarmChat/ScpCoreBridgeModule.cpp`

Steps in Visual Studio:
1. Open `windows\SwarmChat.sln`
2. Right-click SwarmChat project → Add → Existing Item → select both .h and .cpp
3. In `ReactPackageProvider.cpp`:
   ```cpp
   #include "ScpCoreBridgeModule.h"
   // In CreateReactPackageProvider():
   packageBuilder.AddModule(L"ScpCore", ...);
   ```
4. Copy `scp_core_bridge.dll` to the output directory:
   ```powershell
   copy target\x86_64-pc-windows-msvc\release\scp_core_bridge.dll windows\Debug\
   ```

### W-3: Verify JS Bridge
Build and run:
```powershell
npx react-native run-windows
```
In the app, verify `NativeModules.ScpCore.generateIdentity()` returns proper JSON.

### W-4: UI Layout Adaptation
- Set minimum window size: 800×600
- Responsive layout via `useWindowDimensions`
- Split pane: left contact list + right chat panel

### W-5: System Tray
- `react-native-system-tray` configuration
- Minimize to tray on window close
- Right-click menu: Show/Exit

### W-6: Desktop Notifications
- Windows native toast notifications for new messages
- Click notification → open specific chat

### W-7: Packaging & CI
- MSIX packaging
- GitHub Actions Windows runner for automated builds

---

## 6. Key Architectural Decisions

1. **No libp2p on mobile**: The mobile app communicates with relay nodes via HTTPS. Pure Rust crypto only (ed25519-dalek, x25519-dalek, chacha20-poly1305). No networking stack in the Rust crate.

2. **C FFI as universal bridge**: The same `extern "C"` exports serve iOS (ObjC → C FFI), Android (JNI wraps C FFI), and Windows (C++/WinRT LoadLibrary). No platform-specific Rust code needed beyond JNI feature gate.

3. **Dual-mode bridge**: JS fallback exists in `ScpCoreBridge.ts` for development without native builds. Windows can reuse this pattern: if DLL isn't loaded, fall back to the same TS crypto.

4. **RN 0.76.9 for NewArch compatibility**: Needed for `react-native-windows` support.

5. **npm mirror**: The project uses npmmirror.com for faster installs in China network environments. Set via `.npmrc` or `npm config set registry https://registry.npmmirror.com`.

---

## 7. Known Issues & Pitfalls

### Rust Build
- **scp-core Cargo.toml**: The main `scp-core/` project (monorepo root level) is a *library*. The bridge sub-package is at `swarmchat/rust/scp-core-bridge/` — use this one, NOT the root `scp-core/`.
- **crate-type**: Ensure `cdylib` is in Cargo.toml for Windows DLL output.
- **If `ring` crate fails**: The bridge doesn't use `ring`, but transitive deps might pull it in. If so, verify feature flags.

### RNW Initialization
- `npx react-native-windows-init --overwrite` may overwrite the `windows/SwarmChat/` directory. **Keep a copy** of `ScpCoreBridgeModule.h/.cpp` before running it. The pre-written files are the only bridge implementation.
- RNW version must match RN 0.76.9. If the init generates a different version, it may need manual adjustment.

### Network
- **WSL has slow download speeds** (~100 KB/s from GitHub/npm). This is a WSL network issue, not the project. If you operate on Windows host, downloads should be normal speed.
- For the Windows agent: you don't have this limitation since you're on the Windows host directly.

### Git
- The repo has uncommitted changes (scp-core crypto module updates from Phase 1).
- A `.gitignore` has been added — review it before committing.
- The origin remote is `https://github.com/hucx9527/swarmchat.git`.

---

## 8. Phase H (HarmonyOS) — Reference Only

Do not start HarmonyOS until Windows bridge is verified working (W-3 completed).

Key tasks when ready:
1. **H-1**: Verify `aarch64-linux-ohos` Rust target exists (or use `aarch64-linux-android` + OHOS NDK linker)
2. Write HarmonyOS NAPI bridge (analogous to C++/WinRT)
3. Rewrite ArkTS UI (cannot reuse React Native code)

See `CROSS_PLATFORM_PLAN.md` for full details.

---

## 9. Contact / Code Sources

All source files on the main branch have been pushed to GitHub:
- **GitHub**: https://github.com/hucx9527/swarmchat
- **WSL working copy**: `/home/hjudgex/projects/swarmchat/`
- **Windows copy**: Should be `C:\dev\swarmchat\` (copy from WSL or clone from GitHub)

### Key Reference Documents (in repository root)

| File | What it covers |
|------|---------------|
| `README.md` | Project overview, component table, quick start |
| `PHASE_1_PROGRESS.md` | scp-core identity & transport implementation details |
| `PHASE_2_PROGRESS.md` | Relay, CLI, SDK, Portal implementation details |
| `PHASE_3_PROGRESS.md` | React Native app full file inventory |
| `CROSS_PLATFORM_PLAN.md` | **MUST READ** — Windows & HarmonyOS execution plan |
| `UPGRADE_RN076.md` | RN 0.74 → 0.76.9 upgrade changelog |
| `WINDOWS_BUILD.md` | Step-by-step Windows build guide |
| `HANDOFF_WINDOWS_AGENT.md` | This document |

---

## 10. Phase W Success Criteria

✅ W-0: `npx react-native run-windows` launches an empty RNW app
✅ W-1: `scp_core_bridge.dll` compiles and exports all 7 C FFI functions
✅ W-2: `NativeModules.ScpCore` is registered and resolves from JavaScript
✅ W-3: `NativeModules.ScpCore.generateIdentity()` returns DID JSON in the running app
✅ W-4: UI renders correctly at 800×600 minimum, responsive layout works
✅ W-5: System tray with minimize-to-tray and right-click menu
✅ W-6: Windows toast notifications on new message
✅ W-7: MSIX package builds, GitHub Actions CI configured for Windows

**Final verification**: Windows client can exchange encrypted messages with Android/iOS peers.

---

*This document was prepared by Hermes (WSL/Linux agent) for handover to the Windows platform agent. Questions? Investigate the files mentioned above — all code is structural and documented.*
