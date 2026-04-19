#!/bin/bash

echo "Running SCP Crypto Module Integration Tests"
echo "=========================================="

cd /home/hjudgex/swarmchat/scp-core

echo ""
echo "1. Compiling the library..."
cargo build --quiet
if [ $? -ne 0 ]; then
    echo "❌ Compilation failed"
    exit 1
fi
echo "✅ Compilation successful"

echo ""
echo "2. Running unit tests..."
cargo test --quiet --lib
if [ $? -ne 0 ]; then
    echo "❌ Unit tests failed"
    exit 1
fi
echo "✅ Unit tests passed"

echo ""
echo "3. Running integration tests..."
echo "------------------------------------------"
cargo test --test crypto_integration -- --nocapture
if [ $? -ne 0 ]; then
    echo "❌ Integration tests failed"
    exit 1
fi
echo "------------------------------------------"
echo "✅ Integration tests passed"

echo ""
echo "4. Final verification..."
echo "   - Checking module structure..."
ls -la src/crypto/*.rs | wc -l | xargs echo "   - Crypto modules:"
echo "   - Total lines of crypto code:"
wc -l src/crypto/*.rs | tail -1

echo ""
echo "=========================================="
echo "SCP CRYPTO MODULES: ALL TESTS PASSED! 🎉"
echo "=========================================="
echo ""
echo "Phase 0 Cryptographic Modules Complete:"
echo "✅ P0-4: X3DH (Key Agreement)"
echo "✅ P0-5: Double Ratchet (1:1 Encryption)"
echo "✅ P0-6: Sender Key (Group Encryption)"
echo "✅ P0-7: Integration Tests"
echo ""
echo "Next: Phase 1 - Identity and DID Modules"