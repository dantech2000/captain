# ADR 0001: Use GPUI Kit for the UI

- Status: Accepted
- Date: 2026-09-28

## Context

Captain is a native desktop app for macOS, Linux, and Windows, written only in Rust. The UI needs fast tables for hundreds of containers, streaming log views, and later a terminal.

GPUI is the GPU-accelerated UI framework from Zed. On its own it has no ready-made widgets. GPUI Kit (`longbridge/gpui-kit`, formerly `gpui-component`) adds more than 60 styled components: data tables with virtual scrolling, sidebars, dock layouts, themes, and a code editor. Longbridge ships a commercial app with it.

## Decision

Use the `gpui-kit` crate as the only UI dependency. It pins a matching GPUI version and re-exports it, so we do not pin GPUI ourselves.

## Consequences

- GPUI Kit is pre-1.0 and releases often. Minor version bumps can break the API. Upgrade on purpose, in its own commit.
- GPUI has its own async executor, not tokio. See ADR 0002.
- Only `captain-ui` and `captain-app` depend on `gpui-kit`.
