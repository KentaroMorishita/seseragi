# Seseragi / interface reset (visual prototype)

This directory is intentionally **static HTML and CSS**. It is not an attempt to preserve or patch the old Docs implementation.

## Scope of this first pass

- Home: brand, hero, one code sample, three purposeful routes.
- Docs: editorial reading hierarchy without duplicate card grids.
- Article: readable code, local table of contents, reference detail when wanted.
- Mobile: responsive layout and native menu.

We deliberately postponed **all content migration, code generation, mass API references, test-suite restructuring and compiler integration**. The included example is a UI specimen, not a formally executed compiler fixture.

The Vercel configuration change in this branch serves these static files **only for the prototype preview**, leaving `main` and the existing production Docs untouched. Review visual quality first; if accepted, integrate the rendering/SSG architecture in a separate follow-up.

See Issue #601 for the original 61 design decisions. New concept/design work does not inherit old article or route preservation obligations.
