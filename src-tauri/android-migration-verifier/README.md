# Android credential migration verifier

This standalone instrumentation APK runs the repository's production
`keystore.rs` and `crypto.rs` inside an existing RoomTemp application's process.
It reads the legacy credential independently with android-native-keyring-store
**0.4.0**, then exercises the production migration into the named 1.0 store.
The existing RoomTemp APK is not replaced. Android requires the instrumentation
APK to be signed with the same certificate as its target application.

The verifier checks an existing, nonempty encrypted access token. It never prints
key material, token contents, URLs, or raw platform errors. Its SQLite connection
uses `mode=ro`; it compares the database and legacy preferences before/after,
checks the retained legacy key, and verifies equality of decrypted plaintext.

`migrate` requires the named key to be absent. It **does write the named store**
using the production migration code. `verify` requires that key to already exist
and checks persistence from another instrumentation process. Neither mode deletes
credentials or creates a replacement key when the legacy key is missing.

## Build

Requirements: arm64 Android device, Rust Android target, Android SDK platform 36,
Build Tools 35.0.0, NDK 29.0.14206865, and JDK (Android Studio's bundled JDK works).
The standalone `Cargo.lock` pins the verifier's dependencies, including both
legacy and current store implementations.

```sh
rustup target add aarch64-linux-android
python3 src-tauri/android-migration-verifier/build.py
```

The default output is an unsigned aligned APK. `ANDROID_HOME`,
`ANDROID_NDK_HOME`, `JAVA_HOME`, and `CARGO_TARGET_DIR` can override the paths.
Build outputs are ignored by Git.

On macOS, the signing option reuses this repository's existing ignored
`src-tauri/gen/android/app/build/signing/tmp-release.jks` and password from the
configured OS keychain entry. It passes the password through stdin, never argv,
and suppresses signing output. It verifies the signature and compares its
certificate with a previously pulled installed APK; mismatches remove the
signed result.

```sh
python3 src-tauri/android-migration-verifier/build.py \
  --sign --installed-apk /absolute/path/to/installed-base.apk
```

## Run

Select the intended device with `adb -s SERIAL` if multiple devices are connected.
The commands below install **only the verifier package**. Do not uninstall or
clear the target RoomTemp application.

```sh
adb install src-tauri/android-migration-verifier/build/probe.apk
adb shell am instrument -w -e mode migrate \
  net.withforesight.roomtemp.migrationprobe/net.withforesight.roomtemp.migrationprobe.MigrationProbe
adb shell am instrument -w -e mode verify \
  net.withforesight.roomtemp.migrationprobe/net.withforesight.roomtemp.migrationprobe.MigrationProbe
adb uninstall net.withforesight.roomtemp.migrationprobe
adb shell am start -n net.withforesight.roomtemp/.MainActivity
```

Success requires both `migration_report=PASS ...` and
`INSTRUMENTATION_CODE: -1`. A shell exit code alone does not establish success.
After migration, rerun **only `verify`**: `migrate` intentionally rejects an
already present named key. The verifier operates on real app data, so use it only
when verification of that installation is intended.

See [the physical-device result](../../docs/android-migration-verification.md).
