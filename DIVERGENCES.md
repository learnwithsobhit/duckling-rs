# Intentional divergences from upstream Duckling

The Rust port is byte-exact with upstream Haskell Duckling on every recorded fixture. This file lists every place
where behaviour intentionally differs. Parser bug fixes (Phase 7) are added here, each with a regression test.

## HTTP server (`duckling-server`) — transport-level only

Response bodies are byte-identical to the Haskell example server (verified on ~12k randomised requests with
`tools/diff_server.py`). Differences are limited to the HTTP envelope:

| # | Area | Upstream (Snap) | Rust (axum/hyper) | Why |
|---|------|-----------------|-------------------|-----|
| S1 | Reason phrase for 422 | `Bad Input` | `Unprocessable Entity` | hyper emits canonical phrases |
| S2 | `Server` header, chunked encoding | Snap sets `Server: Snap/…` | absent | not part of the API contract |
| S3 | Invalid `reftime` when a Time entity is resolved | connection dropped (lazy exception) | HTTP 500 with a message | clients get an error rather than a reset |
| S4 | `/targets` key order | Haskell `HashMap` order | sorted by language code | JSON objects are unordered |

## Parser bug fixes

None yet (Phase 7).

## Fixture allowlist

CI fails if any `fixtures/*.jsonl` file changes unless this document names that file
(basename or path). Phase 7 bug fixes list the regenerated fixtures here.
