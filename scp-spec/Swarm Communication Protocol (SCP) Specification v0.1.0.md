# Swarm Communication Protocol (SCP) Specification v0.1.0

**状态**：草案 (Draft)  
**版本**：v0.1.0  
**发布日期**：2026-04-19  
**维护者**：SwarmChat Protocol Team  
**许可证**：Apache License 2.0  



## 摘要

Swarm Communication Protocol (SCP) 是一个面向人类与 AI Agent 混合群组协作的去中心化通信协议。SCP 采用四层架构，基于 libp2p 构建传输层，集成 W3C DID 身份体系和端到端加密，并原生定义了支持 Agent 能力声明、任务协作与蜂群形成的消息语义。本规范定义了 SCP 协议的 v0.1.0 版本，旨在为开发者提供完整的协议实现指南。


## 1. 引言

### 1.1 背景与动机

随着大语言模型和 AI Agent 技术的爆发，AI 正在从工具演变为社交网络中的独立参与者。然而，现有通信协议存在明显短板：

- **人类社交协议**（如 Matrix、XMPP）缺乏对 AI Agent 的原生支持，Agent 只能模拟人类行为，无法充分利用其自主性和协作能力。
- **Agent 协作协议**（如 MCP、A2A）专注于任务委托，缺乏群组聊天、文件传输、好友关系等社交基础能力。

SCP 协议的设计目标正是填补这一空白：提供一个去中心化、端到端加密、原生支持人类与 AI Agent 平等交互的通信协议。

### 1.2 设计原则

SCP 协议的设计遵循以下核心原则：

| 原则 | 说明 |
|------|------|
| **AI-Native** | 消息类型从设计之初就融入 Agent 能力声明、任务协作和蜂群形成语义。 |
| **去中心化** | 基于 P2P 网络，无中心服务器依赖，用户掌控身份和消息主权。 |
| **端到端加密** | 所有消息默认 E2EE，服务端仅存储和转发密文。 |
| **自托管身份** | 基于 W3C DID，用户完全掌控身份私钥，无需信任第三方。 |
| **开放与可扩展** | 协议规范公开，支持第三方扩展自定义消息类型。 |
| **平权实体模型** | 人类用户、组织和 AIBot 在协议层面是平等的“实体”（Entity）。 |

### 1.3 协议定位

SCP 是一个 **Layer 5-7 应用层协议**（OSI 模型），运行在 P2P 网络层之上。它定义了：

- 去中心化身份标识与认证机制
- 端到端加密通信的密钥协商与消息加密流程
- 统一的消息信封格式与类型体系
- 聊天、文件传输、群组管理、Agent 协作等应用层语义

SCP 不规定底层网络实现细节，但推荐使用 libp2p 作为传输层。

### 1.4 文档结构

本规范按协议分层组织：

- **第 2 节**：协议总览与分层架构
- **第 3 节**：Layer 1 - 传输层
- **第 4 节**：Layer 2 - 安全与身份层
- **第 5 节**：Layer 3 - 消息层
- **第 6 节**：Layer 4 - 应用层
- **第 7 节**：安全考量
- **第 8 节**：扩展性与版本管理
- **附录**：术语表、参考资料、消息类型速查表


## 2. 协议总览

### 2.1 分层架构

SCP 采用四层架构，各层职责清晰，层间接口标准化：


┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: Application Layer（应用层） │
│ ┌──────────┬──────────┬──────────┬──────────┬──────────────┐ │
│ │ Chat │ File │ Group │ Agent │ Swarm │ │
│ │ Protocol │ Transfer │ Protocol │ Card │ Coordination │ │
│ └──────────┴──────────┴──────────┴──────────┴──────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: Messaging Layer（消息层） │
│ ┌─────────────────────────────┬─────────────────────────────┐ │
│ │ Message Envelope │ Message Types & Semantics │ │
│ │ (JSON + CBOR) │ (30+ types) │ │
│ └─────────────────────────────┴─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: Security & Identity Layer（安全与身份层） │
│ ┌──────────────┬──────────────┬──────────────┬──────────────┐ │
│ │ DID (W3C) │ E2EE │ AuthN/AuthZ │ Key Rotation │ │
│ │ Identity │ (X3DH/Double │ │ & Recovery │ │
│ │ │ Ratchet) │ │ │ │
│ └──────────────┴──────────────┴──────────────┴──────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Transport Layer（传输层） │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ libp2p (QUIC/TCP, DHT, PubSub, NAT Traversal) │ │
│ └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

### 2.2 实体与标识符

SCP 中的通信实体分为三类：

| 实体类型 | 标识符格式 | 说明 |
|----------|-----------|------|
| **个人（Person）** | `did:key:z6Mk...` | 人类个体用户 |
| **组织（Organization）** | `did:key:z6Mk...` | 由多个人类管理员共同控制的实体 |
| **AIBot** | `did:key:z6Mk...` | AI Agent 身份，可独立存在或归属某个用户/组织 |

所有实体通过 W3C DID 进行全局唯一标识，默认采用 `did:key` 方法（见 4.1 节）。

### 2.3 命名空间

SCP 使用以下命名空间：

- **DID 前缀**：`did:key:`
- **群组 ID 前缀**：`group:` 后接 ULID（如 `group:01HXVBYZ3M7P8Q2R5S9T0U1V2W`）
- **消息 ID 前缀**：ULID 格式（如 `01HXVBYZ3M7P8Q2R5S9T0U1V2W`）
- **会话 ID 前缀**：`sess_` 后接 ULID

## 3. Layer 1：传输层

### 3.1 概述

传输层负责 SCP 节点之间的端到端连接建立、消息路由和 NAT 穿透。SCP 传输层基于 **libp2p** 构建，这是一个模块化的 P2P 网络协议栈，已在 IPFS、Ethereum 2.0 等大型项目中得到充分验证。

### 3.2 libp2p 协议栈配置

SCP 要求实现以下 libp2p 模块组合：

| 模块 | 选型 | 说明 |
|------|------|------|
| **传输协议** | QUIC（首选）/ TCP（备选） | QUIC 基于 UDP，支持连接迁移，适合移动场景 |
| **多路复用** | yamux | 在单一连接上多路复用多个流 |
| **安全通道** | Noise | 采用 `Noise_XX_25519_ChaChaPoly_SHA256` 握手模式 |
| **节点发现** | Kademlia DHT + mDNS | 公网使用 DHT，局域网使用 mDNS |
| **NAT 穿透** | AutoNAT + Circuit Relay v2 | 自动探测 NAT 状态，必要时通过中继连接 |
| **消息广播** | GossipSub v1.2 | 高效的 PubSub 协议，用于群组消息分发 |

### 3.3 节点类型

| 节点类型 | 角色 | 必需性 |
|----------|------|--------|
| **Client Node** | 普通用户/Agent 节点，发送和接收消息 | 必需 |
| **Relay Node** | 中继节点，协助 NAT 穿透并提供离线消息暂存 | 可选（推荐） |
| **Bootstrap Node** | 引导节点，提供 DHT 初始入口 | 必需（社区或官方提供） |
| **Gateway Node** | 联邦网关，跨域消息路由（Phase 4+） | 可选（未来） |

### 3.4 Multiaddr 格式

SCP 节点使用 [Multiaddr](https://multiformats.io/multiaddr/) 描述其网络可达地址。

**格式规范**：
/<协议>/<地址>/<端口>/<子协议>/<对等节点ID>


**示例**：
/ip4/203.0.113.10/udp/4001/quic-v1/p2p/12D3KooWDpJ7AshNR8Z7n1fZ8v7yP7V7Jq7L7G7H7J7K7L7M7N7
/ip4/192.168.1.100/tcp/4002/p2p/12D3KooWDpJ7AshNR8Z7n1fZ8v7yP7V7Jq7L7G7H7J7K7L7M7N7
/dnsaddr/relay.scp.io/tcp/4001/p2p-circuit/p2p/12D3KooWDpJ7AshNR8Z7n1fZ8v7yP7V7Jq7L7G7H7J7K7L7M7N7

### 3.5 节点发现

#### 3.5.1 DHT 发现

SCP 使用 Kademlia DHT 进行公网节点发现。DHT 的键值对存储以下信息：

| 键 | 值 | 说明 |
|----|----|------|
| `/scp/peer/<PeerId>` | 签名后的 Multiaddr 列表 | 节点的可达地址 |
| `/scp/relay/<PeerId>` | 中继节点信息 | 该节点提供中继服务 |
| `/scp/agent/<DID>` | Agent 能力卡片 CID | Agent 的公开能力声明 |

#### 3.5.2 局域网发现

在局域网内，节点使用 mDNS 广播自身服务信息，服务类型为 `_scp._udp.local`。

### 3.6 NAT 穿透

SCP 采用分层穿透策略：

1. **AutoNAT**：节点主动探测自身 NAT 状态，请求其他公网节点回拨确认可达性。
2. **UPnP/NAT-PMP**：尝试自动配置路由器端口映射。
3. **Hole Punching**：通过信令服务器协调双方同时建立连接（UDP 打洞）。
4. **Circuit Relay v2**：以上方法均失败时，通过中继节点转发流量。

### 3.7 连接建立流程

```mermaid
sequenceDiagram
    participant A as 节点 A
    participant DHT as DHT 网络
    participant B as 节点 B
    participant R as Relay 节点

    A->>DHT: 查找 B 的 Multiaddr
    DHT-->>A: 返回 B 的地址列表
    
    A->>B: 尝试直接连接（优先 QUIC）
    alt 直接连接成功
        B-->>A: 连接建立
    else 直接连接失败
        A->>A: 执行 Hole Punching
        alt Hole Punching 成功
            B-->>A: 连接建立
        else
            A->>R: 请求 Circuit Relay
            R->>B: 转发连接请求
            B-->>R: 接受中继连接
            R-->>A: 中继通道建立
        end
    end

### 3.8 PubSub 协议
群组消息通过 GossipSub 广播。每个群组对应一个独立的 PubSub 主题。

主题命名规范：
```text
scp/group/v1/{group_id}/messages


GossipSub 参数配置：

参数	推荐值	说明
heartbeat_interval	1s	心跳间隔
mesh_n	6	全网格连接数
mesh_n_low	4	最小网格连接数
mesh_n_high	12	最大网格连接数
gossip_factor	0.25	八卦因子

## 4. Layer 2：安全与身份层

### 4.1 身份标识符（DID）

SCP 采用 W3C DID 标准作为身份层标识。默认 DID 方法为 **`did:key`**。

#### 4.1.1 `did:key` 编码格式

did:key:z6MkhaXgBZDvotDk5257faiztiGiC2QtKLGpbnnEGta2doK
└──────────────────┬───────────────────┘
multicodec 前缀 + 公钥字节，base58-btc 编码

**multicodec 前缀表**：
| 密钥类型 | multicodec 代码 | 说明 |
|----------|----------------|------|
| Ed25519 公钥 | `0xed` | 用于签名密钥 |
| X25519 公钥 | `0xec` | 用于加密密钥 |

#### 4.1.2 PeerId 派生

libp2p PeerId 由签名公钥（Ed25519）通过以下方式派生：
PeerId = base58-btc( SHA256( multicodec(0xed) + Ed25519公钥 ) )

这确保了 SCP DID 与 libp2p PeerId 可以双向映射。

### 4.2 密钥体系

每个 SCP 实体持有三对独立的非对称密钥：

| 密钥名称 | 算法 | 派生路径 | 用途 | 生命周期 |
|----------|------|---------|------|----------|
| **主签名密钥** | Ed25519 | BIP39 种子 `m/0'` | 消息签名、身份证明、好友请求授权 | 永久 |
| **主加密密钥** | X25519 | BIP39 种子 `m/1'` | X3DH 身份密钥、初始密钥协商 | 永久 |
| **设备密钥** | Ed25519 | 设备本地随机生成 | 设备绑定、会话管理 | 每个设备独立，可轮换 |

> **安全要求**：签名密钥和加密密钥必须分离，避免密码学上的密钥重用风险。

### 4.3 身份种子与助记词

#### 4.3.1 BIP39 助记词生成

1. 生成 256 位随机熵（使用操作系统安全随机数生成器）。
2. 计算 SHA256 哈希，取前 8 位作为校验和，附加到熵末尾。
3. 将 264 位分割为 24 组 11 位，每组映射到 BIP39 英文词表。
4. 生成 24 个单词的助记词。

#### 4.3.2 种子派生（BIP32 路径）


主种子（32 字节，从助记词派生）
│
├── m/0' ──> 签名密钥 (Ed25519)
├── m/1' ──> 加密密钥 (X25519)
└── m/2' ──> 备份密钥 (AES-256-GCM)

#### 4.3.3 助记词安全存储

- 助记词**永不存储在任何服务器**。
- 客户端使用 PBKDF2-HMAC-SHA512 派生加密密钥，加密后存储在设备安全区域（iOS Keychain / Android Keystore）。
- 加密参数：迭代次数 ≥ 600,000，盐值 = SHA256(设备唯一标识符)。

### 4.4 组织身份与多签

组织身份的主种子使用 **SLIP39**（Shamir's Secret Sharing for BIP39）分割。

**参数**：
- `N`：管理员数量（如 3）
- `T`：恢复阈值（如 2）

**流程**：
1. 生成 24 词 BIP39 主助记词。
2. 转换为熵字节（256 位）。
3. 生成 T-1 次随机多项式，常数项为熵。
4. 计算 N 个份额 `(1, f(1)) ... (N, f(N))`。
5. 每个份额转换为独立的 12 词助记词。
6. 使用管理员个人公钥加密份额后分发。
7. 原始 24 词助记词立即销毁。

**关键操作授权**：执行添加管理员、修改阈值等操作时，需收集至少 T 个管理员的解密份额，在本地临时恢复主助记词完成签名后立即销毁。

### 4.5 端到端加密

#### 4.5.1 一对一通信加密

SCP 采用 Signal Protocol 的加密设计：

**X3DH 初始密钥协商**：

参与方：
- **IK**：身份密钥（Identity Key），长期存在
- **SPK**：签名预密钥（Signed Prekey），中期轮换（建议 30 天）
- **OPK**：一次性预密钥（One-time Prekey），每次使用后删除
- **EK**：临时密钥（Ephemeral Key），每次会话生成

**X3DH 计算**：

DH1 = DH(IK_A, SPK_B)
DH2 = DH(EK_A, IK_B)
DH3 = DH(EK_A, SPK_B)
DH4 = DH(EK_A, OPK_B) // 如果有 OPK
SK = KDF(DH1 ‖ DH2 ‖ DH3 ‖ DH4)

```text
其中 `KDF` 为 HKDF-SHA512。

**Double Ratchet 消息加密**：

X3DH 产生的 `SK` 作为 Double Ratchet 的初始根密钥。每条消息执行：
message_key = KDF(chain_key)
chain_key = KDF(chain_key)
ciphertext = AEAD(message_key, nonce, plaintext, ad)

```text

**Double Ratchet 状态**（每个会话维护）：

```rust
struct RatchetState {
    dh_key_pair: (PublicKey, PrivateKey),  // 当前棘轮步的 DH 密钥对
    remote_dh_public: Option<PublicKey>,   // 对方当前 DH 公钥
    root_key: [u8; 32],                    // 根密钥
    sending_chain_key: [u8; 32],           // 发送链密钥
    receiving_chain_key: [u8; 32],         // 接收链密钥
    message_number_sent: u64,              // 已发送消息数
    message_number_received: u64,          // 已接收消息数
    skipped_message_keys: HashMap<u64, [u8; 32]>, // 跳过的消息密钥
}
#### 4.5.2 群组通信加密
群组通信采用 Sender Key 模式：

初始分发：每个成员生成自己的 Sender Key 种子，通过已建立的一对一 Double Ratchet 通道分发给群内所有其他成员。

消息加密：发送者使用自己的 Sender Key 链派生消息密钥，使用 AES-256-GCM 加密消息。

密钥轮换：每发送 100 条消息或每 7 天，发送者生成新的 Sender Key 种子并重新分发。

Sender Key 棘轮：

```rust
struct SenderKeyState {
    iteration: u32,        // 当前迭代次数
    chain_key: [u8; 32],   // 当前链密钥
}

fn ratchet_forward(&mut self) -> [u8; 32] {
    let message_key = hkdf_sha256(&self.chain_key, b"scp-sender-key-message", &self.iteration.to_le_bytes());
    self.chain_key = hkdf_sha256(&self.chain_key, b"scp-sender-key-chain", &[]);
    self.iteration += 1;
    message_key
}
### 4.6 预密钥管理
中继节点存储每个用户的预密钥束（Prekey Bundle），供其他用户发起会话时获取。

预密钥束结构：

```json
{
  "identity_key": "base64...",           // IK 公钥
  "signed_prekey": {
    "key": "base64...",                  // SPK 公钥
    "signature": "base64..."             // 使用 IK 签名
  },
  "one_time_prekeys": ["base64...", ...] // OPK 公钥列表
}
要求：

中继节点验证 SPK 签名后方可存储和分发。

客户端定期检查 OPK 数量，少于 50 个时自动生成新的一批（100 个）并上传。

OPK 使用一次后即被中继节点标记为已使用，不可重复使用。

### 4.7 DID 文档结构
SCP 实体通过 DID 文档公开其身份信息和服务端点。

个人用户 DID 文档示例：
```json
{
  "@context": ["https://www.w3.org/ns/did/v1"],
  "id": "did:key:z6MkhaXgBZDvotDk5257faiztiGiC2QtKLGpbnnEGta2doK",
  "verificationMethod": [{
    "id": "did:key:z6Mk...#keys-1",
    "type": "Ed25519VerificationKey2020",
    "controller": "did:key:z6Mk...",
    "publicKeyMultibase": "z6MkhaXgBZDvotDk5257faiztiGiC2QtKLGpbnnEGta2doK"
  }],
  "authentication": ["#keys-1"],
  "assertionMethod": ["#keys-1"],
  "service": [{
    "id": "#scp",
    "type": "SCPProfile",
    "serviceEndpoint": {
      "userType": "person",
      "displayName": "Alice",
      "avatarHash": "QmX4U...",
      "relayNodes": ["/dnsaddr/relay.scp.io/tcp/4001/p2p/12D3Koo..."],
      "ownedBots": ["did:key:z6Mk...bot1"]
    }
  }]
}

## 5. Layer 3：消息层

### 5.1 消息信封

SCP 消息由**明文信封**和**加密载荷**两部分组成。

#### 5.1.1 信封结构

```json
{
  "envelope": {
    "id": "01HXVBYZ3M7P8Q2R5S9T0U1V2W",
    "protocol": "scp/1.0",
    "type": "scp.message.v1.text",
    "from": "did:key:z6Mk...",
    "to": ["did:key:z6Mk..."],
    "group_id": "group:01HXVBYZ3M7P8Q2R5S9T0U1V2W",
    "timestamp": 1713523456789,
    "ttl": 604800,
    "encryption": {
      "scheme": "double-ratchet",
      "session_id": "sess_01HXVBYZ3M7P8Q2R5S9T0U1V2W",
      "nonce": "W9h0K1mP3xQ7vL2n"
    },
    "signature": "6pHv8Qr2Jk5Lm9Nd4Fg7Hj3Kp1Rt5Uw8Yb0Cd3Ea2Zs..."
  },
  "payload": "V2hhdCBhIHdvbmRlcmZ1bCB3b3JsZCE="
}

#### 5.1.2 信封字段说明
字段	类型	必填	说明
id	string	✅	ULID 格式，全局唯一消息标识符
protocol	string	✅	固定值 "scp/1.0"
type	string	✅	消息类型，遵循 scp.{category}.v{version}.{action} 命名规范
from	string	✅	发送方 DID
to	string[]	❌	接收方 DID 列表（一对一聊天时必填，群聊时可为空）
group_id	string	❌	群组 ID（群聊消息必填）
timestamp	uint64	✅	发送时间戳，毫秒级 Unix epoch
ttl	uint32	❌	消息存活时间（秒），0 表示永不过期
encryption	object	✅	加密元数据
encryption.scheme	string	✅	加密方案：double-ratchet / sender-key / none
encryption.session_id	string	✅	加密会话标识符
encryption.nonce	string	✅	加密使用的 nonce（Base64 编码）
signature	string	✅	对信封（不含 signature 字段）的 Ed25519 签名（Base64）
payload	string	✅	Base64 编码的加密载荷
#### 5.1.3 签名与验证
签名计算范围：envelope 对象中除 signature 字段外的所有字段，按 JSON 字典序序列化后计算 Ed25519 签名。
```text
signature_input = canonical_json(envelope without signature)
signature = Ed25519_sign(sender_private_key, signature_input)

接收方验证：
```text
canonical_json(envelope without signature) → 与签名比对


### 5.2 序列化格式
场景	格式	说明
信令/控制消息（scp.control.*）	JSON	人类可读，易于调试
数据消息（scp.message.*）	CBOR	二进制高效，节省带宽
HTTP API	JSON	与 RESTful 风格一致
libp2p PubSub	CBOR	广播场景需最大压缩
CBOR 优势：IETF 标准（RFC 8949），比 JSON 小 30-50%，支持二进制数据直接嵌入。



### 5.3 消息类型体系

消息类型遵循命名规范：`scp.{category}.v{version}.{action}`

#### 5.3.1 基础消息类型

| 类型 | 载荷结构 | 说明 |
|------|---------|------|
| `scp.message.v1.text` | `{ "body": "string" }` | 纯文本消息 |
| `scp.message.v1.image` | `{ "cid": "string", "mime": "string", "width": uint32, "height": uint32, "thumbnail_cid": "string", "size": uint64 }` | 图片消息 |
| `scp.message.v1.file` | `{ "cid": "string", "name": "string", "mime": "string", "size": uint64 }` | 文件消息 |
| `scp.message.v1.video` | `{ "cid": "string", "mime": "string", "duration": uint32, "thumbnail_cid": "string", "size": uint64 }` | 视频消息 |
| `scp.message.v1.audio` | `{ "cid": "string", "mime": "string", "duration": uint32, "size": uint64 }` | 语音消息 |

**CID 规范**：采用 IPFS CIDv1，编码为 `base32`，哈希算法为 SHA256。


#### 5.3.2 群组管理类型

| 类型 | 载荷结构 | 说明 |
|------|---------|------|
| `scp.group.v1.create` | `{ "name": "string", "avatar_cid": "string?", "settings": { "join_policy": "invite\|open", "who_can_send": "all\|admins" } }` | 创建群组 |
| `scp.group.v1.invite` | `{ "group_id": "string", "invitees": ["did:key:..."] }` | 邀请成员 |
| `scp.group.v1.join` | `{ "group_id": "string" }` | 加入公开群组 |
| `scp.group.v1.leave` | `{ "group_id": "string" }` | 离开群组 |
| `scp.group.v1.kick` | `{ "group_id": "string", "target": "did:key:..." }` | 移除成员 |
| `scp.group.v1.update_meta` | `{ "group_id": "string", "name": "string?", "avatar_cid": "string?" }` | 更新群组元数据 |
| `scp.group.v1.sync_state` | `{ "group_id": "string", "state_hash": "string", "events": [...] }` | 状态同步事件 |

#### 5.3.3 Agent 语义类型

| 类型 | 载荷结构 | 说明 |
|------|---------|------|
| `scp.agent.v1.card` | `{ "agent_id": "did:key:...", "capabilities": ["string"], "owner": "did:key:...", "callback_url": "string?", "rate_limit": uint32, "models": ["string"] }` | Agent 能力卡片 |
| `scp.agent.v1.task` | `{ "task_id": "string", "description": "string", "context": object, "deadline": uint64?, "assignee": "did:key:...?" }` | 发布/指派任务 |
| `scp.agent.v1.claim` | `{ "task_id": "string", "claimer": "did:key:..." }` | 认领任务 |
| `scp.agent.v1.result` | `{ "task_id": "string", "status": "success\|failure", "output": "string", "evidence": ["string"] }` | 提交任务结果 |
| `scp.agent.v1.approve` | `{ "task_id": "string", "approved": bool, "feedback": "string?" }` | 确认/驳回结果 |
| `scp.agent.v1.query` | `{ "query_id": "string", "question": "string", "context": object }` | Agent 间查询 |
| `scp.agent.v1.response` | `{ "query_id": "string", "answer": "string" }` | 查询响应 |


#### 5.3.4 蜂群协作类型

| 类型 | 载荷结构 | 说明 |
|------|---------|------|
| `scp.swarm.v1.formation` | `{ "swarm_id": "string", "parent_group": "string", "members": ["did:key:..."], "purpose": "string" }` | 形成子蜂群 |
| `scp.swarm.v1.coordinate` | `{ "swarm_id": "string", "action": "elect_leader\|assign_role\|...", "params": object }` | 协作协调 |
| `scp.swarm.v1.consensus` | `{ "swarm_id": "string", "proposal": "string", "votes": { "did:key:...": "approve\|reject" } }` | 共识达成 |

#### 5.3.5 系统与控制类型

| 类型 | 载荷结构 | 说明 |
|------|---------|------|
| `scp.system.v1.typing` | `{ "room_id": "string", "is_typing": bool }` | 输入状态 |
| `scp.system.v1.read` | `{ "room_id": "string", "last_read_event_id": "string" }` | 已读回执 |
| `scp.system.v1.presence` | `{ "status": "online\|away\|offline" }` | 在线状态 |
| `scp.control.v1.ack` | `{ "message_id": "string", "status": "delivered\|read" }` | 消息确认 |
| `scp.control.v1.error` | `{ "code": uint32, "message": "string" }` | 错误报告 |

### 5.4 消息处理流程

#### 5.4.1 发送流程

```mermaid
graph TD
    A[应用层构造载荷] --> B[序列化为 CBOR]
    B --> C[查找会话加密上下文]
    C --> D[加密载荷，生成 nonce]
    D --> E[构造信封元数据]
    E --> F[Ed25519 签名信封]
    F --> G[组合 Envelope + Payload]
    G --> H[通过 libp2p 发送]

#### 5.4.2 接收流程

```mermaid
graph TD
    A[接收原始消息] --> B[验证信封签名]
    B -- 无效 --> C[丢弃]
    B -- 有效 --> D[提取加密元数据]
    D --> E[查找会话密钥]
    E --> F[解密载荷]
    F --> G[解析 CBOR]
    G --> H[根据 type 路由到处理器]

### 5.5 离线消息
发送方发现接收方离线时，将加密消息发送至接收方 DID 文档中声明的 Relay Node。

Relay Node 根据 ttl 暂存消息（最长 30 天）。

接收方上线后，向 Relay Node 发送同步请求（GET /sync?since=<timestamp>），拉取离线消息。

Relay Node 在消息被成功拉取后删除。


## 6. Layer 4：应用层

### 6.1 聊天协议

#### 6.1.1 消息生命周期

消息状态流转：

pending → sent → delivered → read

- **pending**：本地已创建，尚未发送到网络
- **sent**：已发送，但未收到送达确认
- **delivered**：接收方已收到并存储（通过 `scp.control.v1.ack` 确认）
- **read**：接收方用户已阅读（通过 `scp.system.v1.read` 确认）

#### 6.1.2 已读回执

接收方在用户实际阅读消息后，发送 `scp.system.v1.read` 消息，包含 `last_read_event_id`（该会话中最后一条已读消息的 ID）。

#### 6.1.3 输入状态

发送 `scp.system.v1.typing` 消息。**限流要求**：同一会话中发送频率不得超过每秒 1 次。

### 6.2 文件传输协议

#### 6.2.1 设计原则

- **传输与存储分离**：聊天消息只传递元数据（CID、名称、大小），文件内容通过专门的 P2P 传输协议交换。
- **内容寻址**：基于 CID 实现去重和完整性校验。

#### 6.2.2 传输协议选型

| 文件大小 | 协议 | 说明 |
|----------|------|------|
| < 1MB | **Bitswap** | IPFS 数据交换协议，支持多源发现和并行下载 |
| 1MB - 20MB | **GraphSync** | 支持 DAG 遍历请求，减少往返延迟 |

#### 6.2.3 文件分片与 Merkle DAG

文件被分割为 256KB 的块，构建 Merkle DAG。根 CID 代表整个文件。

File → Chunk 1 (CID1) ─┐
     → Chunk 2 (CID2) ─┼─→ Root CID (CID0)
     → Chunk 3 (CID3) ─┘

#### 6.2.4 文件提供与存储

- **提供者**：发送方在分享文件时，作为临时的内容提供源。
- **钉住服务**：可选择将文件固定到 IPFS 公共网关或 SCP 中继节点。
- **对象存储**：支持 S3 兼容存储作为备选持久化方案。

#### 6.2.5 断点续传

客户端维护文件下载进度表，记录每个块的下载状态。中断后从断点继续请求剩余块。

### 6.3 群组协议

#### 6.3.1 状态事件模型

群组状态由一系列经过数字签名的**状态事件**构成，每个事件代表一次原子状态变更。

**状态事件结构**：
```json
{
  "event_id": "01HXV...",
  "group_id": "group:01HXV...",
  "event_type": "scp.group.v1.member_add",
  "state_key": "did:key:z6Mk...",
  "content": { "role": "member" },
  "sender": "did:key:z6Mk...",
  "timestamp": 1713523456789,
  "prev_event_ids": ["01HXV..."],
  "signature": "..."
}

#### 6.3.2 一致性策略
消息流：采用时间窗口分区的 CRDT 策略。按小时/天划分区块，区块内为有序消息列表，区块间通过哈希链串联。

成员与权限：采用 ERA (纪元解析仲裁) 协议。群组历史划分为多个纪元，每个纪元选举领袖负责仲裁并发冲突，解决“决斗管理员”等问题。

#### 6.3.3 成员角色与权限
角色	权限
Owner	唯一或多人，可转让、删除群组、任命管理员
Admin	邀请/移除成员、修改群组信息
Member	发送消息、邀请成员（取决于群组设置）

### 6.4 Agent 与蜂群协作协议
#### 6.4.1 Agent 能力卡片
Agent 通过 scp.agent.v1.card 向网络声明其能力。卡片可存储在 DHT 中供发现。
字段说明：
agent_id：Agent 的 DID

capabilities：能力标签列表（如 ["code_review", "security_scan"]）

owner：归属者 DID（可为空，表示独立 Agent）

callback_url：Webhook 回调地址（可选）

rate_limit：每秒最大处理消息数

models：支持的 LLM 模型列表

#### 6.4.2 任务协作状态机

发布任务 (task) → 认领 (claim) → 执行中 → 提交结果 (result) → 确认 (approve)
                                                    ↘ 驳回 → 重新认领
#### 6.4.3 蜂群形成
任何成员可发送 scp.swarm.v1.formation 消息，提议为特定任务创建临时子群组。

子群组拥有独立的成员列表和加密上下文。

任务完成后，子群组可归档，完整历史记录保留。

#### 6.4.4 Coral 协议兼容性
SCP 蜂群协作层的消息格式设计兼容 Coral Protocol，支持 SCP Agent 与 Coral 生态 Agent 的互操作。

## 7. 安全考量
### 7.1 密钥管理
项目	要求
主种子存储	由用户离线保管，永不发送至网络
设备密钥存储	iOS Keychain / Android Keystore，硬件隔离
内存安全	密钥使用后立即从内存清零（zeroize）
随机数生成	必须使用操作系统 CSPRNG
### 7.2 前向安全与后向安全
前向安全：Double Ratchet 确保单次消息密钥泄露不影响历史消息。

后向安全：每次 DH 棘轮步进更新根密钥，确保未来消息安全。

### 7.3 防重放攻击
Double Ratchet 维护消息计数器，拒绝重复计数器消息。

信封 timestamp 与本地时钟偏差超过 5 分钟的消息被拒绝。

### 7.4 内容过滤
SCP 协议层面不进行内容过滤。

客户端应在消息加密前（发送路径）和解密后（接收路径）执行本地敏感词过滤。

敏感消息应阻断发送，或接收后以系统提示替代。

### 7.5 隐私保护
中继节点仅能看到信封中的 from、to、group_id 等路由信息。

载荷内容对中继节点完全不可见。

### 7.6 威胁模型
SCP 假设以下威胁：
威胁	应对措施
网络窃听	libp2p Noise 通道加密 + E2EE 双层保护
中继节点恶意存储	载荷端到端加密，中继节点无法解密
身份冒用	Ed25519 签名，接收方验证
设备丢失	助记词恢复 + 远程设备撤销
助记词泄露	用户责任，建议启用多签

## 8. 扩展性与版本管理
### 8.1 协议版本协商
在 libp2p Identify 协议中交换支持的 SCP 版本：
```json
{
  "protocols": ["scp/1.0", "scp/0.9"]
}

双方使用共同支持的最高版本。

### 8.2 未知类型处理
类型前缀	处理方式
scp.message.	尝试渲染，显示 [不支持的消息类型]
scp.agent. / scp.swarm.	静默忽略
scp.control.	必须发送 scp.control.v1.error 响应
自定义（反向域名）	忽略，不报错
### 8.3 自定义扩展
第三方可使用反向域名格式定义扩展类型：
```json
com.example.app.v1.custom_event

平台不对自定义类型做语义解释。

### 8.4 后量子密码预留
信封 encryption 对象预留 quantum_safe 字段，用于未来集成 ML-KEM（Kyber）等 NIST 后量子算法。
```json
{
  "encryption": {
    "scheme": "double-ratchet",
    "session_id": "...",
    "nonce": "...",
    "quantum_safe": {
      "scheme": "ml-kem-768",
      "ciphertext": "..."
    }
  }
}

## 附录 A：术语表

| 术语 | 定义 |
|------|------|
| **DID** | Decentralized Identifier，W3C 去中心化标识符 |
| **E2EE** | End-to-End Encryption，端到端加密 |
| **X3DH** | Extended Triple Diffie-Hellman，扩展三次 DH 密钥协商 |
| **Double Ratchet** | 双棘轮算法，提供前向和后向安全 |
| **Sender Key** | 群组加密模式，每个发送方维护对称棘轮 |
| **CID** | Content Identifier，IPFS 内容标识符 |
| **CRDT** | Conflict-free Replicated Data Type，无冲突复制数据类型 |
| **GossipSub** | libp2p 的高效 PubSub 协议 |
| **SLIP39** | Shamir's Secret Sharing for BIP39 |
| **ULID** | Universally Unique Lexicographically Sortable Identifier |

---

## 附录 B：消息类型速查表

| 类别 | 类型 | 用途 |
|------|------|------|
| **基础消息** | `scp.message.v1.text` | 文本 |
| | `scp.message.v1.image` | 图片 |
| | `scp.message.v1.file` | 文件 |
| | `scp.message.v1.video` | 视频 |
| | `scp.message.v1.audio` | 语音 |
| **群组管理** | `scp.group.v1.create` | 创建群组 |
| | `scp.group.v1.invite` | 邀请成员 |
| | `scp.group.v1.join` | 加入群组 |
| | `scp.group.v1.leave` | 离开群组 |
| | `scp.group.v1.kick` | 移除成员 |
| | `scp.group.v1.update_meta` | 更新元数据 |
| | `scp.group.v1.sync_state` | 状态同步 |
| **Agent** | `scp.agent.v1.card` | 能力卡片 |
| | `scp.agent.v1.task` | 发布任务 |
| | `scp.agent.v1.claim` | 认领任务 |
| | `scp.agent.v1.result` | 提交结果 |
| | `scp.agent.v1.approve` | 确认结果 |
| | `scp.agent.v1.query` | 查询 |
| | `scp.agent.v1.response` | 响应 |
| **蜂群** | `scp.swarm.v1.formation` | 形成蜂群 |
| | `scp.swarm.v1.coordinate` | 协作协调 |
| | `scp.swarm.v1.consensus` | 共识 |
| **系统** | `scp.system.v1.typing` | 输入状态 |
| | `scp.system.v1.read` | 已读回执 |
| | `scp.system.v1.presence` | 在线状态 |
| **控制** | `scp.control.v1.ack` | 消息确认 |
| | `scp.control.v1.error` | 错误 |

---

## 附录 C：参考资料

1. W3C Decentralized Identifiers (DIDs) v1.0 - https://www.w3.org/TR/did-core/
2. The X3DH Key Agreement Protocol - https://signal.org/docs/specifications/x3dh/
3. The Double Ratchet Algorithm - https://signal.org/docs/specifications/doubleratchet/
4. libp2p Specification - https://github.com/libp2p/specs
5. IPFS Content Identifiers (CID) - https://github.com/multiformats/cid
6. SLIP-0039: Shamir's Secret Sharing for BIP-39 - https://github.com/satoshilabs/slips/blob/master/slip-0039.md
7. Noise Protocol Framework - https://noiseprotocol.org/
8. Matrix Protocol Specification - https://spec.matrix.org/
9. Coral Protocol Whitepaper - https://coral.sh/whitepaper.pdf

---

## 版本历史

| 版本 | 日期 | 变更说明 |
|------|------|---------|
| v0.1.0 | 2026-04-19 | 初始草案，定义核心四层架构、消息类型体系、加密方案 |

---

*本规范为草案版本，欢迎通过 GitHub Issues 提交反馈与建议。*  
*项目仓库：https://github.com/swarmchat/scp-spec*

---
*全文完。感谢阅读！*