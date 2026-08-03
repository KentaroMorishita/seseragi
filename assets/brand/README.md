# Seseragi brand assets

This directory is the canonical home for the Seseragi visual identity.

## Canonical files

| Asset | Intended use |
|---|---|
| `source/seseragi-logo-light.svg` | Transparent horizontal lockup for light surfaces |
| `source/seseragi-logo-dark.svg` | Transparent horizontal lockup for dark surfaces |
| `source/seseragi-icon.svg` | Transparent icon-only vector master |
| `extension/logo-light.svg` | README and documentation copy for light surfaces |
| `extension/logo-dark.svg` | README and documentation copy for dark surfaces |
| `extension/icon.png` | VS Code Marketplace and square product icon (128×128 RGBA) |
| `favicon/favicon-16x16.png` | Small browser favicon export |
| `manifest.json` | Canonical paths and brand colors |

`extensions/seseragi/images/` contains distribution copies consumed by the VS Code package. Update the canonical files here first, then copy or render the distribution assets from the SVG masters.

## Safety and usage

- The SVG masters are the source of truth.
- Keep both lockups transparent, identically sized, and identically positioned; only the wordmark color differs.
- The icon uses a white filled hexagon underlay beneath the gradient symbol so the circuit S remains white without raster edge artifacts.
- Keep the symbol proportions unchanged.
- Keep clear space around the symbol of at least one terminal-node diameter.
- Prefer the symbol at small sizes. Use the horizontal lockup from about 240 px wide.
- Use `#011324` for the light-surface wordmark, white for the dark-surface wordmark, and `#009aad` → `#00b4b8` for the symbol gradient.
- Raster exports must be truecolor RGBA PNG, not indexed or palette PNG.
