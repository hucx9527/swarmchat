# SwarmChat App — 构建指引

构建并运行 React Native 移动客户端，支持 iOS 和 Android。

## 前置依赖

- **Node.js** ≥ 18
- **npm** 或 **yarn**
- **Xcode**（macOS，用于 iOS 构建）
- **Android Studio**（用于 Android 构建）
- **Rust 工具链**（编译 scp-core 桥接库，开发时可跳过）

## 快速启动

```bash
# 1. 安装依赖
cd swarmchat
npm install

# 2. 启动 Metro 打包器
npx react-native start

# 3. 运行到设备/模拟器（另开终端）
npx react-native run-android   # Android
npx react-native run-ios       # iOS
```

## Rust 原生桥接 (scp-core)

App 通过 Rust FFI 桥接调用核心加密库（X3DH、Double Ratchet）。构建方法：

### Android
```bash
# 需要 cargo-ndk 和 Android NDK
npm run build:rust:android
```

### iOS
```bash
# 需要 cargo-lipo
npm run build:rust:ios
```

> **提示**：不编译 Rust 桥接也能开发，设置 `SCP_SKIP_NATIVE=1` 环境变量即可回退到纯 JS 加密实现。

## 项目结构

```
swarmchat/
├── App.tsx                          # 根组件
├── src/
│   ├── screens/                     # 页面组件
│   │   ├── WelcomeScreen.tsx        # 欢迎页面 / 创建身份
│   │   ├── HomeScreen.tsx           # 主聊天列表
│   │   ├── ChatScreen.tsx           # 一对一聊天
│   │   ├── GroupChatScreen.tsx      # 群聊
│   │   ├── ContactsScreen.tsx       # 联系人列表
│   │   ├── AddFriendScreen.tsx      # 添加好友（输入 DID）
│   │   ├── QRScanScreen.tsx         # 二维码扫描
│   │   ├── ProfileScreen.tsx        # 用户资料
│   │   └── SettingsScreen.tsx       # 设置
│   ├── components/                  # 可复用 UI 组件
│   ├── services/                    # 业务逻辑服务
│   ├── store/                       # Redux 状态管理
│   ├── hooks/                       # 自定义 React Hooks
│   ├── types/                       # TypeScript 类型
│   └── utils/                       # 工具函数
├── rust/
│   └── scp-core-bridge/             # scp-core Rust FFI 桥接
└── package.json
```

## Android 环境配置

1. 安装 Android Studio
2. 设置 `ANDROID_HOME` 环境变量
3. 创建 AVD（Android 虚拟设备）或连接真机
4. 启用 USB 调试

## iOS 环境配置

1. 从 Mac App Store 安装 Xcode
2. 安装 CocoaPods：`sudo gem install cocoapods`
3. iOS pods 通过 `postinstall` 脚本自动安装

## 常见问题

- **Metro 无法启动**：`npx react-native start --reset-cache`
- **iOS 构建失败**：`cd ios && pod install && cd ..`
- **Android 构建失败**：检查 Android Studio 中的 NDK 版本
- **找不到 Rust 桥接**：App 会打印警告，不会崩溃
