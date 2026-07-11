check:
    cargo fmt --check
    cargo test
    cd tests && uv run isort --check-only .
    cd tests && uv run black --check .
    cd tests && uv run ruff check .
    cd tests && uv run pytest -q

fmt:
    cargo fmt
    cd tests && uv run isort .
    cd tests && uv run black .

build:
    cargo build

release:
    cargo build --release

build-releases version=`cargo pkgid | sed 's/.*#//'`:
    rm -rf dist
    RUSTFLAGS="-C strip=symbols" cargo build --release --target x86_64-unknown-linux-gnu
    RUSTFLAGS="-C strip=symbols" cargo build --release --target x86_64-unknown-linux-musl
    RUSTFLAGS="-C linker=rust-lld -C strip=symbols" cargo build --release --target aarch64-unknown-linux-musl
    RUSTFLAGS="-C strip=symbols" cargo zigbuild --release --target aarch64-unknown-linux-gnu
    for target in x86_64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-gnu aarch64-unknown-linux-musl; do dir="dist/rs-prompt-v{{version}}-$target"; mkdir -p "$dir"; cp "target/$target/release/rs-prompt" "$dir/"; tar -C dist -czf "dist/rs-prompt-v{{version}}-$target.tar.gz" "rs-prompt-v{{version}}-$target"; done

install:
    cargo install --path .
    printf 'Installed %s\n' "$HOME/.local/bin/rs-prompt"
    printf 'Add to .zshrc: eval "$(%s init zsh)"\n' "$HOME/.cargo/bin/rs-prompt"
    printf 'Add to config.fish: %s init fish | source\n' "$HOME/.cargo/bin/rs-prompt"
