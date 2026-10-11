#!/usr/bin/env python3
"""Build an arm64 verifier; signing never prints or puts passwords in argv."""
import argparse
import os
from pathlib import Path
import platform
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--sign', action='store_true')
parser.add_argument('--installed-apk', type=Path)
args = parser.parse_args()
if args.sign and not args.installed_apk:
    parser.error('--sign requires --installed-apk for certificate comparison')
sdk = Path(os.environ.get('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk')))
ndk = Path(os.environ.get('ANDROID_NDK_HOME', str(sdk / 'ndk/29.0.14206865')))
java = Path(os.environ.get('JAVA_HOME', '/Applications/Android Studio.app/Contents/jbr/Contents/Home'))
host = 'darwin-x86_64' if platform.system() == 'Darwin' else 'linux-x86_64'
llvm = ndk / 'toolchains/llvm/prebuilt' / host / 'bin'
tools = sdk / 'build-tools/35.0.0'
android_jar = sdk / 'platforms/android-36/android.jar'
output = ROOT / 'build'
classes = output / 'classes'
dex = output / 'dex'
classes.mkdir(parents=True, exist_ok=True)
dex.mkdir(parents=True, exist_ok=True)
env = dict(os.environ)
target = Path(env.get('CARGO_TARGET_DIR', str(ROOT / 'target'))).resolve()
env.update({
    'JAVA_HOME': str(java),
    'CARGO_TARGET_DIR': str(target),
    'CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER': str(llvm / 'aarch64-linux-android24-clang'),
    'CC_aarch64_linux_android': str(llvm / 'aarch64-linux-android24-clang'),
    'AR_aarch64_linux_android': str(llvm / 'llvm-ar'),
})

def run(command):
    subprocess.run([str(part) for part in command], check=True, env=env)

run(['cargo', 'build', '--manifest-path', ROOT / 'Cargo.toml', '--target', 'aarch64-linux-android', '--locked'])
source = ROOT / 'java/net/withforesight/roomtemp/migrationprobe/MigrationProbe.java'
run([java / 'bin/javac', '-source', '8', '-target', '8', '-classpath', android_jar, '-d', classes, source])
run([tools / 'd8', '--lib', android_jar, '--min-api', '24', '--output', dex,
     classes / 'net/withforesight/roomtemp/migrationprobe/MigrationProbe.class'])
unsigned = output / 'unsigned.apk'
run([tools / 'aapt2', 'link', '-I', android_jar, '--manifest', ROOT / 'AndroidManifest.xml', '-o', unsigned])
with zipfile.ZipFile(unsigned, 'a') as archive:
    archive.write(dex / 'classes.dex', 'classes.dex')
    archive.write(target / 'aarch64-linux-android/debug/libroomtemp_migration_probe.so',
                  'lib/arm64-v8a/libroomtemp_migration_probe.so')
aligned = output / 'aligned.apk'
run([tools / 'zipalign', '-P', '16', '-f', '4', unsigned, aligned])
if args.sign:
    password = subprocess.run([
        'security', 'find-generic-password', '-w', '-s',
        'net.withforesight.roomtemp.android_keypass', '-a', 'keystore_v1',
    ], capture_output=True, check=True).stdout.strip()
    signed = output / 'probe.apk'
    signing = subprocess.run([
        str(tools / 'apksigner'), 'sign', '--ks',
        str(REPO / 'src-tauri/gen/android/app/build/signing/tmp-release.jks'),
        '--ks-key-alias', 'upload', '--ks-pass', 'stdin', '--key-pass', 'stdin',
        '--out', str(signed), str(aligned),
    ], input=password + b'\n' + password + b'\n', capture_output=True, env=env)
    del password
    if signing.returncode:
        raise SystemExit('APK signing failed; output suppressed to protect credentials')

    def certificate(apk):
        result = subprocess.run([str(tools / 'apksigner'), 'verify', '--print-certs', str(apk)],
                                capture_output=True, check=True, env=env)
        return next(line for line in result.stdout.splitlines() if b'certificate SHA-256 digest:' in line)

    if certificate(signed) != certificate(args.installed_apk):
        signed.unlink()
        raise SystemExit('Certificate mismatch; signed verifier removed')
    print('Certificate matches installed app. APK:', signed)
else:
    print('Unsigned aligned APK:', aligned)
