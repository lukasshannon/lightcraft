# LightCraft Android APK branch

This branch carries the native Android port developed in this chat, based on
`storytold/lightcraft` at `31ee2643cf208a18cf780b8a63c677e87ab256f7`. The existing editor changes are applied directly
to their original files. `lightcraft-android/` is an independent Cargo workspace containing
the NativeActivity adapter, document-provider bridge, manifest, icon resources
and pinned Cargo.lock. The Android adapter points at this checkout's `crates/`.

## Build on Windows

Install Python 3, Git and Visual Studio C++ Build Tools. In this checkout:

```powershell
python setup-toolchain.py
$env:CARGO_HOME = Join-Path $PWD '.tools/cargo'
$env:RUSTUP_HOME = Join-Path $PWD '.tools/rustup'
& ./.tools/rustup-init.exe -y --no-modify-path --default-toolchain 1.95.0
. ./build-lightcraft-env.ps1
rustup target add aarch64-linux-android x86_64-linux-android wasm32-unknown-unknown
git clone https://github.com/storytold/craft-fonts.git craft-fonts
git -C craft-fonts checkout 43a994403242b287ab4c78499b7ff7d943ea6dba
python build-lightcraft-apk.py --abi arm64-v8a
python build-lightcraft-apk.py --abi x86_64
```

The helper uses JDK 21, Android SDK 35, NDK r27c and 16 KiB ELF/ZIP alignment.
APKs and metadata are written under `artifacts/`. The build generates a local
test signing key under `.private/` if absent. A newly generated key cannot
update the already distributed APK in place; retain your own signing identity
for subsequent builds. No signing keys, SDKs, APKs, fonts, provider grants,
private hosting configuration or uploaded user screenshots are committed here.

## Validation and provenance

The editor and adapter sources are the previously validated Android port. This
publication reorganizes packaging paths for a standalone fork. It does not
rebase onto newer upstream code or claim another full emulator/build run.
The adapter's locked Cargo metadata and Python syntax are checked after relocation.
The original upstream repository and its licenses remain the source of truth.

## Upstream contribution status

This is an unofficial Android distribution branch, not a claim that the port
has passed upstream review. The editor stays native Rust/egui; the Android
Activity uses handwritten Java for OS document-provider, lifecycle and keyboard
integration. Packaging uses Python and PowerShell. Each native entry also uses
Rust 2024's `#[unsafe(no_mangle)]` attribute to export `android_main`; it is
an unsafe attribute, not an unsafe block. These boundaries must be reviewed
against the upstream Rust-only and unsafe-code policies before submission.

Related request: [#707](https://github.com/storytold/lightcraft/issues/707).
Overlapping work already exists: [#818](https://github.com/storytold/lightcraft/pull/818), [#2802](https://github.com/storytold/lightcraft/pull/2802).
A future upstream proposal must explain that overlap, build on existing work
where appropriate, and use a reviewed current-main base. The tested APK base
above is deliberately retained here; an upstream rebase and its tests have not
been completed. New UI text and shortcuts, icon attribution, dependency policy
and the repository's PR template/checks must all be verified on that proposal.

Do not mark CI, device coverage or upstream-policy exceptions as approved
based only on the previous APK tests or the packaging checks above.
