# Seseragi brand assets

This directory is the canonical home for the Seseragi visual identity.

## Canonical files

| Asset | Intended use |
|---|---|
| `source/seseragi-symbol.svg` | Editable vector symbol source |
| `extension/logo.png` | README / documentation horizontal lockup |
| `extension/icon.png` | VS Code Marketplace and square product icon (128×128) |
| `favicon/favicon-16x16.png` | Small browser favicon export |
| `manifest.json` | Canonical paths and brand colors |

`extensions/seseragi/images/` contains distribution copies consumed by the VS Code package. Update the canonical files here first, then regenerate or copy the distribution assets.

## Usage

- Keep the symbol proportions unchanged.
- Do not recolor individual circuit paths independently.
- Keep clear space around the symbol of at least one terminal-node diameter.
- Prefer the symbol at small sizes. Use the horizontal lockup from about 240 px wide.
- Use `#001521` for the primary wordmark, `#008e9d` → `#57d9b4` for the symbol gradient, and `#08100e` as the primary dark surface.
- Raster exports are generated assets and should not be edited directly.

## Rollout

The root README and VS Code extension use these assets in this change. Dark-background lockups, monochrome exports, social preview, and the remaining favicon sizes are tracked by the brand rollout issues.
