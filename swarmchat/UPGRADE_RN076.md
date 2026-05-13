# RN 0.74 → 0.76.9 升级改动清单

## 一、版本变更

| 包名 | 旧版本 | 新版本 |
|---|---|---|
| react-native | 0.74.0 | 0.76.9 |
| react-native-reanimated | 3.7.1 | 3.19.5 |
| react-native-safe-area-context | ^4.9.0 | 4.14.1 |
| @babel/core | ^7.24.0 | ^7.26.0 |
| @babel/preset-env | ^7.24.0 | ^7.26.0 |
| @babel/runtime | ^7.24.0 | ^7.26.0 |

新增 devDeps（RN 0.76 标配）：
- @react-native/babel-preset: 0.76.9
- @react-native/eslint-config: 0.76.9
- @react-native/metro-config: 0.76.9
- @react-native/typescript-config: 0.76.9

## 二、文件改动

### package.json
- 更新所有版本号
- 新增 `"windows": "react-native run-windows"` 脚本
- 移除 `metro-react-native-babel-preset`（被 `@react-native/babel-preset` 替代）

### babel.config.js
```diff
- presets: ['module:metro-react-native-babel-preset'],
+ presets: ['@react-native/babel-preset'],
```

### android/settings.gradle
```diff
- apply from: file("../node_modules/@react-native-community/cli-platform-android/native_modules.gradle"); applyNativeModulesSettingsGradle(settings)
+ rootProject.name = 'SwarmChat'
+ include ':app'
+ includeBuild('../node_modules/@react-native/gradle-plugin')
```

### android/build.gradle
- 增加阿里云 Maven 镜像（加速依赖下载）
- 增加 `apply plugin: "com.facebook.react.rootproject"`

### android/app/build.gradle
```diff
- apply from: file("../../node_modules/@react-native-community/cli-platform-android/native_modules.gradle"); applyNativeModulesAppBuildGradle(project)
```
（移除，RN 0.76 自动链接）

### android/gradle.properties
- `newArchEnabled=true`（保持默认）

## 三、Windows 端待办

在 Windows 宿主机（非 WSL）上执行：

```powershell
# 1. 安装 Rust MSVC target
rustup target add x86_64-pc-windows-msvc

# 2. 编译 DLL
cd rust\scp-core-bridge
cargo build --release --target x86_64-pc-windows-msvc

# 3. 初始化 React Native for Windows
cd ..\..
npx react-native-windows-init --overwrite

# 4. 将 ScpCoreBridgeModule.h/.cpp 添加到 windows/ 项目
copy windows\SwarmChat\ScpCoreBridgeModule.h windows\SwarmChat\
copy windows\SwarmChat\ScpCoreBridgeModule.cpp windows\SwarmChat\

# 5. 编译运行
npx react-native run-windows
```

C++/WinRT 桥接文件已写好：
- `windows/SwarmChat/ScpCoreBridgeModule.h`
- `windows/SwarmChat/ScpCoreBridgeModule.cpp`

详细指南见 `WINDOWS_BUILD.md`

## 四、HarmonyOS 待启动

Phase H 起步条件：
1. 确认 aarch64-linux-ohos Rust target 或替代方案
2. DevEco Studio 5.0+ 环境
3. 华为开发者账号
