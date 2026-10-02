# Contributing

## Checks

CI runs these on macOS, Linux, and Windows. Run them before you push:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-file-size.sh
```

## Crate boundaries

Dependencies point one way: `captain-app` → `captain-ui` → `captain-core` ← `captain-docker`. `captain-ui` also uses `captain-terminal`. `captain-host` and `captain-kube` depend on `captain-core`, and only `captain-app` and `captain-cli` wire them in.

- `captain-core` must not depend on GPUI or bollard. Put logic here when you can, because it is the easiest code to test.
- `captain-docker` converts bollard types into `captain-core` types. Bollard types do not leave this crate.
- `captain-host` is Captain Engine (the Lima VM) behind the `EngineHost` trait. `captain-kube` holds the kube-rs code.
- `captain-ui` talks to the engine only through the `Engine` trait.

## File size

Small files are easier to read and review.

- Keep files under 300 lines. If a file grows past that, split it before you add more.
- The hard limit is 500 lines. `scripts/check-file-size.sh` fails CI above it.
- Put one concept in each file: one view, one model, or one trait with its impl.
- `lib.rs` and `mod.rs` only declare modules and re-export items.
- Put unit tests in a sibling file: `foo.rs` declares `#[cfg(test)] mod tests;` and the tests live in `foo/tests.rs`.
- Split a large view into sub-components, each in its own file.

## Tests

CI runs every test on three OSes, so keep the suite small and fast.

- Write one focused test per behavior. Put the edge cases of that behavior in the same test.
- Do not test derives, plain getters, constant labels, or formatting that a higher-level test already checks.
- Do not repeat a `captain-core` test in `captain-docker`, `captain-host`, or `captain-ui`. Test only what that layer adds.
- Always test parsing, merge rules, and safety checks: privileged commands, deletion, `daemon.json`, migration, switch-over, snapshots, and locks.
- Live tests that need an engine or a VM go in `crates/*/tests/live_*.rs` and stay `#[ignore]`d. Run them by hand with `cargo test -- --ignored`.

## Docs

- A new feature starts with a spec in `docs/features/`. Copy the format of an existing one.
- An architecture decision gets an ADR in `docs/adr/`. Number it with the next free number.
- A change that users see updates the [user guide](docs/guide/README.md), and a hand check goes in [docs/testing.md](docs/testing.md).
- Do not edit the generated files in `docs/reference/`. Regenerate them instead:
  - `settings.md` and `settings.schema.json`: `CAPTAIN_BLESS=1 cargo test -p captain-core reference`.
  - `cli.md`: `cargo run -p captain-cli -- docs cli > docs/reference/cli.md`.
  - `mcp.md`: `cargo run -p captain-cli -- docs mcp > docs/reference/mcp.md`.
