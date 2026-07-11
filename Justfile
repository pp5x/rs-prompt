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

install:
    cargo install --path .

install-local:
    mkdir -p "$HOME/.local/bin"
    cargo build --release
    cp target/release/rs-prompt "$HOME/.local/bin/rs-prompt"
    printf 'Installed %s\n' "$HOME/.local/bin/rs-prompt"
    printf 'Add to .zshrc: eval "$(%s init zsh)"\n' "$HOME/.local/bin/rs-prompt"
