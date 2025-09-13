fn main() {
    // Android ビルドで DEP_TAURI_DEV が来ない環境向けフォールバック
    if std::env::var("DEP_TAURI_DEV").is_err() {
        // Cargo の PROFILE をヒントに dev/production を推定（なければ false=production）
        let dev = std::env::var("PROFILE")
            .map(|p| p != "release")
            .unwrap_or(false);
        unsafe { std::env::set_var("DEP_TAURI_DEV", if dev { "true" } else { "false" }) };
    }
    tauri_build::build()
}
