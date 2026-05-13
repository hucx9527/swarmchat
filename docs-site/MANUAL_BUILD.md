# SwarmChat APK — 手动构建指引

当前 WSL 网络下载太慢（~100KB/s），需要你先从 Windows 下载以下文件，然后手动搬进 WSL。

---

## 下载清单

### 1️⃣ JDK 17（Zulu 或 Eclipse Temurin）

> **去哪下**：Windows 浏览器打开以下链接，下载 `.tar.gz` 版本（不是 installer/msi）
>
> | 来源 | 链接 |
> |------|------|
> | Azul Zulu **JDK 17** (推荐, 190MB) | https://cdn.azul.com/zulu/bin/zulu17.66.19-ca-jdk17.0.19-linux_x64.tar.gz |
> | Eclipse Temurin JDK 17 (备选) | https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.14%2B7/OpenJDK17U-jdk_x64_linux_hotspot_17.0.14_7.tar.gz |

> **搬进 WSL**：
> ```bash
> # 从 Windows Downloads 复制到 WSL /tmp/
> cp /mnt/c/Users/hjudgex/Downloads/zulu17.66.19-ca-jdk17.0.19-linux_x64.tar.gz /tmp/
> # 或
> cp /mnt/c/Users/hjudgex/Downloads/OpenJDK17U-jdk_x64_linux_hotspot_17.0.14_7.tar.gz /tmp/jdk17.tar.gz
> ```

### 2️⃣ Android SDK 命令行工具

> **下载链接**：https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
>
> **搬进 WSL**：
> ```bash
> cp /mnt/c/Users/hjudgex/Downloads/commandlinetools-linux-11076708_latest.zip /tmp/
> ```

### 3️⃣ Android NDK（用于 Rust 交叉编译）

> **下载链接**：https://dl.google.com/android/repository/android-ndk-r27c-linux.zip
> > ⚠️ 约 1.2GB，可以在继续下一步的同时后台下载

---

## 操作步骤（文件就位后我来执行）

把以上文件放到 `/tmp/` 后告诉我，我来跑下面的命令：

### 步骤 A — 安装 JDK
```bash
sudo tar -xzf /tmp/zulu17*.tar.gz -C /usr/lib/jvm/
export JAVA_HOME=/usr/lib/jvm/zulu17.66.19-ca-jdk17.0.19-linux_x64
export PATH=$JAVA_HOME/bin:$PATH
```

### 步骤 B — 安装 Android SDK + NDK
```bash
# 解压 cmdline-tools
mkdir -p ~/Android/Sdk/cmdline-tools
unzip /tmp/commandlinetools-linux-11076708_latest.zip -d /tmp/
mv /tmp/cmdline-tools ~/Android/Sdk/cmdline-tools/latest

# 安装 SDK 组件
yes | ~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager \
  "platforms;android-34" \
  "build-tools;34.0.0"

# 解压 NDK
unzip /tmp/android-ndk-r27c-linux.zip -d ~/Android/Sdk/ndk/
```

### 步骤 C — 安装 Rust Android 目标 + cargo-ndk
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
cargo install cargo-ndk
```

### 步骤 D — 构建 APK（我来执行）
- npm install
- 编译 Rust 桥接库
- React Native 打包 APK
- 上传到 docs-site

---

## 总耗时预估

| 阶段 | 你手动下载 | 我自动处理 |
|------|-----------|-----------|
| JDK 17 | ~2 分钟 (190MB) | 30 秒解压配置 |
| SDK 工具 | ~1 分钟 (135MB) | 1 分钟解压 |
| NDK | ~10 分钟 (1.2GB) | 2 分钟解压 |
| Rust 目标 + cargo-ndk | — | 3 分钟 |
| npm install | — | 2 分钟 |
| Rust 桥接编译 | — | 5 分钟 |
| APK 构建 | — | ~15 分钟 |

**总计：你下载 ~15 分钟 → 我构建 ~25 分钟 → APK 到手**
