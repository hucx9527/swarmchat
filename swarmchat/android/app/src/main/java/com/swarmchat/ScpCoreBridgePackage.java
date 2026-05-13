package com.swarmchat;

import androidx.annotation.NonNull;

import com.facebook.react.ReactPackage;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.uimanager.ViewManager;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * ScpCoreBridgePackage — registers the ScpCoreBridge native module
 * with React Native's NativeModules registry.
 *
 * Add this package to your Application's getPackages() list in MainApplication.java:
 *
 *   @Override
 *   protected List<ReactPackage> getPackages() {
 *       List<ReactPackage> packages = new PackageList(this).getPackages();
 *       packages.add(new ScpCoreBridgePackage());
 *       return packages;
 *   }
 */
public class ScpCoreBridgePackage implements ReactPackage {

    @NonNull
    @Override
    public List<NativeModule> createNativeModules(@NonNull ReactApplicationContext reactContext) {
        List<NativeModule> modules = new ArrayList<>();
        modules.add(new ScpCoreBridgeModule(reactContext));
        return modules;
    }

    @NonNull
    @Override
    public List<ViewManager> createViewManagers(@NonNull ReactApplicationContext reactContext) {
        return Collections.emptyList();
    }
}
