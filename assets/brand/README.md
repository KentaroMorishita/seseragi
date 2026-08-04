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
| `social/seseragi-social-preview.svg` | Editable 1200×630 social preview source |
| `public/brand/seseragi-icon.svg` | Canonical icon copy deployed by Vite |
| `public/brand/favicon-16x16.png` | Browser favicon, 16×16 |
| `public/brand/favicon-32x32.png` | Browser favicon, 32×32 |
| `public/brand/favicon-48x48.png` | Browser favicon, 48×48 |
| `public/brand/favicon.ico` | Multi-resolution 16 / 32 / 48 favicon |
| `public/brand/apple-touch-icon.png` | Apple touch icon, 180×180 |
| `public/brand/seseragi-social-preview.png` | GitHub / Open Graph / social share export |
| `public/brand/site.webmanifest` | Installed web app identity |
| `manifest.json` | Canonical paths and brand colors |

`extensions/seseragi/images/` contains VS Code package copies. `assets/brand/public/` is configured as the Playground Vite public directory, so the deployed web assets remain under the canonical brand tree instead of being copied into the application source. Update the canonical files here first, then render or copy distribution assets from the SVG masters.

## Surface contract

- Root README uses the light / dark horizontal SVG lockups through `<picture>`.
- VS Code uses the canonical icon rendered as a truecolor RGBA PNG because packaged extension README and Marketplace surfaces do not accept every SVG reference.
- Playground and Tour use the canonical icon-only SVG in their compact headers.
- Playground and Tour receive favicon, manifest, Open Graph, and Twitter metadata from the shared Vite HTML transform.
- `public/brand/seseragi-social-preview.png` is the repository social-preview upload candidate and the deployed Open Graph image.
- Surface-specific copies must not alter the shape, proportions, gradient, terminal nodes, or white underlay.

## Safety and usage

- The SVG masters are the source of truth.
- Keep both lockups transparent, identically sized, and identically positioned; only the wordmark color differs.
- The icon uses a white filled hexagon underlay beneath the gradient symbol so the circuit S remains white without raster edge artifacts.
- Keep the symbol proportions unchanged.
- Keep clear space around the symbol of at least one terminal-node diameter.
- Prefer the symbol at small sizes. Use the horizontal lockup from about 240 px wide.
- Use `#011324` for the light-surface wordmark, white for the dark-surface wordmark, and `#009aad` → `#00b4b8` for the symbol gradient.
- Raster exports must be truecolor RGBA PNG, not indexed or palette PNG.
- Browser icon exports are generated from `source/seseragi-icon.svg`; do not retouch individual sizes.
- The social preview is 1200×630 and should preserve the approved dark lockup plus safe margins.

## Review checklist

- Verify root README on GitHub light / dark and mobile / desktop.
- Verify Playground and Tour headers at desktop, portrait mobile, and compact landscape sizes.
- Verify browser tab favicon, Apple touch icon, and installed web manifest icon.
- Verify Open Graph image dimensions and metadata URLs in the built HTML.
- Run the Playground test lane; `brand-assets.test.ts` rejects missing sizes and stale canonical distribution copies.
