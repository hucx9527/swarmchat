#!/bin/bash
#
# patch-rn074-compat.sh
# 给 React Native 0.74 + NDK 27 的项目打兼容补丁。
# 用法：项目根目录下执行  bash scripts/patch-rn074-compat.sh
#
# 解决的问题：
#   1. react-native-reanimated 3.7.1 C++ 与 RN 0.74 / NDK 27 不兼容
#   2. react-native-screens 4.10.0 缺少 RN 0.74 的 prefab 目标
#   3. react-native-gesture-handler 2.31.2 codegen 引用不存在的 NewArch 接口
#

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

echo "=== patching react-native-reanimated ==="
REANIMATED_CMAKE="node_modules/react-native-reanimated/android/CMakeLists.txt"

# 去掉 -Werror（NDK 27 Clang 把 deprecation 当错误）
if grep -q -- "-Werror" "$REANIMATED_CMAKE"; then
  sed -i 's/ -Wall -Werror"/ -Wall"/' "$REANIMATED_CMAKE"
  echo "  [OK] 移除 -Werror"
else
  echo "  [SKIP] -Werror 已不存在"
fi

# 非 NewArch 模式下排除 Fabric 目录的 .cpp 文件
if grep -q "GLOB_RECURSE SOURCES_COMMON" "$REANIMATED_CMAKE" | grep -v "IS_NEW_ARCHITECTURE_ENABLED" >/dev/null 2>&1; then :; fi
EXISTING_GLOB=$(grep -c 'file(GLOB_RECURSE SOURCES_COMMON CONFIGURE_DEPENDS' "$REANIMATED_CMAKE" || true)
if [ "$EXISTING_GLOB" -gt 0 ] && ! grep -q 'IS_NEW_ARCHITECTURE_ENABLED' "$REANIMATED_CMAKE"; then
  # Replace the single glob with conditional glob
  sed -i 's|file(GLOB_RECURSE SOURCES_COMMON CONFIGURE_DEPENDS "${COMMON_SRC_DIR}/cpp/\*\*\.cpp")|if(${IS_NEW_ARCHITECTURE_ENABLED})\n    file(GLOB_RECURSE SOURCES_COMMON CONFIGURE_DEPENDS "${COMMON_SRC_DIR}/cpp/**.cpp")\nelse()\n    file(GLOB_RECURSE SOURCES_COMMON CONFIGURE_DEPENDS "${COMMON_SRC_DIR}/cpp/AnimatedSensor/**.cpp" "${COMMON_SRC_DIR}/cpp/LayoutAnimations/**.cpp" "${COMMON_SRC_DIR}/cpp/NativeModules/**.cpp" "${COMMON_SRC_DIR}/cpp/ReanimatedRuntime/**.cpp" "${COMMON_SRC_DIR}/cpp/Registries/**.cpp" "${COMMON_SRC_DIR}/cpp/SharedItems/**.cpp" "${COMMON_SRC_DIR}/cpp/Tools/**.cpp" "${COMMON_SRC_DIR}/cpp/hidden_headers/**.cpp")\nendif()|' "$REANIMATED_CMAKE"
  echo "  [OK] 非NewArch排除Fabric .cpp源文件"
else
  echo "  [SKIP] 条件 glob 已存在或旧的 glob 不存在"
fi

echo "=== patching react-native-screens ==="
SCREENS_CMAKE="node_modules/react-native-screens/android/CMakeLists.txt"
if grep -q "react_render_consistency" "$SCREENS_CMAKE"; then
  sed -i '/ReactAndroid::react_render_consistency/d' "$SCREENS_CMAKE"
  sed -i '/ReactAndroid::react_performance_timeline/d' "$SCREENS_CMAKE"
  sed -i '/ReactAndroid::react_render_observers_events/d' "$SCREENS_CMAKE"
  echo "  [OK] 移除3个不存在的prefab链接目标"
else
  echo "  [SKIP] 3个prefab目标已不存在"
fi

echo "=== patching react-native-gesture-handler ==="
GH_BASE="node_modules/react-native-gesture-handler/android/paper/src/main/java/com/facebook/react/viewmanagers"

for file in RNGestureHandlerRootViewManagerInterface.java RNGestureHandlerButtonManagerInterface.java; do
  f="$GH_BASE/$file"
  if [ -f "$f" ]; then
    # 去掉 extends ViewManagerWithGeneratedInterface 和对应的 import
    sed -i '/import com.facebook.react.uimanager.ViewManagerWithGeneratedInterface;/d' "$f"
    sed -i 's/ extends ViewManagerWithGeneratedInterface//' "$f"
    echo "  [OK] $file 修复"
  else
    echo "  [SKIP] $file 不存在"
  fi
done

echo "=== patching android/gradle.properties ==="
if grep -q "newArchEnabled=true" android/gradle.properties; then
  sed -i 's/newArchEnabled=true/newArchEnabled=false/' android/gradle.properties
  echo "  [OK] 关闭 NewArchitecture"
else
  echo "  [SKIP] newArchEnabled 已为 false"
fi

echo ""
echo "=== 全部补丁已打完 ==="
echo "运行 ./gradlew assembleDebug 即可重新构建"
