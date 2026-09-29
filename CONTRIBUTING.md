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

Dependencies point one way: `captain-app` → `captain-ui` → `captain-core` ← `captain-docker`.

- `captain-core` must not depend on GPUI or bollard. Put logic here when you can, because it is the easiest code to test.
- `captain-docker` converts bollard types into `captain-core` types. Bollard types do not leave this crate.
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
