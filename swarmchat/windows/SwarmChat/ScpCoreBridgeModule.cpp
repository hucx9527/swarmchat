// Copyright (c) SwarmChat Contributors
// ScpCoreBridgeModule.cpp — React Native for Windows native module implementation

#include "pch.h"
#include "ScpCoreBridgeModule.h"

using namespace winrt;
using namespace Microsoft::ReactNative;

namespace winrt::SwarmChat::implementation {

    ScpCoreBridgeModule::ScpCoreBridgeModule() noexcept {
        LoadDll();
    }

    ScpCoreBridgeModule::~ScpCoreBridgeModule() {
        if (m_dllHandle) {
            FreeLibrary(m_dllHandle);
            m_dllHandle = nullptr;
        }
    }

    bool ScpCoreBridgeModule::LoadDll() {
        // Look for scp_core_bridge.dll next to the app executable
        m_dllHandle = LoadLibraryExW(
            L"scp_core_bridge.dll",
            nullptr,
            LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_USER_DIRS);

        if (!m_dllHandle) {
            OutputDebugStringW(L"[ScpCoreBridge] Failed to load scp_core_bridge.dll\n");
            return false;
        }

        // Resolve function pointers
        m_fnGenerateIdentity = reinterpret_cast<FnGenerateIdentity>(
            GetProcAddress(m_dllHandle, "scp_bridge_generate_identity"));
        m_fnRecoverIdentity = reinterpret_cast<FnRecoverIdentity>(
            GetProcAddress(m_dllHandle, "scp_bridge_recover_identity"));
        m_fnDidFromPublicKey = reinterpret_cast<FnDidFromPublicKey>(
            GetProcAddress(m_dllHandle, "scp_bridge_did_from_public_key"));
        m_fnDoubleRatchetEncrypt = reinterpret_cast<FnDoubleRatchetEncrypt>(
            GetProcAddress(m_dllHandle, "scp_bridge_double_ratchet_encrypt"));
        m_fnSenderKeyEncrypt = reinterpret_cast<FnSenderKeyEncrypt>(
            GetProcAddress(m_dllHandle, "scp_bridge_sender_key_encrypt"));
        m_fnSignEnvelope = reinterpret_cast<FnSignEnvelope>(
            GetProcAddress(m_dllHandle, "scp_bridge_sign_envelope"));
        m_fnFreeString = reinterpret_cast<FnFreeString>(
            GetProcAddress(m_dllHandle, "scp_bridge_free_string"));

        bool allLoaded = m_fnGenerateIdentity && m_fnRecoverIdentity &&
                         m_fnDidFromPublicKey && m_fnDoubleRatchetEncrypt &&
                         m_fnSenderKeyEncrypt && m_fnSignEnvelope && m_fnFreeString;

        if (!allLoaded) {
            OutputDebugStringW(L"[ScpCoreBridge] Some function symbols not found in DLL\n");
        }

        return allLoaded;
    }

    // ---- Helper methods ----

    void ScpCoreBridgeModule::CallBridge(
        std::function<char*()> fn,
        ReactPromise<JSValue> const& promise) {

        if (!m_fnFreeString || !fn) {
            promise.Reject(L"Native library not loaded");
            return;
        }

        char* rawJson = fn();
        if (!rawJson) {
            promise.Reject(L"Null response from native library");
            return;
        }

        std::string jsonStr(rawJson);
        m_fnFreeString(rawJson);

        // Parse JSON and resolve
        auto json = JSValueObject::FromString(jsonStr);
        if (json.HasProperty("error")) {
            promise.Reject(winrt::to_hstring(json["error"].AsString()));
        } else {
            promise.Resolve(std::move(json));
        }
    }

    void ScpCoreBridgeModule::CallBridgeWithArg(
        std::function<char*(const char*)> fn,
        std::string const& arg,
        ReactPromise<JSValue> const& promise) {

        if (!m_fnFreeString || !fn) {
            promise.Reject(L"Native library not loaded");
            return;
        }

        char* rawJson = fn(arg.c_str());
        if (!rawJson) {
            promise.Reject(L"Null response from native library");
            return;
        }

        std::string jsonStr(rawJson);
        m_fnFreeString(rawJson);

        auto json = JSValueObject::FromString(jsonStr);
        if (json.HasProperty("error")) {
            promise.Reject(winrt::to_hstring(json["error"].AsString()));
        } else {
            promise.Resolve(std::move(json));
        }
    }

    void ScpCoreBridgeModule::CallBridgeWithTwoArgs(
        std::function<char*(const char*, const char*)> fn,
        std::string const& arg1,
        std::string const& arg2,
        ReactPromise<JSValue> const& promise) {

        if (!m_fnFreeString || !fn) {
            promise.Reject(L"Native library not loaded");
            return;
        }

        char* rawJson = fn(arg1.c_str(), arg2.c_str());
        if (!rawJson) {
            promise.Reject(L"Null response from native library");
            return;
        }

        std::string jsonStr(rawJson);
        m_fnFreeString(rawJson);

        auto json = JSValueObject::FromString(jsonStr);
        if (json.HasProperty("error")) {
            promise.Reject(winrt::to_hstring(json["error"].AsString()));
        } else {
            promise.Resolve(std::move(json));
        }
    }

    // ---- Exported methods ----

    void ScpCoreBridgeModule::GenerateIdentity(
        ReactPromise<JSValue> &&result) noexcept {

        CallBridge([this]() {
            return m_fnGenerateIdentity();
        }, result);
    }

    void ScpCoreBridgeModule::RecoverIdentity(
        std::string mnemonic,
        ReactPromise<JSValue> &&result) noexcept {

        CallBridgeWithArg(
            [this](const char* arg) { return m_fnRecoverIdentity(arg); },
            mnemonic, result);
    }

    void ScpCoreBridgeModule::DidFromPublicKey(
        std::string publicKeyHex,
        ReactPromise<JSValue> &&result) noexcept {

        CallBridgeWithArg(
            [this](const char* arg) { return m_fnDidFromPublicKey(arg); },
            publicKeyHex, result);
    }

    void ScpCoreBridgeModule::DoubleRatchetEncrypt(
        std::string sessionId,
        std::string plaintext,
        ReactPromise<JSValue> &&result) noexcept {

        CallBridgeWithTwoArgs(
            [this](const char* a1, const char* a2) { return m_fnDoubleRatchetEncrypt(a1, a2); },
            sessionId, plaintext, result);
    }

    void ScpCoreBridgeModule::SenderKeyEncrypt(
        std::string groupId,
        std::string plaintext,
        ReactPromise<JSValue> &&result) noexcept {

        CallBridgeWithTwoArgs(
            [this](const char* a1, const char* a2) { return m_fnSenderKeyEncrypt(a1, a2); },
            groupId, plaintext, result);
    }

    void ScpCoreBridgeModule::SignEnvelope(
        std::string envelopeJson,
        std::string signingKeyBase64,
        ReactPromise<JSValue> &&result) noexcept {

        CallBridgeWithTwoArgs(
            [this](const char* a1, const char* a2) { return m_fnSignEnvelope(a1, a2); },
            envelopeJson, signingKeyBase64, result);
    }

} // namespace winrt::SwarmChat::implementation
