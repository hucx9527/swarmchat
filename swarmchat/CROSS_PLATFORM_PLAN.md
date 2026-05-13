# SwarmChat 跨平台扩展开发计划书（修订版）

**文档版本**：v1.2（Hermes 战略修订）
**适用对象**：AI 编程助手（本文件即执行脚本）
**前提**：Android APK 已编译通过，iOS 桥接代码就位
**基线**：`rust/scp-core-bridge` 单体 crate，C ABI + JNI 双模式

### 人类决策记录
- ✅ Windows 编译环境：Windows 宿主机（WSL 宿主机，/mnt/c/ 互通）
- ✅ RN 版本：从 0.74.0 升级至 0.76+（为了更好的 RNW 兼容性和 NewArch 支持）
- ✅ 执行顺序：先 Phase W（Windows 桥接层），再 Phase H（HarmonyOS）

---

## 一、核心架构决策（修订要点 vs 原版）

| 原版假设 | 实际情况 | 修订策略 |
|---|---|---|
| Rust crate 名 `scp-core`，有 workspace | 单体 `scp-core-bridge`，无 workspace | **保持单体**，Windows/HarmonyOS 加 feature gate |
| Windows 用 JSI/TurboModule | iOS 已用 C FFI + RCTBridgeModule | **Windows 复用同一套 C FFI**，通过 C++/WinRT 原生模块暴露 |
| HarmonyOS 用 N-API (Node.js) | HarmonyOS 有独立的 Native API (NAPI) | **用 HarmonyOS NAPI**，底层仍是 C FFI |
| libp2p 有编译风险 | 实际无 libp2p，纯加密库（ed25519-dalek, x25519-dalek, bip39） | **风险归零**，无网络栈依赖 |
| 开发者在 Windows + VS2022 | 开发者在 WSL/Linux | **Windows 编译需 Windows 宿主机或 CI runner** |

### 统一桥接架构

```
                    rust/scp-core-bridge/src/lib.rs
                    ┌──────────────────────────────┐
                    │  // 平台无关的核心逻辑         │
                    │  generate_identity()          │
                    │  recover_identity()           │
                    │  double_ratchet_encrypt()     │
                    │  ...                          │
                    ├──────────────────────────────┤
                    │  extern "C" 导出 (所有平台)    │
                    │  scp_bridge_generate_identity │
                    │  scp_bridge_free_string       │
                    ├──────────────────────────────┤
                    │  cfg(feature="android") {     │
                    │    JNI: Java_com_swarmchat_*  │
                    │  }                            │
                    └──────────┬──────────┬────────┘
                               │          │
                    ┌──────────▼──┐  ┌────▼──────────┐
                    │  Windows    │  │  HarmonyOS    │
                    │  LoadLibrary│  │  dlopen/NAPI  │
                    │  + C++/WinRT│  │  + ArkTS 封装 │
                    └─────────────┘  └───────────────┘
```

---

## 二、Phase W：Windows 桌面客户端（修订版）

### 2.1 依赖关系

```
Windows 宿主（真机或 CI）
  ├── Visual Studio 2022 Build Tools (MSVC v143, Windows SDK)
  ├── Rust 1.80+ (x86_64-pc-windows-msvc target)
  └── Node.js 18+ (npx react-native-windows-init)
```

**⚠ 关键约束**：RNW 编译必须在 Windows 上进行。WSL/Linux 只能写代码、不能编译。

### 2.2 Cargo.toml 修改方案

```toml
[lib]
name = "scp_core_bridge"
crate-type = ["cdylib", "staticlib"]

[features]
default = []
android = ["jni"]
# Windows 无需额外 feature——extern "C" 已是默认导出
```

Windows 复用 `extern "C"` 导出，无需修改 Rust 代码。

### 2.3 任务分解

| 编号 | 任务 | 详细步骤 | 验收标准 |
|---|---|---|---|
| **W-0** | 搭建 Windows 编译环境 | 1. Windows 宿主机安装 VS 2022 Build Tools（C++工作负载+Windows SDK）<br>2. `rustup target add x86_64-pc-windows-msvc`<br>3. 克隆仓库到 Windows<br>4. `npm install && npx react-native-windows-init --overwrite`<br>5. 确认 `windows/` 目录生成 | `windows/` 目录存在，含 `.sln` 文件 |
| **W-1** | 编译 DLL | 1. Windows 上 `cargo build --release --target x86_64-pc-windows-msvc --features default`<br>2. 验证 `target/x86_64-pc-windows-msvc/release/scp_core_bridge.dll` 生成<br>3. 编写测试 .exe 调用 `scp_bridge_generate_identity()`<br>4. 确认返回值正确（JSON 含 mnemonic/did/peerId） | DLL 可被 C++ 程序加载调用 |
| **W-2** | C++/WinRT 原生模块 | 1. 在 `windows/` 下创建 `ScpCoreBridgeModule.h/.cpp`<br>2. `LoadLibrary(L"scp_core_bridge.dll")` + `GetProcAddress` 绑定 6 个函数指针<br>3. 实现 `REACT_METHOD` 导出给 JS<br>4. 注册到 `ReactPackageProvider` | `NativeModules.ScpCore` 在 JS 中可调用 |
| **W-3** | 验证 JS 桥接 | 1. 运行 `npx react-native run-windows`<br>2. JS 调用 `NativeModules.ScpCore.generateIdentity()`<br>3. 打印返回的 DID | JS 控制台输出 DID 字符串 |
| **W-4** | UI 布局适配 | 1. 设置窗口最小 800x600<br>2. `useWindowDimensions` 响应式布局<br>3. 分栏布局：左侧联系人列表 + 右侧聊天面板 | 缩放窗口 UI 不崩溃 |
| **W-5** | 系统托盘 | 1. `react-native-system-tray` 配置<br>2. 托盘图标、右键菜单（显示/退出）<br>3. 关闭窗口→隐藏到托盘 | 点击 X 不退出，托盘可见 |
| **W-6** | 桌面通知 | 1. Windows 原生通知（`react-native-windows` 自带 `LottieNotification` 或自定义）<br>2. 新消息触发 toast<br>3. 点击跳转 | 收到消息弹出通知 |
| **W-7** | 打包与 CI | 1. MSIX 打包配置<br>2. GitHub Actions Windows runner 自动构建 | `.msix` 产出 |

**Phase W 完成标志**：Windows 客户端可与 Android/iOS 互发加密消息。

---

## 三、Phase H：HarmonyOS 6.0（修订版）

### 3.1 关键差异

| 方面 | 原版计划 | 修订 |
|---|---|---|
| Rust target | `aarch64-linux-ohos` | **需确认** — 华为未官方支持。备选：`aarch64-linux-android` + 静态链接 |
| N-API | 误以为是 Node.js N-API | 使用 HarmonyOS Native API (`napi.h`) |
| ArkTS UI | 从零重写 | 同意，必须重写（鸿蒙不兼容 RN） |
| 网络栈 | 担心 libp2p 编译 | 无 libp2p，无风险 |

### 3.2 H-1：交叉编译可行性验证（风险前置）

这部分决定整个 Phase H 的生死。先做 H-1，通过才继续。

```bash
# 步骤 1：配置 OHOS NDK 链接器
# 下载 OHOS NDK → 解压，获取交叉编译工具链路径
# 写入 .cargo/config.toml

# 步骤 2：编译最小测试库
cargo new --lib ohos-test
# 写一个简单 extern "C" fn hello() -> *const c_char
cargo build --target aarch64-linux-android  # 先用 Android target 验证

# 步骤 3：在鸿蒙模拟器中加载
# 用 DevEco Studio 创建空白工程
# 通过 NAPI 加载 .so 并调用 hello()
```

**难点**：
- `aarch64-linux-ohos` 不是 Rust 官方 target。可能方案：
  a. 华为官方 Rust 分支（如有）
  b. 用 `aarch64-linux-android` target + `-C linker=` 指向 OHOS NDK 的 clang
  c. 如都不行，用 `aarch64-unknown-linux-gnu` + musl
- `ring` crate 在非标准 target 上可能编译失败（依赖汇编代码）
  - 当前 `scp-core-bridge` 不依赖 `ring`，所以大概率跳过此坑

### 3.3 任务分解（H-1 通过后执行）

| 编号 | 任务 | 说明 | 预估 |
|---|---|---|---|
| **H-1** | 交叉编译可行性验证 | 见 3.2 | 2天 |
| **H-2** | 编译 scp-core-bridge 为 OHOS .so | 配置完成后一次编译 | 1天 |
| **H-3** | NAPI 桥接层 | C++ 桥接 napi.h + scp_bridge_* C FFI | 2天 |
| **H-4** | ArkTS UI 框架 | 登录/注册/主页路由 | 2天 |
| **H-5** | 身份管理 UI | 创建/导入身份，生物识别解锁 | 2天 |
| **H-6** | 好友/聊天 UI | 联系人列表、扫码、一对一聊天 | 3天 |
| **H-7** | 群聊 + 文件 | 群组 UI、文件选择、CID 传输 | 2天 |
| **H-8** | 系统集成 | 通知、后台保活、深色模式 | 2天 |
| **H-9** | 打包 | HAP/APP + 签名 | 2天 |

---

## 四、风险矩阵（修订版）

| 风险 | 概率 | 影响 | 应对 |
|---|---|---|---|
| W-0：无 Windows 宿主机 | 高 | Windows 端阻塞 | 使用 GitHub Actions Windows runner 编译，或装虚拟机 |
| H-1：`aarch64-linux-ohos` 标准 target 不存在 | 高 | 需自定义 | 尝试 `aarch64-linux-android` + NDK linker；退路：静态编译后用 `cargo-xbuild` |
| H-1：`ring` / 汇编依赖在非标 target 编译失败 | 低 | 编译阻塞 | 当前无 `ring` 依赖，如遇到选 `ring` 的 transitive dep 则 feature flag 去掉 |
| RNW 版本与 RN 0.74 不兼容 | 中 | Windows 桥接层开发受阻 | 锁 `react-native-windows@0.74.x`，如无则升 RN |
| HarmonyOS 真机调试权限 | 中 | 无法最终验证 | 提前申请开发者证书 |

---

## 五、执行策略（给 AI 和人类）

### 优先级排序

```
第一次迭代（最高价值/最低风险）：
  W-0 → W-1 → W-2 → W-3   （Windows 桥接层，验证通路）
  └── 先在 Windows 上跑通 "Hello from DLL"
  
第二次迭代：
  H-1                      （HarmonyOS 可行性验证）
  └── 成败在此一举，先开枪再瞄准

第三次迭代：
  W-4 → W-5 → W-6 → W-7   （Windows 桌面特性）
  H-2 → H-3 → H-4         （HarmonyOS 基础设施）

第四次迭代：
  H-5 → H-6 → H-7 → H-8 → H-9  （HarmonyOS UI 完整实现）
```

### AI 执行规范
- 每个子任务完成后输出 `✅ W-N` 或 `❌ H-N + 错误原因`
- 编译错误先尝试修复，连续 3 次失败则暂停+报告
- Rust 代码统一用 `rustfmt` + `clippy`
- Phase 交替进行：编 Windows 时等 CI，抽空做 HarmonyOS

---

## 六、附录：现有导出函数清单（可直接复用）

```
scp_bridge_generate_identity()        → *mut c_char (JSON)
scp_bridge_recover_identity(*c_char)  → *mut c_char (JSON)
scp_bridge_did_from_public_key(*c_char) → *mut c_char (JSON)
scp_bridge_double_ratchet_encrypt(*c_char, *c_char) → *mut c_char (JSON)
scp_bridge_sender_key_encrypt(*c_char, *c_char)     → *mut c_char (JSON)
scp_bridge_sign_envelope(*c_char, *c_char)           → *mut c_char (JSON)
scp_bridge_free_string(*mut c_char)                  → void
```

Windows 加载方式：`LoadLibrary("scp_core_bridge.dll")` → `GetProcAddress`
HarmonyOS 加载方式：`dlopen("libscp_core_bridge.so")` → `dlsym`
