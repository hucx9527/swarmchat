# SwarmChat Windows 客户端构建指南

## 前置条件

### Windows 宿主机需要
- Visual Studio 2022 社区版（勾选"使用 C++ 的桌面开发"工作负载）
- Windows SDK (10.0.22621.0+)
- Rust 工具链：`rustup target add x86_64-pc-windows-msvc`
- Node.js 18 LTS+
- Git

### 从 WSL 复制项目到 Windows
```powershell
# 在 Windows PowerShell 中执行
# WSL 项目路径: /home/hjudgex/projects/swarmchat/swarmchat/
# 复制到 Windows: C:\dev\swarmchat\
robocopy \\wsl.localhost\Ubuntu\home\hjudgex\projects\swarmchat\swarmchat C:\dev\swarmchat /E /NP
```

---

## 步骤 1: 编译 scp-core-bridge DLL

```powershell
cd C:\dev\swarmchat\rust\scp-core-bridge
cargo build --release --target x86_64-pc-windows-msvc
```

验证产出：
```
dir target\x86_64-pc-windows-msvc\release\scp_core_bridge.dll
```

复制 DLL 到 RNW 原生库目录：
```powershell
copy target\x86_64-pc-windows-msvc\release\scp_core_bridge.dll ..\..\windows\ScpCoreBridge\
```

---

## 步骤 2: 初始化 React Native for Windows

```powershell
cd C:\dev\swarmchat
npm install
npx react-native-windows-init --overwrite
```

这会：
1. 生成 `windows/` 目录（含 `.sln` 解决方案文件）
2. 配置 RNW 原生依赖
3. 自动链接已安装的 react-native 包

---

## 步骤 3: 添加 C++/WinRT 原生模块

复制预先写好的桥接文件到 RNW 项目：

```powershell
copy windows\SwarmChat\ScpCoreBridgeModule.h windows\SwarmChat\
copy windows\SwarmChat\ScpCoreBridgeModule.cpp windows\SwarmChat\
```

**手动步骤（在 Visual Studio 中）：**
1. 打开 `windows\SwarmChat.sln`
2. 在 `SwarmChat` 项目上右键 → 添加 → 现有项
3. 选择 `ScpCoreBridgeModule.h` 和 `ScpCoreBridgeModule.cpp`
4. 在 `ReactPackageProvider.cpp` 中注册模块：

```cpp
// 在文件顶部添加 include
#include "ScpCoreBridgeModule.h"

// 在 CreateReactPackageProvider 中添加：
void AddScpCoreBridge(winrt::Microsoft::ReactNative::IReactPackageBuilder const &packageBuilder) noexcept
{
    packageBuilder.AddModule(L"ScpCore", [](winrt::Microsoft::ReactNative::IReactModuleBuilder const &moduleBuilder) noexcept {
        return winrt::make<winrt::SwarmChat::implementation::ScpCoreBridgeModule>();
    });
}
```

---

## 步骤 4: 构建运行

```powershell
npx react-native run-windows
```

首次构建较慢（下载 NuGet 包 + 编译 C++ 代码）。

---

## 步骤 5: JS 侧验证

在 React Native JS 代码中调用：

```typescript
import { NativeModules } from 'react-native';

// 验证桥接通路
NativeModules.ScpCore.generateIdentity()
  .then((result: any) => {
    console.log('DID:', result.did);
    console.log('PeerId:', result.peerId);
    console.log('Mnemonic:', result.mnemonic);
  })
  .catch((err: any) => {
    console.error('Bridge error:', err);
  });
```

---

## 常见问题

| 问题 | 解法 |
|---|---|
| `LoadLibrary` 失败 (126) | DLL 不在搜索路径。把 `scp_core_bridge.dll` 复制到 `windows\Debug\` 或 `windows\Release\` |
| `GetProcAddress` 找不到符号 | 确认 DLL 导出了正确的函数名：`dumpbin /exports scp_core_bridge.dll` |
| RNW 构建版本不匹配 | `react-native-windows` 版本需与 `react-native` 版本匹配 |
