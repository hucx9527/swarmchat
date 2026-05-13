// Copyright (c) SwarmChat Contributors
// ScpCoreBridgeModule.h — React Native for Windows native module
//
// Loads scp_core_bridge.dll via LoadLibrary/GetProcAddress
// and exposes C FFI functions as REACT_METHODs to JavaScript.

#pragma once

#include "pch.h"
#include <winrt/Windows.Foundation.h>
#include <functional>
#include <string>
#include "NativeModules.h"

namespace winrt::SwarmChat::implementation {

    struct ScpCoreBridgeModule : winrt::implements<
        ScpCoreBridgeModule,
        winrt::Microsoft::ReactNative::IReactModuleBuilder> {

        ScpCoreBridgeModule() noexcept;
        ~ScpCoreBridgeModule();

        // Module name exposed to JS: NativeModules.ScpCore
        static std::string const &ModuleName() noexcept {
            static std::string name = L"ScpCore";
            return name;
        }

        // React Native module registration
        static std::vector<winrt::Microsoft::ReactNative::ModuleMethod> GetMethods() noexcept;
        static std::map<std::string, winrt::Microsoft::ReactNative::ConstantProvider> GetConstants() noexcept;

        // ---- Exported methods (REACT_METHOD) ----

        // Generate a new identity (mnemonic + DID + keypair)
        void GenerateIdentity(
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

        // Recover identity from BIP39 mnemonic
        void RecoverIdentity(
            std::string mnemonic,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

        // Derive did:key from hex public key
        void DidFromPublicKey(
            std::string publicKeyHex,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

        // Double Ratchet encrypt
        void DoubleRatchetEncrypt(
            std::string sessionId,
            std::string plaintext,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

        // Sender Key encrypt (group messaging)
        void SenderKeyEncrypt(
            std::string groupId,
            std::string plaintext,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

        // Sign a message envelope
        void SignEnvelope(
            std::string envelopeJson,
            std::string signingKeyBase64,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> &&result) noexcept;

    private:
        // DLL handle and function pointers
        HMODULE m_dllHandle = nullptr;

        // C FFI function pointer types (matching scp-core-bridge exports)
        using FnGenerateIdentity = char* (*)();
        using FnRecoverIdentity = char* (*)(const char*);
        using FnDidFromPublicKey = char* (*)(const char*);
        using FnDoubleRatchetEncrypt = char* (*)(const char*, const char*);
        using FnSenderKeyEncrypt = char* (*)(const char*, const char*);
        using FnSignEnvelope = char* (*)(const char*, const char*);
        using FnFreeString = void (*)(char*);

        FnGenerateIdentity m_fnGenerateIdentity = nullptr;
        FnRecoverIdentity m_fnRecoverIdentity = nullptr;
        FnDidFromPublicKey m_fnDidFromPublicKey = nullptr;
        FnDoubleRatchetEncrypt m_fnDoubleRatchetEncrypt = nullptr;
        FnSenderKeyEncrypt m_fnSenderKeyEncrypt = nullptr;
        FnSignEnvelope m_fnSignEnvelope = nullptr;
        FnFreeString m_fnFreeString = nullptr;

        // Load DLL and resolve symbols
        bool LoadDll();

        // Helper: call C FFI, parse JSON result, resolve/reject promise
        void CallBridge(
            std::function<char*()> fn,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> const& promise);

        void CallBridgeWithArg(
            std::function<char*(const char*)> fn,
            std::string const& arg,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> const& promise);

        void CallBridgeWithTwoArgs(
            std::function<char*(const char*, const char*)> fn,
            std::string const& arg1,
            std::string const& arg2,
            winrt::Microsoft::ReactNative::ReactPromise<winrt::Microsoft::ReactNative::JSValue> const& promise);
    };

} // namespace winrt::SwarmChat::implementation
