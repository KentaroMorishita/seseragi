# Checked native project linking

Tracking: #837 under #812 / #809.

The experimental Rust commands accept a package root as well as one source file:

```sh
seseragi experimental build-native \
  examples/spec/fixtures/rust-backend-projects/linked \
  --entry main --out-dir target/native-project
env PATH="" target/native-project/program
```

`seseragi.lock` must already exist and pass the ordinary strict project lock
validation. Update it explicitly with `seseragi lock update <package-root>`.
The entry module comes from `[run].entry`; `--entry` selects an unconstrained
Unit function in that exact module. A function with the same short name in a
dependency cannot become the entry. Pure entries print the native inspection
result; synchronous Effect entries execute the ordinary native plan.

## Cargo builds

The CLI downloads its pinned Rust toolchain on the first native build. It uses
the version in `rust-toolchain.toml`, with Cargo and the host standard library
from the same release. The bootstrap uses the official rustup 1.28.2 installer
over HTTPS and checks its SHA-256 before running it. The first download needs
`curl` and internet access. Later builds use the cached tools directly and do
not contact the distribution server. Rust does not need to be on `PATH`.

The default cache locations are:

| Platform | Directory |
| --- | --- |
| macOS | `~/Library/Caches/seseragi/native/` |
| Linux | `$XDG_CACHE_HOME/seseragi/native/`, or `~/.cache/seseragi/native/` |
| Windows | `%LOCALAPPDATA%/seseragi/cache/native/` |

Set `SESERAGI_NATIVE_CACHE` to use another writable directory. Toolchains are
separated by Rust version and host OS/CPU. Downloads are staged under a lock;
an interrupted installation can be retried without using partial tools. The
CLI checks the Rust and Cargo versions, host and standard library before use.
It reports incomplete or incompatible cached installations with repair steps.

To prepare an offline machine, copy a complete matching Rust installation and
set `SESERAGI_NATIVE_TOOLCHAIN` to its root, containing `bin/` and `lib/rustlib/`.
This override uses the same version and host checks. Cargo's dependency cache
stays under `SESERAGI_NATIVE_CACHE/cargo/`. Native builds do not use the caller's
`CARGO_HOME`, Rust version selection, compiler or compiler wrappers.

The system linker and SDK remain separate prerequisites. On macOS, install
Xcode Command Line Tools with `xcode-select --install`. On Linux, install the
C compiler and development libraries, such as Debian/Ubuntu's `build-essential`.
On Windows, install Visual Studio Build Tools with Desktop development with C++
and the Windows SDK. Build failures include the underlying compiler diagnostic
and platform-specific setup steps when the linker or SDK is unavailable.

Native builds keep a Cargo project under `.seseragi/native/` in the package
root, or beside a standalone source file. The directory is stable for each
input and entry function. It contains `Cargo.toml`, `Cargo.lock`, `program.rs`
and a copy of the native runtime crate embedded in the CLI. Unchanged files
keep their modification times. Cargo reuses dependencies after program edits
and rebuilds the runtime when its source changes.

Allow roughly 84 MB per generated project for a hello-world development build,
mostly for Cargo's build outputs. The size varies with the toolchain and debug
settings, and grows when adding dependencies or building another profile.
The project template's `.gitignore` excludes `.seseragi/` from version control.

Build outputs normally stay in `.seseragi/native-target/` beside those
projects, or in the user cache on Windows. When
`CARGO_TARGET_DIR` is set, each project uses a separate directory under
`$CARGO_TARGET_DIR/seseragi-native/`. Development builds use optimization level
0; `--profile release` uses level 3. Both ordinary native commands and
`experimental build-native` accept the profile selection. A failed build leaves
the previously published executable, Rust source and metadata intact.

Each project's build outputs are separated by runtime version and source,
Rust version, host, dependency features, linker settings, build profile and
Cargo/Rust build environment settings. Editing only the program can reuse its
runtime build.

Packages can declare Rust adapter dependencies and linker inputs in
`seseragi.toml`:

```toml
[tool.native]
libraries = ["sqlite3"]
search-paths = ["native/lib"]

[tool.native.dependencies]
adapter = { path = "native/adapter" }
```

Dependencies use Cargo's dependency syntax. Paths are relative to the package
root. `libraries` uses Cargo's `rustc-link-lib` syntax, including optional
`static=` or `dylib=` prefixes. `search-paths` lists native library directories.
These settings make dependencies available to generated Rust and future native
adapters; they do not add a Seseragi foreign-function interface.

Cargo always builds offline. Registry dependencies must already be available
in the managed Cargo cache; path dependencies and programs without external
dependencies need no network access. The generated Cargo lockfile is retained
between builds.

## Shared frontend, separate target

`compile_core_project` and `compile_core_local_project` reuse source discovery,
package scope, export visibility, the checked module graph and dependency-ordered
semantic analysis. They return a `CoreProject` of `CoreCompilation` values before
TypeScript output planning or emission. The test compares each module's CoreIr
and typed interface with the ordinary TS pipeline and proves the CoreIr path
also accepts paths that the TS output planner rejects.

The Rust linker retains canonical module / function / constructor symbols. It
indexes user instance identities by their defining checked module; identities
for standard traits are not parsed as module names. Imported evidence must name
that exact provider and identity, with nested evidence passed to the same
factory. No short-name search or standard dictionary fallback is used. Imported
factories are not newly cached or moved. Existing source application stages,
returned-function arity, evaluation order and closure captures remain intact.
The metadata records the original modules in dependency order.

## Bounded acceptance

The fixture contains five modules in two path packages, including two files
named `lib.ssrg`, a reexport facade, nested generic dictionaries, an HKT
constraint, user Add / Show instances and a returned function. It also uses a
Ref created in another module and a retained callback that captures it. Typed
failure recovery crosses the module boundary and two scoped finalizers run in
LIFO order before the caller resumes.

Tests compare native with both TS profiles, require final managed-live=0 for
the Effect entry, and check an explicit pure entry. Counterfeit imported owner /
local evidence, missing or reordered graph nodes, wrong entry-module selection,
private and opaque access, missing / stale locks and unsupported imported source
are rejected. Target diagnostics carry the logical owner separately from the
portable CoreIr source name; the CLI maps it to the loaded physical source and
keeps the original byte range, even when basenames coincide.

Top-level initialization uses shared CoreIr analysis of source-import DFS
postorder, distinct from checking order, and source order within each module. Monomorphic values, structural and algebraic data,
closures, and cold synchronous Effect plans are evaluated once before entry.
Function references remain callable before value initialization; delayed reads
inside functions use the stored global value. Lexical bindings take precedence,
and module-local CoreIr spellings resolve only against their original owner.

Global values remain roots through entry execution and all scope finalizers.
Both pure and Effect entry runners release partial initialization on Defect,
then global roots and dictionaries before collecting unreachable Ref cycles.
TypeScript keeps source initialization edges for unused imports; release
application pruning retains eager initializers and their dependencies, including
through empty facades. Unimported modules remain outside initialization. The
explicit CoreIr API requires the entry dependency graph.

The `globals` fixture covers a four-module, two-package diamond, a selected
nested dictionary captured by a global closure, partial application, tuple
patterns, cold plan replay, and a Ref callback used during finalization.

Polymorphic `CoreBinding` values and constrained values without source
parameters are rejected before output. Constrained callable aliases already
eta-expanded by the shared frontend retain their parameters and evidence ABI.
General polymorphic value initialization remains
unsupported, including unused bindings. Derived dictionaries, supertraits,
unimplemented standard operations, async / providers, foreign interop and
general program WASM retain their existing boundaries. Unsupported code in any
linked source fails before Rust, metadata or executables are written. This slice
does not complete #812 or imply TS backend parity.

No new performance optimization is claimed by linking. The earlier TS/native
measurements and shared CoreIr candidates in #816 remain separate evidence.

## Validation before canonical distribution

The initialization slice uses scoped backend, process runtime, and CLI checks.
Backend coverage includes 24 runtime, 29 generated-program and three project
ownership tests; the process runtime includes text / JSON initialization failure
in normal and bundled execution. CLI comparisons cover the existing single-file
corpus, both multi-package projects, eager initialization Defects, and shared
initialization diagnostics. Canonical artifact freshness, exact-head integration,
official Release, dogfood and installed CLI acceptance are tracked on #839 and
#841. Local checks alone do not establish distribution acceptance.

The ordinary five-module TS build also matches the fixture in development and
release on fresh Bun 1.3.11 and Node 24.14.0 processes (four executions), with an
empty runtime PATH. This validates observable behavior, not a speed improvement.

Initialization parity also covers process exit behavior. A top-level Defect
stops before entry and returns exit 70. The TS process runner handles rejection
of its literal application imports before provider setup, including structured
Defect diagnostics, while retaining seed-before-import ordering and bundling.
Browser startup is unchanged. Native teardown is additionally checked after
partial initialization, including a 100,000-node graph and Ref cycle on a
256 KiB stack.
