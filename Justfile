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
    printf 'Installed %s\n' "$HOME/.local/bin/rs-prompt"
    printf 'Add to .zshrc: eval "$(%s init zsh)"\n' "$HOME/.cargo/bin/rs-prompt"
