# RoomTemp

RoomTemp is a cross-platform desktop and mobile application that visualizes ambient temperature, humidity, and illumination collected from a dedicated gRPC service. The UI is built with Next.js (App Router) and Tailwind, while a Rust-based Tauri backend handles secure configuration storage and gRPC communication.

## Features

- Interactive temperature, humidity, and illumination charts with time-range selection.
- Secure gRPC connectivity using bearer-token authentication and optional HTTP proxy support.
- In-app settings editor with encrypted persistence backed by the OS keychain.
- Unified Tauri configuration for desktop and mobile builds with explicit capability definitions.

## Tech Stack

- **Frontend:** Next.js 16 (App Router), React 19, Tailwind CSS, shadcn/ui, Recharts.
- **Bridge:** Tauri 2 with IPC commands exposed through `@tauri-apps/api`.
- **Backend:** Rust 1.98.1 (edition 2024; pinned via mise), Diesel + SQLite, tonic gRPC client, ChaCha20-Poly1305 encryption.
- **Proto definitions:** TypeScript bindings from the versioned `tempgrpcd-protos-v0.1.0` GitHub Release tarball (HTTPS) and Rust bindings from `tempgrpcd-protos` on `withforesight000/protobuf-rust` (HTTPS).

The TypeScript protobuf package is distributed at:

- [tempgrpcd-protos v0.1.0](https://github.com/withforesight000/protobuf-ts/releases/tag/tempgrpcd-protos-v0.1.0)
- Artifact: `tempgrpcd-protos-0.1.0.tgz`
- SHA-256: `83362b5dc105f944f4c9a375129caf76b1741154c11f4cca6cb7576c10a6dacf`
- Source commit: `42916f2ccc7babfe1799ae16ae2193c18bb206a1` (see the release's `provenance.json`).

When updating this package, verify the artifact against `SHA256SUMS`, review its provenance and generated-code changes, update the exact release URL and lockfile, and run type checks, build, and the WebDriver suite. Release assets can be replaced; the checksum and lockfile integrity pin their contents.

## Requirements

- [mise](https://mise.jdx.dev/getting-started.html) for installing and selecting Bun 1.4.3 and Rust 1.98.1 from the tracked `mise.toml`. Node.js/npm/pnpm are not required for application tooling.
- The platform prerequisites listed in the [Tauri 2 documentation](https://v2.tauri.app/start/).
- HTTPS access to GitHub Releases for the TypeScript proto package and Git access to `withforesight000/protobuf-rust` for the Rust proto package.
- A reachable `tempgrpcd` gRPC server that supports TLS and bearer-token authentication.

Install mise, then set up the project tools:

```bash
mise trust
mise install
mise exec -- bun --version
mise exec -- rustc --version
```

`bunfig.toml` forces Bun for Node-shebang tools, uses isolated dependency linking, and applies a 24-hour minimum release age to new registry resolutions. Only `protobufjs`, `sharp`, and `unrs-resolver` install scripts are trusted. Commit `bun.lock` alongside dependency changes; retain both TypeScript aliases and review peer resolutions when updating. The protobuf generator's `@typescript/vfs` retains its TypeScript 5.4.5 peer through `@bufbuild/protoplugin`, while Next.js uses the TypeScript 6 alias.

Run project tasks with `mise run`, or use `mise exec -- <command>` without shell activation. To run the existing Bun/Cargo commands directly, [activate mise in your shell](https://mise.jdx.dev/cli/activate.html). Machine-specific overrides belong in ignored `mise.local.toml`; do not add Node or pnpm to the project toolchain. GitHub JavaScript actions use the runner's own Node runtime, independently of application tooling.

CI installs tools from `mise.toml` using `jdx/mise-action`. Keep its Bun pin synchronized with `.bun-version`, `package.json` and the Docker image default; keep the Rust pin synchronized with the Docker rustup installer. Rust uses rustup internally, including the `llvm-tools` component needed for coverage. Android SDK/NDK and JDK remain platform prerequisites; configure them as described by Tauri before using the Android tasks.

## Quick Start

```bash
# install dependencies
mise run install

# run the Next.js dev server and Tauri shell together
mise run dev
```

Other tasks: `mise run lint`, `mise run build`, `mise run test:rust`, `mise run test:e2e`, and `mise run tauri:build`. For Android arm64, run `mise run android:setup` once, then `mise run android:build`. List available tasks with `mise tasks`.

## Testing

### Frontend E2E (Tauri WebDriver)

WebDriver-driven end-to-end tests run against the real Tauri shell through the `tauri-driver` binary (`src-tauri/webdriver/test/test.js`). The suite enables `NEXT_PUBLIC_TAURI_WEBDRIVER_MOCKS` during the build so the UI always receives deterministic mock data without touching an external gRPC server.

Before running the suite install the WebDriver stack:

```bash
cargo install tauri-driver --locked
```

Make sure the appropriate native driver is available on `PATH` (WebKitWebDriver on Linux, `msedgedriver` on Windows) or provide a custom path via `TAURI_DRIVER_PATH`.

#### Running the suite inside Docker (macOS)

`tauri-driver` is not supported natively on macOS, so run the tests inside the Linux container defined at `docker/webdriver.Dockerfile`. The image bundles Bun, Rust, `tauri-driver`, and `webkit2gtk-driver`, and the helper script runs `xvfb` so the WebView can start headlessly:

```bash
docker build --build-arg BUN_VERSION="$(cat .bun-version)" --target base -t roomtemp-webdriver -f docker/webdriver.Dockerfile .
docker run --rm -it \
  -v "$PWD":/workspace \
  -v roomtemp-bun-node:/workspace/node_modules \
  -v roomtemp-bun-target:/workspace/src-tauri/target \
  -v roomtemp-bun-next:/workspace/.next \
  -v roomtemp-bun-cache:/root/.bun/install/cache \
  -e CI=true -e NEXT_TELEMETRY_DISABLED=1 \
  -w /workspace \
  roomtemp-webdriver \
  ./docker/run-webdriver.sh
```

Run the suite through the helper script:

```bash
bun --bun run test:e2e
```

`bun --bun run test:e2e` will build the Tauri app with the mock flag, start `tauri-driver` on port 4444, and execute the Mocha test harness through Selenium.

### Frontend unit and Hook tests

```sh
bun --bun run test:frontend
```

The explicit `tests/frontend` directory keeps Bun from collecting the Mocha WebDriver suite. DOM globals are preloaded using happy-dom; React Testing Library runs the actual Hooks, including root Strict Mode setup/cleanup. Deferred promises control completion order without timing sleeps. Tests cover IPC contracts, decoder validation, synchronous duplicate admission, save/fetch races, delayed loads versus edits, and form/provider teardown.

Run `bun --bun run lint` for both TypeScript versions and Biome. Run the Linux WebDriver suite separately: the simulated DOM does not replace the real Tauri WebView, and WebDriver mocks do not verify production authentication, TLS or secure storage. A malformed graph payload is a fetch error; valid empty protobuf and legacy JSON `{}` remain successful empty results.

### Rust unit tests & coverage

Rust unit tests live under `src-tauri`. Package scripts are available for running them from the repository root:

```bash
bun --bun run test:rust           # runs `cd src-tauri && cargo test`
bun --bun run test:rust:coverage  # runs `cd src-tauri && cargo llvm-cov --workspace`
```

If you want a quick human-readable coverage output on stdout, run:

```bash
cd src-tauri
cargo llvm-cov --workspace --text --show-missing-lines
```

Note: `cargo-llvm-cov` is required; install it with `cargo install cargo-llvm-cov` if not already available.

`bun --bun run tauri dev` automatically runs the Next.js development server defined in the Tauri configuration and opens the Tauri window once the frontend is ready.

### Linting

`mise run lint` runs TypeScript 7 and TypeScript 6 compatibility checks, followed by Biome 2.5.15.

Run all frontend checks:

```bash
bun --bun run lint
```

To run Biome directly, use:

```bash
bun --bun run lint:biome
```

TypeScript 7 provides the `tsc` command through the `@typescript/native` alias. The
`typescript` alias provides the TypeScript 6 API required by Next.js, and `tsc6`
powers `bun --bun run typecheck:compat`. Both type checks remain enabled.

`biome.json` retains the existing source-file scope and enables the equivalents
of the previous unused-variable and React Hooks checks, with separate checks for
unused imports and function parameters. Biome ignores underscore-prefixed unused
bindings; its diagnostics are not identical to ESLint's. The lint command fails on
warnings as well as errors (previously ESLint allowed up to 20 warnings). Formatting
and import organization are disabled; this migration does not reformat the UI.

### Configure the gRPC endpoint

1. Launch the app with `bun --bun run tauri dev`.
2. Open **Settings** (top-right icon or navigate to `/settings`).
3. Provide the gRPC URL, access token, and optional proxy URL.
4. Click **Update** to save and trigger a reconnect with the new credentials.

## Building

Desktop builds embed the static Next.js output inside the Tauri shell, while mobile builds reuse the generated Android project under `src-tauri/gen/android`.

```bash
# Desktop installers (macOS, Windows, Linux)
bun --bun run tauri build

# Android debug build / deploy (requires Android SDK)
bun --bun run tauri android dev
```

Consult the Tauri mobile documentation for iOS setup; configuration entries are already present for mobile capability profiles.

## Data & Security

- Settings persist in a SQLite database created in the OS-specific application data directory.
- Access tokens are encrypted with ChaCha20-Poly1305; encryption keys are stored through keyring-core 1 and the platform-specific native store.
- macOS keeps the login keychain, iOS keeps protected storage, Windows keeps the existing credential target, and Linux keeps the `keyring-rs:<account>@<service>` keyutils description. Linux keyutils credentials retain their existing lifetime (they are not persistent disk storage).
- Android reads the named `net.withforesight.roomtemp.credentials.v1` store first. If its key is absent, it copies the legacy key, verifies the saved value, and retains the legacy credential. Existing installations can still use that legacy key when rolling back; keys created by a fresh installation in the named store are not copied back for downgrade compatibility.
- Missing keys for saved ciphertext, inaccessible storage, malformed keys, and failed migration verification are errors. The app does not silently generate a replacement key for existing encrypted settings. Failed settings saves show an error and do not reconnect.

To explicitly verify macOS provider compatibility using an isolated test credential:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --lib macos_legacy_provider_preserves_credential_identity -- --ignored
```
- gRPC traffic runs over TLS with the host derived from the configured endpoint; bearer tokens are injected with a tonic interceptor.

## Architecture Overview

- **Frontend:** Pages use injected application operations through `useFrontend`. Pure typed reducers manage connection, graph and form transitions; Hooks own asynchronous execution and React lifecycle.
- **Application boundary:** `src/usecases/ports` defines connection, domain graph-data and settings contracts. Use cases import no React, Tauri, protobuf or mocks.
- **Composition:** `src/frameworks/frontendServices.ts` selects Tauri or WebDriver adapters and the clock. Tauri adapters own IPC payloads and protobuf/legacy JSON decoding. Presenters own chart conversion and safe display messages.
- **Concurrency:** The provider owns connection admission and the settings workflow. Generation, graph epoch and lifecycle guards reject stale work synchronously, including the interval before React rerenders. Saving clears graphs; a failed save restores connection availability without reconnecting or refetching. Successful persistence triggers reconnection even if the form has unmounted. Edited fields take precedence over delayed settings loads.
- **Bridge:** `@tauri-apps/api` `invoke` calls proxy to Rust commands for establishing connections and retrieving protobuf payloads.
- **Backend:** Tauri commands manage connection pooling, execute Diesel-backed use cases, and stream serialized protobuf responses to the UI.

## Project Layout

```text
.
├── src/                     # Next.js application (App Router)
│   ├── app/                 # Pages, layout, global styles
│   ├── components/          # UI building blocks
│   ├── data/                # Time-series adapters and fixtures
│   ├── domain/              # Frontend domain models
│   ├── frameworks/          # Composition, IPC error mapping and proto decoding
│   ├── interfaces/          # Adapters, presenters, React hooks/reducers/contexts
│   ├── lib/                 # Shared utilities
│   └── usecases/            # Application operations and repository ports
├── tests/frontend/          # Application, adapter, decoder and Hook tests
└── src-tauri/               # Tauri 2 Rust workspace
    ├── src/                 # Commands, controllers, repositories, infrastructure
    │   └── migration/       # Diesel migrations
    ├── capabilities/        # Desktop/mobile capability profiles
    ├── permissions/         # IPC permission sets for custom commands
    ├── gen/                 # Generated mobile project artifacts
    └── icons/               # Platform icon assets
```

## Troubleshooting

- **gRPC transport errors:** verify endpoint URL, TLS certificate, proxy settings, and access token validity.
- **Keychain/Keystore issues:** ensure the app has permission to access secure storage on the host OS (macOS and mobile may require explicit consent).

Happy hacking!

**Continuous Integration**: A GitHub Actions workflow runs on push and pull requests. It performs linting, frontend unit/Hook tests, the Tauri WebDriver end-to-end suite, Rust unit tests, and generates an `lcov.info` coverage artifact (uploaded on CI runs). See `.github/workflows/ci.yml` for details.

### Verified gRPC connectivity

The application checks `grpc.health.v1.Health/Check` for
`tempgrpcd.v1.TempgrpcdService` using the same channel, TLS/proxy configuration
and Bearer token as graph requests. Channel creation and the health check share
a 10-second timeout. Only a `SERVING` response enables **Fetch Data**.
This confirms authenticated gRPC availability; Redis readiness and the presence
of data are checked only when fetching. Graph RPCs also have a 10-second limit.

Fetching remains manual. Connection checks and settings saves disable fetching;
reconnecting or changing settings clears old charts and ignores obsolete
requests. A failed settings save preserves the previous settings and connection.
Empty results display **No data for the selected period**. Authentication,
unavailability and timeout errors clear verified connectivity; use **Retry
Connection** to check again. Other graph errors allow another fetch.

Deploy a tempgrpcd server with authenticated Health Check and Watch before
distributing this client. The older public Health API accepts incorrect tokens
and cannot verify credentials. Configure monitoring probes with Bearer
authentication and verify valid-token success and invalid-token rejection
through the actual TLS/proxy route before rollout. Roll back the client before
restoring a server with public Health. No automatic reconnect or legacy Health
fallback is provided.

To repeat the real-server integration test, start an isolated tempgrpcd and Redis,
then set `ROOMTEMP_TEST_GRPC_ENDPOINT` to its local HTTP endpoint and
`ROOMTEMP_TEST_GRPC_TOKEN` to its disposable test token. Run
`cargo test --manifest-path src-tauri/Cargo.toml updated_server_accepts_valid_credentials_and_rejects_invalid_credentials -- --ignored`.
This test covers the real Health/data RPC combination, not the production
TLS or proxy route. Do not use production credentials in this fixture.
