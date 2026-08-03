# Seseragi brand assets

This directory is the canonical source for the Seseragi visual identity.

## Files

| Asset | Intended use |
|---|---|
| `source/seseragi-symbol.svg` | Editable vector symbol source |
| `source/seseragi-lockup.svg` | Editable horizontal symbol + wordmark source |
| `source/seseragi-symbol-mono.svg` | One-color symbol; inherits `currentColor` |
| `png/seseragi-lockup-light.png` | Light-background README / documentation hero |
| `png/seseragi-lockup-dark.png` | Dark-background README / documentation hero |
| `png/seseragi-symbol-*.png` | Raster symbol exports for UI surfaces |
| `png/seseragi-symbol-mono-*.png` | One-color raster exports |
| `favicon/` | Browser favicon and Apple touch icon |
| `social/seseragi-social-preview.png` | GitHub social preview source (1280×640) |
| `extension/icon.png` | VS Code Marketplace icon source (128×128) |
| `extension/logo.png` | VS Code extension README lockup |

## Usage

- Keep the symbol proportions unchanged.
- Do not recolor individual circuit paths independently.
- Use the light lockup on white or very light backgrounds.
- Use the dark lockup on `#08100e` or similarly dark backgrounds.
- Keep clear space around the symbol of at least one terminal-node diameter.
- Prefer the simplified symbol at 16–64 px. Use the horizontal lockup from 240 px wide.
- `extensions/seseragi/images/` contains generated distribution copies. Edit the canonical files here, then regenerate those copies.

## Source

The SVG files are the editable source for subsequent geometric cleanup. PNG files are generated exports and should not be edited directly.
