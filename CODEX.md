# CODEX.md

## Project Shape

`rs-prompt` is a Rust shell prompt generator with source-built configuration.
Keep the crate at the repository root.

- Rust implementation: `src/`
- Shell integration templates: `scripts/`
- Python end-to-end tests: `tests/`

## Implementation Direction

The project follows a suckless-style model:

- No runtime configuration files.
- Configuration changes happen in source code and require rebuilding.
- Keep prompt logic small and direct.
- Avoid unnecessary filesystem IO in prompt generation.
- Do not add dependencies unless they clearly reduce complexity.

## Validation

Run:

```sh
just check
```

Generated outputs should stay untracked:

- `target/`
- `.pytest_cache/`
- `tests/.venv/`
- `tests/.pytest_cache/`
- `tests/__pycache__/`
