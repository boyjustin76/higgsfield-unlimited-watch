# Contributing

Start with a reproducible observation: tool version, OS, UI language, command and expected/actual state. Include sanitized synthetic HTML when selectors change. Never post cookies, sessions, private media URLs, company images or real prompts.

## Development

Rust is the primary CLI; the retained Python helper uses only the standard library. A skill-folder-only copy contains Python; build/install the Rust executable separately. See [Installation](README.md#installation) and the [compatibility matrix](RUST_PORT.md).

```bash
python -m unittest discover -s tests -v
cargo fmt --check
cargo test --locked
cargo build --locked --release
python tests/compare_demo.py --rust target/release/huw
```

Use `target/release/huw.exe` for comparison on Windows. These are offline tests; they do not establish live browser support. See [validation](docs/validation.md).

## Scope

Useful contributions: field-verified pending/failure status variants, second-machine setup reports, reconnect tests and adapters for other hosts. Mark synthetic coverage separately from real browser observations. Avoid arbitrary token-savings percentages or claiming support based only on a unit test.

Do not add automatic credit fallback, implicit retry after uncertain submission, multi-account scheduling or token extraction to this project. Discuss new scope in an issue first. A future MCP adapter should reuse the existing local interfaces rather than change the entitlement assumptions.
