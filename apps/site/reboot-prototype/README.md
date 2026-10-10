# Seseragi UI reset — second prototype

This is a deliberately small, independent static HTML/CSS prototype, not the legacy Docs implementation.

## Design constraints actually applied

- Use the existing Seseragi logo asset (`assets/brand/extension/logo-light.svg`), unchanged.
- Home is the language's front door: one clear promise, two working actions, one verified-source code illustration **below** the hero.
- No invented wave logo, serif rebranding, saturated red, decorative circles, tilted code, card wall or excessive mobile top whitespace.
- Docs index is a navigable entrance, not a duplicate list of category cards.
- Article is for reading: accessible header/menu, readable line lengths, working in-page anchors and a specific code example.
- All three screens have layouts for desktop and 360px / 390px mobile, using the same small responsive stylesheet.
- Code example is adapted from the repository's FizzBuzz sample (`examples/samples/fizzbuzz/main.ssrg`) and links to the original source.
- This stage does **not** create a new compiler, API generator, mass corpus or heavyweight test-suite.
- Existing site production and Playground are unchanged. Branch-only Vercel config serves static prototype for visual review.

The prototype currently shows Japanese copy only; Japanese/English authoring and typed Seseragi SSG integration follow after the UI is evaluated.
