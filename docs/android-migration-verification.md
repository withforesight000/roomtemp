# Android existing-data migration verification

Date: 2026-10-11 (JST)

Production implementation: `f46a718` on `codex/update-dependencies`.

Device: Pixel 8 Pro, Android 17 / API 37 (reported by the connected device).
Existing application: `net.withforesight.roomtemp`, version 0.1.0,
versionCode 1000, non-debuggable release installation.

## Method

A separate, same-certificate instrumentation APK executed the production
`keystore.rs` and `crypto.rs` against this installation's real Android Keystore,
SharedPreferences, and existing SQLite ciphertext. It used the old 0.4.0
provider to independently obtain the legacy key and compared it with the
production migration result. It opened SQLite read-only and did not save settings.

The certificate matched the installed APK before the verifier was installed.
The original release APK was retained, and its installation/update metadata was
unchanged. No target uninstall, data clear, credential deletion, fixture token,
or replacement encryption key was used.

## Results

Initial migration, in the existing app's process:

```text
PASS mode=migrate;old_provider=0.4.0;named_before=false;key_equal=true;existing_token_nonempty=true;plaintext_equal=true;legacy_retained=true;database_unchanged=true;legacy_preferences_unchanged=true
INSTRUMENTATION_CODE: -1
```

A second instrumentation invocation restarted the target process and read the
persisted named key:

```text
PASS mode=verify;old_provider=0.4.0;named_before=true;key_equal=true;existing_token_nonempty=true;plaintext_equal=true;legacy_retained=true;database_unchanged=true;legacy_preferences_unchanged=true
INSTRUMENTATION_CODE: -1
```

Thus the migration copied the actual legacy key into the named store, verified
its saved value, decrypted the existing nonempty token to the same plaintext,
and preserved the original key, legacy preference bytes, and database bytes.
Persistence was confirmed from a new process.

## Scope and final state

The named production credential now exists on the device. The legacy credential
remains. The existing release APK was not upgraded; the verifier exercised the
current production migration modules directly. This establishes real-store and
real-ciphertext migration, rather than an APK upgrade/UI test of the new release.

After removal of the verifier and restart of the existing release app, its dashboard
and Fetch Data control were visible, with no Keystore error displayed. A successful
gRPC connection was not established by this check.

The verifier package was removed after the checks. Key material, token contents,
and server addresses are omitted from this report.

Reproducible source and commands:
[android-migration-verifier](../src-tauri/android-migration-verifier/README.md).

Android's same-certificate instrumentation model is documented by
[AOSP's target-app instrumentation example](https://android.googlesource.com/platform/platform_testing/+/android-8.1.0_r22/docs/development/instr-app-e2e.md).
