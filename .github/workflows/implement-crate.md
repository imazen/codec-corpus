---
name: Implement codec-corpus crate
description: Implement the codec-corpus Rust crate from SPEC.md
on:
  workflow_dispatch:
    inputs:
      scope:
        description: "What to implement (e.g. 'full crate', 'download logic only', 'public API only')"
        required: false
        default: "full crate"

engine: claude

tools:
  bash: true
  edit:
  github:

permissions:
  contents: read
  issues: read
  pull-requests: read

safe-outputs:
  create-pull-request: null
---

# Implement the codec-corpus Rust crate

You are implementing a Rust crate called `codec-corpus` based on the specification in `SPEC.md` at the repository root.

## Instructions

1. Read `SPEC.md` thoroughly. It is the single source of truth for what to build.

2. The crate lives in a subdirectory of this repository (NOT the repo root). Create a `crate/` directory with the standard Rust crate layout:
   - `crate/Cargo.toml`
   - `crate/src/lib.rs`
   - `crate/src/download.rs`
   - `crate/README.md`

3. Key requirements from the spec:
   - Tiny crate with minimal dependencies: `dirs`, `tar_light`, `fd-lock`
   - No `serde`, `toml`, `ureq`, `reqwest`, or `gix`
   - Downloads use shell commands: `git` (primary), `curl`/`wget`/`powershell` (fallback)
   - Cache layout: `{cache_root}/codec-corpus/v{major}/`
   - Per-major-version isolation with `.version` file for exact semver tracking
   - `fd-lock` for concurrent access safety
   - Atomic downloads via `.tmp-{pid}-{timestamp}` staging dirs
   - Stale `.tmp-*` cleanup runs ALWAYS (success or failure), not just on success
   - `Corpus::with_cache_root()` overrides the `CODEC_CORPUS_CACHE` env var
   - There is NO `from_local()` method
   - There is NO SHA-256 checksum verification
   - Error type is `#[non_exhaustive]`

4. Use Rust 2024 edition, minimum rust 1.85.

5. Run `cargo check` and `cargo clippy` to verify the code compiles.

6. Create a PR with the implementation. Title: "feat: implement codec-corpus crate from spec"

## Quality bar

- Code should be clean, well-structured, and match the spec exactly
- All public API must match the spec's API section
- Error handling must be robust (no unwrap in library code)
- Cross-platform: Linux, macOS, Windows
