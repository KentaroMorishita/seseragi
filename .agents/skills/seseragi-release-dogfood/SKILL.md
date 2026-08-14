---
name: seseragi-release-dogfood
description: Finish Seseragi canonical releases and synchronize the local official CLI, LSP, and VS Code extension. Use after merging user-visible CLI, LSP, or official extension changes, before advancing the issue queue, or when release and local tool versions may have drifted.
---

# Seseragi release dogfood

1. Fetch `origin/main` and read `bun run release:readiness` plus the live GitHub
   Release state for its canonical tag.
2. If the version is pending, wait for `.github/workflows/release.yml` to finish.
   Verify the GitHub Release has native archives, checksums, platform VSIX files,
   runtime, and WASM artifacts. Do not create alternate version logic.
3. On the updated `main`, run `bun run dogfood:sync`. This repository command is
   the only install implementation; do not reproduce its download or handshake
   logic in the skill.
4. Run `bun run dogfood:check` and retain its machine-readable success or error
   in the work log.
5. If `code`, the host target, permissions, network, release, or an artifact is
   unavailable, report the exact failure and list CLI, LSP, or extension items
   still unsynchronized. Never treat a skipped component as success.
