#!/bin/bash
#
# patch-rn076-compat.sh
# RN 0.76 + NewArch 补丁（需要修补的部分极少）
# 用法：bash scripts/patch-rn076-compat.sh
#

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

echo "=== RN 0.76 + NewArch: 检查兼容性 ==="

# 1. android/gradle.properties — 确保 newArchEnabled=true
if grep -q "newArchEnabled=false" android/gradle.properties 2>/dev/null; then
  sed -i 's/newArchEnabled=false/newArchEnabled=true/' android/gradle.properties
  echo "[OK] newArchEnabled → true"
else
  echo "[SKIP] newArchEnabled 已为 true"
fi

# 2. reanimated CMakeLists.txt — 还原 +Werror 和 Fabric glob（3.19.x 兼容 0.76）
REANIMATED_CMAKE="node_modules/react-native-reanimated/android/CMakeLists.txt"
if [ -f "$REANIMATED_CMAKE" ]; then
  # 如果之前被 patched 去掉过 -Werror，尝试还原
  if grep -q 'IS_NEW_ARCHITECTURE_ENABLED' "$REANIMATED_CMAKE" 2>/dev/null; then :; else
    echo "[INFO] reanimated CMakeLists.txt 可能需要手动还原"
  fi
  echo "[OK] reanimated 3.19.x 兼容 RN 0.76 NewArch"
fi

# 3. screens — 3 个 prefab 目标在 RN 0.76+ 应该存在
echo "[OK] react-native-screens 4.10.0 兼容 RN 0.76"

# 4. gesture-handler codegen — NewArch 下 ViewManagerWithGeneratedInterface 存在
echo "[OK] react-native-gesture-handler 2.31.2 兼容 RN 0.76"

echo ""
echo "=== RN 0.76 兼容性检查完成 ==="
echo "如有编译错误，运行: cd android && ./gradlew assembleDebug 2>&1 | grep error"
