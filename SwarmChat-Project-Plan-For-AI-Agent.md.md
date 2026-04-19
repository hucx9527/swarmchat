# SwarmChat 项目执行计划书（AI Agent 开发版）

**目标**：由 AI 编程助手按照本计划逐步完成 SwarmChat 去中心化通信平台的开发。  
**协议**：Swarm Communication Protocol (SCP) v0.1.0  
**技术栈**：Rust (core)、Go (relay/cli)、React Native (app)、TypeScript  

---

## 一、总体原则（给 AI Agent 的元指令）

1. **分阶段交付**：每个 Phase 结束后应产出一个可独立运行、可演示的里程碑版本，不要一次性写完所有代码。
2. **代码优先，文档辅助**：优先保证 `scp-core` 核心库的编译和测试通过，再补充注释和文档。
3. **测试驱动**：每个加密模块（X3DH、Double Ratchet）必须有单元测试，用 `cargo test` 验证。
4. **遵循规范**：严格按 `SCP-Specification-v0.1.0.md` 中定义的消息格式、加密流程和类型体系实现。
5. **任务粒度**：每完成一个子任务，请输出简短的状态报告（如“已完成 P1-1：DID 生成模块”）。

---

## 二、Phase 0：核心库骨架与加密原语（预计 4 周）

### 目标
搭建 `scp-core` Rust 项目，实现 DID 生成、密钥派生、X3DH、Double Ratchet。

### 子任务列表

| 编号 | 任务 | 输入/依赖 | 预期产出 |
|------|------|----------|---------|
| P0-1 | 创建 `scp-core` Cargo 项目，配置 `Cargo.toml` 依赖（ed25519-dalek, x25519-dalek, aes-gcm, sha2, hkdf, rand, bip39, serde, cbor） | `SCP-Specification-v0.1.0.md` 第 4、5 章 | 可编译的空项目骨架 |
| P0-2 | 实现 `identity` 模块：从 BIP39 助记词生成种子，派生 Ed25519 签名密钥和 X25519 加密密钥 | 规范 4.3 节 | `generate_mnemonic()`, `derive_keys()` |
| P0-3 | 实现 `did` 模块：根据 Ed25519 公钥生成 `did:key` 标识符，解析 DID 提取公钥 | 规范 4.1 节 | `did_from_public_key()`, `public_key_from_did()` |
| P0-4 | 实现 `crypto/x3dh` 模块：执行 X3DH 密钥协商，输入双方身份密钥、预密钥，输出共享密钥 SK | 规范 4.5.1 节 | 通过单元测试验证 DH 计算正确 |
| P0-5 | 实现 `crypto/double_ratchet` 模块：维护会话状态，每条消息派生新的消息密钥 | 规范 4.5.1 节 | `RatchetState` 结构体，`encrypt()`/`decrypt()` 方法 |
| P0-6 | 实现 `crypto/sender_key` 模块：群组加密的对称棘轮 | 规范 4.5.2 节 | `SenderKeyState`，每 100 条消息自动轮换 |
| P0-7 | 编写加密模块集成测试：模拟 Alice 和 Bob 完成一次完整的加密会话 | | `tests/e2e_encryption.rs` 通过 |

### 验收标准
- 运行 `cargo test` 全部通过。
- 提供一个示例程序 `examples/simple_chat.rs`，演示两个内存中的 `RatchetState` 互相加解密消息。

---

## 三、Phase 1：传输层与消息信封（预计 4 周）

### 目标
集成 libp2p，实现节点发现、连接建立，并实现 SCP 消息信封的编解码。

### 子任务列表

| 编号 | 任务 | 输入/依赖 | 预期产出 |
|------|------|----------|---------|
| P1-1 | 在 `scp-core` 中引入 `libp2p` 依赖，配置 QUIC、Noise、yamux、Kademlia DHT、GossipSub | 规范 3.2 节 | `transport::P2PNetwork` 结构体 |
| P1-2 | 实现节点启动：生成 PeerId，监听 Multiaddr，连接 Bootstrap 节点 | 规范 3.7 节 | 可加入公共 DHT 网络 |
| P1-3 | 实现 `messaging::envelope` 模块：定义 `Envelope` 结构体，支持 JSON/CBOR 序列化，实现签名与验证 | 规范 5.1 节 | `Envelope::sign()`, `Envelope::verify()` |
| P1-4 | 实现消息类型枚举：至少包含 `text`、`image`、`file`、`group.create`、`agent.card` | 规范 5.3 节 | `MessageType` 枚举，载荷结构体 |
| P1-5 | 实现发送/接收基础消息：通过 libp2p 直连发送加密后的信封，接收方解密并解析 | | 两个节点可互相发送文本消息 |
| P1-6 | 实现离线消息逻辑：当目标节点不可达时，将消息发送到其中继节点 | 规范 5.5 节 | 中继节点暂存消息，目标上线后拉取 |

### 验收标准
- 两个 `scp-core` 实例可以通过 libp2p 互相发现、建立加密连接、发送文本消息。
- 消息信封签名验证通过，载荷正确加解密。

---

## 四、Phase 2：中继节点与 CLI 工具（预计 4 周）

### 目标
用 Go 语言实现 `scp-relay` 中继服务和 `scp-cli` 命令行工具。

### 子任务列表

| 编号 | 任务 | 输入/依赖 | 预期产出 |
|------|------|----------|---------|
| P2-1 | 搭建 `scp-relay` Go 项目，使用 `libp2p/go-libp2p` 实现 Relay 协议 | 规范 3.3 节 | 可接收客户端预留请求并转发流量 |
| P2-2 | 实现预密钥存储：提供 HTTP API 供客户端上传/获取预密钥束 | 规范 4.6 节 | `POST /prekey`, `GET /prekey/{did}` |
| P2-3 | 实现离线消息存储：接收加密信封，按 TTL 存储，提供 `/sync` 拉取接口 | 规范 5.5 节 | 使用 SQLite 或 Badger 持久化 |
| P2-4 | 搭建 `scp-cli` Go 项目，封装对 `scp-core` FFI 的调用（或直接使用 Go 版协议实现） | | 可执行 `identity create`, `message send` 等命令 |
| P2-5 | CLI 实现所有 GUI 操作对等命令：登录、发送消息、文件传输、群组管理、Agent 卡片发布 | | 参考规范附录 B 类型表 |
| P2-6 | 编写 CLI 使用文档和示例脚本 | | `README.md` 包含快速开始指南 |

### 验收标准
- 部署一个公共中继节点，手机模拟器可通过该节点穿透 NAT。
- CLI 工具能完成身份创建、添加好友、发送消息、创建群组全流程。

---

## 五、Phase 3：React Native 移动端（预计 8 周）

### 目标
使用 React Native 开发 iOS/Android 客户端，集成 `scp-core` 的 Rust 库。

### 子任务列表

| 编号 | 任务 | 输入/依赖 | 预期产出 |
|------|------|----------|---------|
| P3-1 | 搭建 RN 项目，配置 TypeScript、react-native-mmkv、react-navigation | | 基础 UI 框架 |
| P3-2 | 通过 JSI/FFI 将 `scp-core` 编译为 RN 可调用的原生模块 | | `ScpCore` 模块暴露 `createIdentity`、`sendMessage` 等方法 |
| P3-3 | 实现身份创建/恢复流程 UI：助记词展示、备份提醒、双重验证 | | 用户可注册新身份或导入助记词 |
| P3-4 | 实现好友添加：扫码解析 DID，发送好友请求，双向确认 | | 好友列表和聊天入口 |
| P3-5 | 实现一对一聊天界面：文本输入、消息气泡、已读状态、输入指示 | | 完整的单聊体验 |
| P3-6 | 实现群组聊天：创建群组、邀请成员、群设置、@提及 | | 群聊 UI 和逻辑 |
| P3-7 | 实现文件传输：选择图片/文件，计算 CID，通过 Bitswap/GraphSync 发送 | | 支持图片、文件（上限 20MB） |
| P3-8 | 实现本地敏感词过滤：在发送前和解密后调用过滤模块 | | 违规内容阻断并提示 |
| P3-9 | 实现 Agent 扫码添加：扫描 Agent 二维码，展示能力卡片，确认添加 | | Agent 成为好友并可在群聊中互动 |
| P3-10 | 性能优化与多平台测试 | | 在 iOS 和 Android 真机运行流畅 |

### 验收标准
- 在 TestFlight / 内部测试渠道发布 Beta 版，完成一次 10 人以上的真实群聊测试。
- 至少接入一个第三方 AI Agent（例如通过 CLI 运行的 ChatGPT Bot）。

---

## 六、Phase 4：Agent 生态与蜂群协作（预计 4 周）

### 目标
完善 Agent 任务协作和蜂群形成功能，发布 SDK 和文档。

### 子任务列表

| 编号 | 任务 | 输入/依赖 | 预期产出 |
|------|------|----------|---------|
| P4-1 | 在 `scp-core` 中实现任务消息类型的处理逻辑：`task`、`claim`、`result`、`approve` | 规范 5.3.3 节 | 群内可发布任务，Agent 可认领和提交 |
| P4-2 | 实现蜂群形成：从主群组创建子群组，邀请相关成员和 Agent | 规范 6.4.3 节 | 子群组独立加密和聊天 |
| P4-3 | 编写 Python SDK（`scp-sdk-python`）：封装 Webhook 服务器、消息处理装饰器 | | 发布到 PyPI |
| P4-4 | 编写 TypeScript SDK（`scp-sdk-js`）：供 Node.js Agent 使用 | | 发布到 npm |
| P4-5 | 搭建开发者门户网站：包含协议规范、API 文档、快速开始教程 | | `docs.scp.io` 上线 |
| P4-6 | 提供官方示例 Agent：Echo Bot、ChatGPT Bot、Code Review Bot | | 三个可直接扫码添加的示例 |

### 验收标准
- 第三方开发者可通过文档在 30 分钟内创建并上线一个 Agent。
- Agent 可在群组中完成完整的任务认领→执行→确认流程。

---

## 七、AI Agent 执行提示词（System Prompt）

将以下提示词复制给你的 AI 编程助手，它就知道如何理解并执行上述计划。
