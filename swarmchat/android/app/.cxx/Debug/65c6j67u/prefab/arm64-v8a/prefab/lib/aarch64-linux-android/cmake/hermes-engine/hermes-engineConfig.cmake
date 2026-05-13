if(NOT TARGET hermes-engine::libhermes)
add_library(hermes-engine::libhermes SHARED IMPORTED)
set_target_properties(hermes-engine::libhermes PROPERTIES
    IMPORTED_LOCATION "/home/hjudgex/.gradle/caches/8.12/transforms/3e3b3a602bbe8fea943b7f4096305a1d/transformed/jetified-hermes-android-0.74.0-debug/prefab/modules/libhermes/libs/android.arm64-v8a/libhermes.so"
    INTERFACE_INCLUDE_DIRECTORIES "/home/hjudgex/.gradle/caches/8.12/transforms/3e3b3a602bbe8fea943b7f4096305a1d/transformed/jetified-hermes-android-0.74.0-debug/prefab/modules/libhermes/include"
    INTERFACE_LINK_LIBRARIES ""
)
endif()

