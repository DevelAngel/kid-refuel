arch := "aarch64"
libc := "musl"

# --- test ---

[group('test')]
test:
    cargo test --workspace

# --- build test ---

# Full workspace build check (catches cross-crate issues)
[group('lint')]
check:
    echo "{{os()}}"
    cargo check --workspace --all-targets

# --- debug build ---

[group('build-debug')]
debug-native:
    cargo leptos build

# --- release build ---

[group('build-release')]
release-native:
    cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"

[group('build-release')]
release-cross:
    env \
    LEPTOS_BIN_CARGO_COMMAND=cross \
    LEPTOS_BIN_TARGET_TRIPLE={{arch}}-unknown-linux-{{libc}} \
    cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"

[group('build-release')]
release-frontend:
    cargo leptos build --frontend-only --release --lib-cargo-args="--locked"
