# Change Log

## [0.4.0] - 2026-08-02

- Unified the CLI, LSP, runtime, WASM, and VS Code extension on one toolchain
  version source.
- Added commit, channel, target, and dirty-build metadata to CLI and LSP
  version output.
- Added reproducible release artifact names and a tag-validated GitHub Release
  workflow.
- Packages each platform's CLI and LSP in one verified native archive, preserves
  Unix executable modes, and publishes a matching SHA-256 checksum.
- Renamed the official VS Code extension to `seseragi-dev.seseragi` and added
  a non-LSP migration stub for the former extension ID.
