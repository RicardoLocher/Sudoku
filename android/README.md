# Android build

This folder packages the Rust app as an APK using Gradle + `cargo-ndk`.
The app is a `NativeActivity`: there is no Java/Kotlin code, only the
`libsudoku.so` produced from the Rust `cdylib`.

## Prerequisites (Arch Linux)

```bash
# 1. JDK 17 (Gradle needs it)
sudo pacman -S jdk17-openjdk

# 2. Rust Android targets
rustup target add aarch64-linux-android

# 3. cargo-ndk (drives cross-compilation)
cargo install cargo-ndk

# 4. Android SDK + NDK
#    Easiest: Android Studio, or the commandline-tools below.
#    Set these in your shell profile:
#      export ANDROID_HOME=$HOME/Android/Sdk
#      export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>
#      export PATH=$PATH:$ANDROID_HOME/platform-tools   # adb
#    Then accept licenses:
#      sdkmanager --licenses
#      sdkmanager "platform-tools" "platforms;android-34" "ndk;<version>"
```

`minSdk` is 24 (required by egui/winit on Android).

## Build + install (physical arm64 device)

```bash
# from the repository root, build the .so into app/src/main/jniLibs
./android/build-rust.sh --release

# build the APK (uses the Gradle wrapper if present, else system gradle)
cd android
gradle :app:assembleDebug          # or ./gradlew :app:assembleDebug

# install on a connected device
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Enable **USB debugging** on the phone and confirm `adb devices` lists it first.

## Notes

- The library name must stay `sudoku` to match
  `android.app.lib_name` in the manifest and `crate-type = ["cdylib"]`.
- Only `arm64-v8a` is wired up (physical 64-bit phones). For the emulator add
  `-t x86_64` to `cargo ndk` and it will drop into `jniLibs/x86_64/`.
- No Gradle wrapper is committed; run `gradle wrapper` once, or use a system
  `gradle` (8.x).
