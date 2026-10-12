# Real npm declaration corpus

This fixture pins six published packages and their registry integrity in `bun.lock`:
nanoid (function/optional/generic), mitt (generic/callback/structure), p-limit
(callback/async), eventemitter3 (class/namespace/overload), type-fest Promisable
(generic/union/Promise) and Node-only globby (host/async). The case list selects
published declaration files, not handwritten wrappers or invented replacement APIs.

Install explicitly into this fixture or an isolated temporary directory:

```
bun install --frozen-lockfile --ignore-scripts --cwd examples/spec/fixtures/semantic-interop-schema-1/npm
bun scripts/check-real-package-interop.ts
```

Use `--node-modules /absolute/path/to/node_modules` to inspect the same locked
versions from an isolated install. The script builds the source-neutral report
example once, then runs the normal declaration frontend and projection for every
package. It records registry SHA-512, actual declaration SHA-256, version, exported
TS checker roots, shared IR/report, generated output or explicit diagnostics. A
missing TS public root marks the package `partially-interpreted-public-surface`;
zero interpreted roots never claim compatibility. Generated opaque type declarations
are not proof of executable constructor/method support. Snapshot updates require
explicit `--write` after reviewing classification and reason changes.

Remaining semantic/runtime policy requirements are routed to their existing #552
leaf (generic/structural/receiver/callback/async/export/binding/target), with package,
locked version, digest, exact declaration and report reason. New leaf creation is
reserved for reproducible gaps outside an existing leaf's scope. No package-name
conditionals are added to the frontend, shared IR, projector or runtime.

The pinned base64-js@1.5.1 fixture under
`apps/playground/tests/fixtures/interop-base64-js-1.5.1` provides the executable SDK
acceptance path: the same published source/declarations and ordinary Seseragi source
are used by Playground and by the native CLI Node/Bun test. The application calls
generated bindings; it has no handwritten wrapper catalog. Native layout adapts the
flat browser `main.ssrg` location to the normal package `src/main.ssrg` location.
