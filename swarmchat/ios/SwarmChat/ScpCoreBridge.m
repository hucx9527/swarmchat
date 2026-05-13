/**
 * ScpCoreBridge.m
 * SwarmChat iOS
 *
 * React Native native module implementation for iOS.
 * Calls into the scp-core-bridge Rust static library via C FFI.
 *
 * To integrate:
 *   1. Build the Rust library for iOS:
 *      cargo build --release --target aarch64-apple-ios
 *   2. Add `librust_scp_core_bridge.a` to Xcode Link Binary With Libraries
 *   3. Add this file to the Xcode project
 */

#import "ScpCoreBridge.h"
#import <React/RCTLog.h>

@implementation ScpCoreBridge

RCT_EXPORT_MODULE();

// Check if native library symbols are available
RCT_EXPORT_METHOD(isNativeAvailable:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  // In production, attempt a lightweight call to verify linkage
  // For now, assume yes (the library is statically linked)
  resolve(@YES);
}

// ---- Identity ----

RCT_EXPORT_METHOD(generateIdentity:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    char* result = scp_bridge_generate_identity();
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(@{
        @"mnemonic": dict[@"mnemonic"] ?: @"",
        @"seedHex": dict[@"seed_hex"] ?: @"",
        @"did": dict[@"did"] ?: @"",
        @"peerId": dict[@"peer_id"] ?: @"",
        @"publicKey": dict[@"public_key"] ?: @"",
      });
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

RCT_EXPORT_METHOD(recoverIdentity:(NSString *)mnemonic
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    const char* mn = [mnemonic UTF8String];
    char* result = scp_bridge_recover_identity(mn);
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(@{
        @"mnemonic": dict[@"mnemonic"] ?: @"",
        @"seedHex": dict[@"seed_hex"] ?: @"",
        @"did": dict[@"did"] ?: @"",
        @"peerId": dict[@"peer_id"] ?: @"",
        @"publicKey": dict[@"public_key"] ?: @"",
      });
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

RCT_EXPORT_METHOD(didFromPublicKey:(NSString *)publicKeyHex
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    const char* pk = [publicKeyHex UTF8String];
    char* result = scp_bridge_did_from_public_key(pk);
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(dict[@"did"]);
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

// ---- Encryption ----

RCT_EXPORT_METHOD(doubleRatchetEncrypt:(NSString *)sessionId
                  plaintext:(NSString *)plaintext
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    const char* sid = [sessionId UTF8String];
    const char* pt = [plaintext UTF8String];
    char* result = scp_bridge_double_ratchet_encrypt(sid, pt);
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(@{
        @"ciphertext": dict[@"ciphertext"] ?: @"",
        @"nonce": dict[@"nonce"] ?: @"",
      });
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

RCT_EXPORT_METHOD(doubleRatchetDecrypt:(NSString *)sessionId
                  ciphertext:(NSString *)ciphertext
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  // iOS decryption placeholder — delegates to C FFI in production
  // For now, return a placeholder
  resolve(@"decrypted_placeholder");
}

RCT_EXPORT_METHOD(senderKeyEncrypt:(NSString *)groupId
                  plaintext:(NSString *)plaintext
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    const char* gid = [groupId UTF8String];
    const char* pt = [plaintext UTF8String];
    char* result = scp_bridge_sender_key_encrypt(gid, pt);
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(@{
        @"ciphertext": dict[@"ciphertext"] ?: @"",
        @"nonce": dict[@"nonce"] ?: @"",
      });
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

RCT_EXPORT_METHOD(senderKeyDecrypt:(NSString *)groupId
                  ciphertext:(NSString *)ciphertext
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  resolve(@"decrypted_placeholder");
}

// ---- Envelope ----

RCT_EXPORT_METHOD(signEnvelope:(NSString *)envelopeJson
                  signingKeyBase64:(NSString *)signingKeyBase64
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  @try {
    const char* ej = [envelopeJson UTF8String];
    const char* sk = [signingKeyBase64 UTF8String];
    char* result = scp_bridge_sign_envelope(ej, sk);
    NSString *json = [NSString stringWithUTF8String:result];
    scp_bridge_free_string(result);

    NSData *data = [json dataUsingEncoding:NSUTF8StringEncoding];
    NSDictionary *dict = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];

    if (dict[@"error"]) {
      reject(@"BRIDGE_ERROR", dict[@"error"], nil);
    } else {
      resolve(dict[@"signed_envelope"]);
    }
  } @catch (NSException *exception) {
    reject(@"BRIDGE_EXCEPTION", exception.reason, nil);
  }
}

RCT_EXPORT_METHOD(verifyEnvelope:(NSString *)envelopeJson
                  publicKeyBase64:(NSString *)publicKeyBase64
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
  resolve(@YES);
}

@end
