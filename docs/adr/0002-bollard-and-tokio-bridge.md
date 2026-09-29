# ADR 0002: Use bollard, and run it on a private tokio runtime

- Status: Accepted
- Date: 2026-09-28

## Context

Captain talks to the Docker Engine API over a Unix socket, a Windows named pipe, or TCP. `bollard` is the most complete async Rust client for this API. It depends on tokio.

GPUI runs its own executor. Tokio futures that do I/O panic if no tokio runtime is active, so GPUI tasks cannot await bollard calls directly.

We considered two other options:

- Our own HTTP client on hyper. This is a lot of work, and it still needs tokio.
- Call the `docker` CLI and parse JSON. This is slow, and it needs the CLI installed.

## Decision

1. `captain-docker` owns a multi-thread tokio runtime. `DockerEngine` spawns every bollard call on that runtime.
2. Single results come back as the tokio `JoinHandle`, which any executor can await. Streams come back through a `futures::channel::mpsc` channel. Neither needs a tokio runtime on the awaiting side, so a GPUI task can await them.
3. The `Engine` trait in `captain-core` exposes only these runtime-neutral futures and streams.

## Consequences

- Tokio does not leak into `captain-core` or `captain-ui`.
- A fake engine for tests needs no tokio.
- The app runs two thread pools, GPUI's and tokio's. This costs a few threads, which is fine for a desktop app.
