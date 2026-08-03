# Seseragi brand assets

This directory is the canonical home for the Seseragi visual identity.

## Canonical distribution files

| Asset | Intended use |
|---|---|
| `extension/logo-light.jpg` | README and documentation on light surfaces |
| `extension/logo-dark.jpg` | README and documentation on dark surfaces |
| `extension/icon.png` | VS Code Marketplace and square product icon (128×128) |
| `favicon/favicon-16x16.png` | Small browser favicon export |
| `manifest.json` | Canonical paths and brand colors |

`extensions/seseragi/images/` contains distribution copies consumed by the VS Code package. Update the canonical files here first, then copy the verified exports.

## Safety and usage

- These files are reviewed raster exports using standard JPEG or truecolor PNG.
- Do not replace them with indexed or palette PNG files.
- Do not automatically trace the generated artwork into SVG; redraw a future vector master deliberately.
- Keep the symbol proportions unchanged.
- Keep clear space around the symbol of at least one terminal-node diameter.
- Prefer the symbol at small sizes. Use the horizontal lockup from about 240 px wide.
- Use `#001521` for the primary wordmark, `#008e9d` → `#57d9b4` for the symbol gradient, and `#08100e` as the primary dark surface.
