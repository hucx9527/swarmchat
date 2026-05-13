package com.swarmchat;

import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.ReadableMap;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.module.annotations.ReactModule;

import org.json.JSONObject;

/**
 * ScpCoreBridgeModule — React Native native module bridging to scp-core (Rust)
 * via JNI.
 *
 * Each @ReactMethod delegates to a corresponding JNI native method declared
 * in the Rust scp-core-bridge crate (see rust/scp-core-bridge/src/lib.rs).
 *
 * In development mode without the native library loaded, the JS fallback
 * in ScpCoreBridge.ts provides placeholder implementations.
 */
@ReactModule(name = ScpCoreBridgeModule.NAME)
public class ScpCoreBridgeModule extends ReactContextBaseJavaModule {

    public static final String NAME = "ScpCoreBridge";
    private static final String TAG = "ScpCoreBridge";

    // Set to true once the native library is loaded
    private static boolean nativeLibLoaded = false;

    static {
        try {
            System.loadLibrary("scp_core_bridge");
            nativeLibLoaded = true;
            Log.i(TAG, "Native scp_core_bridge library loaded successfully");
        } catch (UnsatisfiedLinkError e) {
            nativeLibLoaded = false;
            Log.w(TAG, "Native scp_core_bridge library not available, will use JS fallback: " + e.getMessage());
        }
    }

    public ScpCoreBridgeModule(ReactApplicationContext reactContext) {
        super(reactContext);
    }

    @NonNull
    @Override
    public String getName() {
        return NAME;
    }

    /**
     * Check if the native library is loaded.
     * The JS bridge can call this to decide whether to use native or JS fallback.
     */
    @ReactMethod
    public void isNativeAvailable(Promise promise) {
        promise.resolve(nativeLibLoaded);
    }

    // ---- Identity ----

    /** Generate a new BIP39 identity (mnemonic + Ed25519 key + DID + PeerId) */
    @ReactMethod
    public void generateIdentity(Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeGenerateIdentity();
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                WritableMap map = Arguments.createMap();
                map.putString("mnemonic", json.getString("mnemonic"));
                map.putString("seedHex", json.getString("seed_hex"));
                map.putString("did", json.getString("did"));
                map.putString("peerId", json.getString("peer_id"));
                map.putString("publicKey", json.getString("public_key"));
                promise.resolve(map);
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Recover identity from a 24-word BIP39 mnemonic */
    @ReactMethod
    public void recoverIdentity(String mnemonic, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeRecoverIdentity(mnemonic);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                WritableMap map = Arguments.createMap();
                map.putString("mnemonic", json.getString("mnemonic"));
                map.putString("seedHex", json.getString("seed_hex"));
                map.putString("did", json.getString("did"));
                map.putString("peerId", json.getString("peer_id"));
                map.putString("publicKey", json.getString("public_key"));
                promise.resolve(map);
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Generate a did:key from a hex-encoded Ed25519 public key */
    @ReactMethod
    public void didFromPublicKey(String publicKeyHex, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeDidFromPublicKey(publicKeyHex);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                promise.resolve(json.getString("did"));
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    // ---- Encryption ----

    /** Double Ratchet encrypt */
    @ReactMethod
    public void doubleRatchetEncrypt(String sessionId, String plaintext, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeDoubleRatchetEncrypt(sessionId, plaintext);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                WritableMap map = Arguments.createMap();
                map.putString("ciphertext", json.getString("ciphertext"));
                map.putString("nonce", json.optString("nonce"));
                promise.resolve(map);
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Double Ratchet decrypt */
    @ReactMethod
    public void doubleRatchetDecrypt(String sessionId, String ciphertext, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeDoubleRatchetDecrypt(sessionId, ciphertext);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                promise.resolve(json.getString("plaintext"));
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Sender Key encrypt for group messaging */
    @ReactMethod
    public void senderKeyEncrypt(String groupId, String plaintext, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeSenderKeyEncrypt(groupId, plaintext);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                WritableMap map = Arguments.createMap();
                map.putString("ciphertext", json.getString("ciphertext"));
                map.putString("nonce", json.optString("nonce"));
                promise.resolve(map);
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Sender Key decrypt */
    @ReactMethod
    public void senderKeyDecrypt(String groupId, String ciphertext, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeSenderKeyDecrypt(groupId, ciphertext);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                promise.resolve(json.getString("plaintext"));
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    // ---- Envelope ----

    /** Sign a message envelope */
    @ReactMethod
    public void signEnvelope(String envelopeJson, String signingKeyBase64, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeSignEnvelope(envelopeJson, signingKeyBase64);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                promise.resolve(json.getString("signed_envelope"));
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    /** Verify a signed envelope */
    @ReactMethod
    public void verifyEnvelope(String envelopeJson, String publicKeyBase64, Promise promise) {
        if (!nativeLibLoaded) {
            promise.reject("NATIVE_NOT_LOADED", "Native library not available");
            return;
        }
        try {
            String result = nativeVerifyEnvelope(envelopeJson, publicKeyBase64);
            JSONObject json = new JSONObject(result);
            if (json.has("error")) {
                promise.reject("BRIDGE_ERROR", json.getString("error"));
            } else {
                promise.resolve(json.getBoolean("valid"));
            }
        } catch (Exception e) {
            promise.reject("BRIDGE_EXCEPTION", e.getMessage(), e);
        }
    }

    // ================================================================
    // JNI native method declarations (implemented in Rust)
    // ================================================================

    private static native String nativeGenerateIdentity();
    private static native String nativeRecoverIdentity(String mnemonic);
    private static native String nativeDidFromPublicKey(String publicKeyHex);
    private static native String nativeDoubleRatchetEncrypt(String sessionId, String plaintext);
    private static native String nativeDoubleRatchetDecrypt(String sessionId, String ciphertext);
    private static native String nativeSenderKeyEncrypt(String groupId, String plaintext);
    private static native String nativeSenderKeyDecrypt(String groupId, String ciphertext);
    private static native String nativeSignEnvelope(String envelopeJson, String signingKeyBase64);
    private static native String nativeVerifyEnvelope(String envelopeJson, String publicKeyBase64);
}
