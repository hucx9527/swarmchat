# React Native ProGuard rules

# Keep annotations
-keepattributes *Annotation*

# Keep React Native
-keep class com.facebook.react.** { *; }
-keep class com.facebook.hermes.** { *; }

# Keep our native module
-keep class com.swarmchat.** { *; }

# Keep Hermes
-keep class com.facebook.jni.** { *; }
