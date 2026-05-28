# Contributing

Contribution standards:

- maintain deterministic compiler behavior
- add tests for every pass and bug fix
- include diagnostics for unsupported patterns
- keep artifact schemas versioned and documented

Before opening a PR:

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets`
