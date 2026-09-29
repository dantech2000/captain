# ADR 0003: A layered Cargo workspace with small files

- Status: Accepted
- Date: 2026-09-28

## Context

Captain will grow feature by feature for a long time. We want each layer to be testable alone, fast incremental builds, and code that is easy to read.

## Decision

Split the code into four crates:

| Crate | Depends on | Holds |
|-------|------------|-------|
| `captain-core` | nothing heavy | models, the `Engine` trait, state stores |
| `captain-docker` | core, bollard, tokio | the Docker `Engine` implementation |
| `captain-ui` | core, gpui-kit | views |
| `captain-app` | all of the above | `main`, window, menus |

Keep files small:

- Soft limit of 300 lines per file. Hard limit of 500 lines, which CI enforces with `scripts/check-file-size.sh`.
- One concept per file.
- `lib.rs` and `mod.rs` hold only module declarations and re-exports.
- Unit tests live in a sibling `tests.rs` file, not at the bottom of the source file.

## Consequences

- A change to a view does not rebuild the Docker layer.
- Core logic has plain unit tests with no window and no daemon.
- The crate boundaries must stay clean. A bollard type in `captain-ui` is a bug.
- Many small files means more `mod` declarations. We accept that.
