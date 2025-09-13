import java.io.FileInputStream
import java.util.Base64
import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// ====== ここから セキュアストレージ連携ユーティリティ ======
val osName = System.getProperty("os.name").lowercase()
val isMac = osName.contains("mac")
val isWin = osName.contains("win")

/** macOS Keychain: `security find-generic-password -w -s <service> -a <account>` */
fun keychain(service: String, account: String) =
providers.exec {
    commandLine("security", "find-generic-password", "-w", "-s", service, "-a", account)
}.standardOutput.asText.map { it.trim() }

/** Windows Credential Manager (PowerShell):
*  事前に: Install-Module CredentialManager -Scope CurrentUser
*  取得: (Get-StoredCredential -Target "<name>").Password
*/
fun cred(service: String, account: String) =
  providers.exec {
    commandLine(
      "powershell", "-NoProfile", "-Command",
      "Import-Module CredentialManager; " +
        "(Get-StoredCredential -Target '${account}.${service}').Password"
    )
  }.standardOutput.asText.map(String::trim)

// 取得名（サービス名/ターゲット名）は必要に応じて変更可
val svcKeystoreB64 = "net.withforesight.roomtemp.android_keystore_b64"
// val svcStorePass   = "net.withforesight.roomtemp.android_storepass"
val svcKeyPass     = "net.withforesight.roomtemp.android_keypass"

// macOS 用アカウント名（Keychain の -a）
val keychainAccount = "keystore_v1"

// keystore Base64 / パスワードを OS 別に取得
val keystoreB64Provider = if (isMac) keychain(svcKeystoreB64, keychainAccount) else if (isWin) cred(svcKeystoreB64, keychainAccount) else
providers.provider { throw GradleException("Unsupported OS for secure keystore retrieval") }

// val storePassProvider = if (isMac) keychain(svcStorePass,   keychainAccount) else if (isWin) cred(svcStorePass) else
// providers.provider { throw GradleException("Unsupported OS for store password retrieval") }

val keyPassProvider   = if (isMac) keychain(svcKeyPass, keychainAccount) else if (isWin) cred(svcKeyPass, keychainAccount) else
providers.provider { throw GradleException("Unsupported OS for key password retrieval") }

// 復元先（ビルド成果物配下なので、リポジトリに残らない）
val restoredKeystore = layout.buildDirectory.file("signing/tmp-release.jks")

// keystore を先に復元してから署名検証/assemble を走らせる
tasks.configureEach {
  if (
    name.startsWith("validateSigning") ||
    name.startsWith("signingConfigWriter") ||
    name.startsWith("package") ||
    name.startsWith("assemble")
  ) {
    dependsOn(restoreKeystore)
  }
}

// keystore 復元タスク
val restoreKeystore by tasks.registering {
    outputs.file(restoredKeystore)
    doLast {
        val b64: String = keystoreB64Provider.orNull
            ?: throw GradleException("Secure keystore (Base64) not found in OS secure storage.")
        val bytes = Base64.getDecoder().decode(b64)
        val out = restoredKeystore.get().asFile
        out.parentFile.mkdirs()
        out.writeBytes(bytes)
        logger.lifecycle("[signing] Restored keystore to: ${out.absolutePath}")
    }
}

android {
    compileSdk = 36
    namespace = "net.withforesight.roomtemp"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "net.withforesight.roomtemp"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    signingConfigs {
        // create("release") {
        //     val keystorePropertiesFile = rootProject.file("keystore.properties")
        //     val keystoreProperties = Properties()
        //     if (keystorePropertiesFile.exists()) {
        //         keystoreProperties.load(FileInputStream(keystorePropertiesFile))
        //     }

        //     keyAlias = keystoreProperties["keyAlias"] as String
        //     keyPassword = keystoreProperties["password"] as String
        //     storeFile = file(keystoreProperties["storeFile"] as String)
        //     storePassword = keystoreProperties["password"] as String
        // }
        create("secureRelease") {
            storeFile = restoredKeystore.get().asFile
            // storePassword = storePassProvider.get()
            storePassword = keyPassProvider.get() // 例: キーストアと同じパスワードを使う場合
            keyAlias = "upload"
            keyPassword = keyPassProvider.get()
            // keyPassword = storePassProvider.get()
            // keyPassword = "" // キーストアと同じパスワードを使う場合

        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
            signingConfig = signingConfigs.getByName("secureRelease")
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")
