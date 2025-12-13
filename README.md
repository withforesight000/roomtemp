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
- **Backend:** Rust 1.82+ (edition 2024), Diesel + SQLite, tonic gRPC client, ChaCha20-Poly1305 encryption.
- **Proto definitions:** TypeScript bindings from `@withforesight/tempgrpcd-protos` on `withforesight000/protobuf-ts` (HTTPS) and Rust bindings from `tempgrpcd-protos` on `withforesight000/protobuf-rust` (HTTPS).

## Requirements

- Node.js 20 LTS or newer compatible with Next.js 15.
- pnpm 9 (`corepack enable` recommended).
- Rust toolchain (`rustup`) and the platform prerequisites listed in the [Tauri 2 documentation](https://v2.tauri.app/start/).
- Git access to GitHub (`withforesight000/protobuf-ts` and `withforesight000/protobuf-rust`) for fetching the shared proto packages.
- A reachable `tempgrpcd` gRPC server that supports TLS and bearer-token authentication.

## Quick Start

```bash
# install dependencies
pnpm install

# run the Next.js dev server and Tauri shell together
pnpm tauri dev
```

## Testing

### Frontend E2E (Playwright)

Playwright-based end-to-end tests live under `src/e2e` and include a small Tauri IPC mock to make tests deterministic.

Install Playwright browsers once:

```bash
pnpm exec playwright install --with-deps
```

Run E2E tests:

```bash
pnpm test:e2e
```

The Playwright config will start the Next.js dev server automatically for the tests.

### Rust unit tests & coverage

Rust unit tests live under `src-tauri`. Convenient npm scripts have been added for running them from the repository root:

```bash
pnpm test:rust           # runs `cd src-tauri && cargo test`
pnpm test:rust:coverage  # runs `cd src-tauri && cargo llvm-cov --workspace --lcov --output-path lcov.info`
```

If you want a quick human-readable coverage output on stdout, run:

```bash
cd src-tauri
cargo llvm-cov --workspace --text --show-missing-lines
```

Note: `cargo-llvm-cov` is required; install it with `cargo install cargo-llvm-cov` if not already available.

`pnpm tauri dev` automatically runs the Next.js development server defined in the Tauri configuration and opens the Tauri window once the frontend is ready.

### Linting

```bash
pnpm lint
```

### Configure the gRPC endpoint

1. Launch the app with `pnpm tauri dev`.
2. Open **Settings** (top-right icon or navigate to `/settings`).
3. Provide the gRPC URL, access token, and optional proxy URL.
4. Click **Update** to save and trigger a reconnect with the new credentials.

## Building

Desktop builds embed the static Next.js output inside the Tauri shell, while mobile builds reuse the generated Android project under `src-tauri/gen/android`.

```bash
# Desktop installers (macOS, Windows, Linux)
pnpm tauri build

# Android debug build / deploy (requires Android SDK)
pnpm tauri android dev
```

Consult the Tauri mobile documentation for iOS setup; configuration entries are already present for mobile capability profiles.

## Data & Security

- Settings persist in a SQLite database created in the OS-specific application data directory.
- Access tokens are encrypted with ChaCha20-Poly1305; encryption keys are stored via the platform keystore or keychain.
- gRPC traffic runs over TLS with the host derived from the configured endpoint; bearer tokens are injected with a tonic interceptor.

## Architecture Overview

- **Frontend:** React components fetch time-series data through a context-provided repository, transform it for visualization, and render with Recharts.
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
│   ├── frameworks/          # Proto decoding helpers
│   ├── interfaces/          # Repositories, presenters, hooks, contexts
│   ├── lib/                 # Shared utilities
│   └── usecases/            # Frontend application services
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

**Continuous Integration**: A GitHub Actions workflow runs on push and pull requests. It performs linting, Playwright E2E tests, Rust unit tests, and generates an `lcov.info` coverage artifact (uploaded on CI runs). See `.github/workflows/ci.yml` for details.
