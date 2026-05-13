# SwarmChat App — Build Guide

Build and run the React Native mobile client for iOS and Android.

## Prerequisites

- **Node.js** ≥ 18
- **npm** or **yarn**
- **Xcode** (macOS, for iOS)
- **Android Studio** (for Android)
- **Rust toolchain** (for scp-core bridge, optional for dev)

## Quick Start

```bash
# 1. Install dependencies
cd swarmchat
npm install

# 2. Start Metro bundler
npx react-native start

# 3. Run on device/emulator (separate terminal)
npx react-native run-android   # Android
npx react-native run-ios       # iOS
```

## Rust Native Bridge (scp-core)

The app uses a Rust FFI bridge for the core crypto library (X3DH, Double Ratchet). To build it:

### Android
```bash
# Requires cargo-ndk and Android NDK
npm run build:rust:android
```

### iOS
```bash
# Requires cargo-lipo
npm run build:rust:ios
```

> **Note**: For development without the Rust bridge, set `SCP_SKIP_NATIVE=1` env var and the app falls back to pure JS crypto implementations.

## Project Structure

```
swarmchat/
├── App.tsx                          # Root component
├── src/
│   ├── screens/                     # Screen components
│   │   ├── WelcomeScreen.tsx        # Onboarding / identity create
│   │   ├── HomeScreen.tsx           # Main chat list
│   │   ├── ChatScreen.tsx           # One-to-one chat
│   │   ├── GroupChatScreen.tsx      # Group chat
│   │   ├── ContactsScreen.tsx       # Contact list
│   │   ├── AddFriendScreen.tsx      # Add friend (DID input)
│   │   ├── QRScanScreen.tsx         # QR code scanner
│   │   ├── ProfileScreen.tsx        # User profile
│   │   └── SettingsScreen.tsx       # App settings
│   ├── components/                  # Reusable UI components
│   ├── services/                    # Business logic services
│   ├── store/                       # Redux state management
│   ├── hooks/                       # Custom React hooks
│   ├── types/                       # TypeScript types
│   └── utils/                       # Utility functions
├── rust/
│   └── scp-core-bridge/             # Rust FFI bridge to scp-core
└── package.json
```

## Environment Setup (Android)

1. Install Android Studio
2. Set up `ANDROID_HOME` environment variable
3. Create an AVD (Android Virtual Device) or connect a physical device
4. Enable USB debugging on your device

## Environment Setup (iOS)

1. Install Xcode from the Mac App Store
2. Install CocoaPods: `sudo gem install cocoapods`
3. iOS pods are installed automatically via `postinstall` script

## Troubleshooting

- **Metro bundler won't start**: `npx react-native start --reset-cache`
- **iOS build fails**: `cd ios && pod install && cd ..`
- **Android build fails**: Check NDK version in Android Studio
- **Rust bridge not found**: The app will log a warning but won't crash
