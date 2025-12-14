FROM ubuntu:24.04 AS base

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  curl \
  gnupg \
  lsb-release \
  build-essential \
  pkg-config \
  libssl-dev \
  git \
  libgtk-4-1 \
  libgtk-3-dev \
  libwebkit2gtk-4.1-0 \
  libwebkit2gtk-4.1-dev \
  libgdk-pixbuf2.0-0 \
  libx11-xcb1 \
  libxcomposite1 \
  libxcursor1 \
  libxdamage1 \
  libxfixes3 \
  libxi6 \
  libxrandr2 \
  libxrender1 \
  libatk-bridge2.0-0 \
  libatspi2.0-0 \
  libdbus-1-3 \
  libnotify4 \
  libpangocairo-1.0-0 \
  libpango-1.0-0 \
  libsoup-3.0-0 \
  libglib2.0-dev \
  libglib2.0-0 \
  libsoup-3.0-dev \
  libfontconfig1 \
  libfreetype6 \
  libharfbuzz0b \
  libxkbcommon0 \
  libjavascriptcoregtk-4.1-dev \
  libxcb1 \
  libxcb-dri3-0 \
  libxcb-xfixes0 \
  libxcb-render0 \
  libxcb-shape0 \
  libxcb-shm0 \
  xvfb \
  webkit2gtk-driver \
  python3 \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
  apt-get update && apt-get install -y nodejs && \
  rm -rf /var/lib/apt/lists/*

RUN npm install -g pnpm
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

RUN cargo binstall cargo-chef --locked
RUN cargo binstall tauri-driver --locked

FROM base AS planner

WORKDIR /workspace/src-tauri

COPY ../src-tauri/ /workspace/src-tauri
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /workspace/src-tauri

COPY --from=planner /workspace/src-tauri/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY ../src-tauri/ /workspace/src-tauri
RUN cargo build --release

WORKDIR /workspace
COPY . /workspace
