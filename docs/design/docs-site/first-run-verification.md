# First-run verification (#699)

Verified on 2026-10-01. This is implementation evidence and bounded author/agent
self-review, not first-time-reader acceptance or a completed browser review.

## Ownership and sources

- `/docs/first-run/` and `/ja/docs/first-run/`, page ID `docs.first-run`, own
  the minimal install/check/run task. They are standalone pages outside the
  Language and Library reference trees, not new specification coverage.
- `README.md` owns short orientation and links. Its main-branch Cargo install
  is now explicitly the compiler-development route.
- `docs/GETTING_STARTED.md` retains the separate local Web project journey;
  its install step reuses first-run and verifies the existing CLI/Bun rather
  than requiring Rust and reinstalling main unconditionally.
- `docs/RELEASE.md` owns distribution identity, archive selection, checksum
  details, and ABI requirements. The first-run page pins its own tested
  version and includes explicit Linux, macOS and Windows command sections.
  The latter two are marked source-checked, not executed.
- Page prose lives in the `en.ssrg` / `ja.ssrg` modules. Both locales use
  `commands.ssrg` and the existing canonical
  `examples/samples/hello-world/main.ssrg`; there is no second code sample.

Official release inspected:
<https://github.com/KentaroMorishita/seseragi/releases/tag/v0.61.19>, published
2026-09-30. Native archive:
`seseragi-v0.61.19-linux-x64.tar.gz`. Its downloaded `.sha256` verified as:

```text
db522f038b1aa1ac2aab9969f371a0f64a42b823d73e6b482be3104e3dc8e389
```

CLI identity from the downloaded executable:

```text
seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)
```

The release contains CLI, LSP, and `UNICODE-LICENSE`. No separate runtime
download or package install is needed for this example. No compiler/runtime,
Cargo manifest/lock, or Rust toolchain source changed between release commit
`a8641b5a81a4063054e4db4061aedaff86c55a7a` and inspected main
`ab42da841d76d322584fac096c249d586ae539ec`.

## Independent installation and first execution

Executed on Linux x86_64, Debian 13, glibc 2.41, with the published native
archive and Bun `1.3.9+cf6cdbbba`. Bun's official installer was downloaded
from <https://bun.com/install>, inspected, and run with `bun-v1.3.9` in a
fresh temporary home. The official pinned-install instructions are at
<https://bun.com/docs/installation#installing-older-versions>.

The complete command blocks displayed by `commands.ssrg` were then executed
again in a second fresh home, using `bash --noprofile --norc`, an initially
system-only PATH, and fresh TMPDIR. Installation and first run succeeded.
The cloud's existing HTTPS proxy and CA configuration were explicitly retained
for downloads; without the proxy, DNS lookup is unavailable in this cloud.
No repository checkout, Rust/Cargo, npm project, or user shell configuration
was used by the installed program. A separate runtime check used `env -i`
with only HOME, TMPDIR, and the explicitly installed binaries on PATH.

System tools used: bash, curl, unzip, tar, sha256sum, uname, getconf, and
ordinary shell/core utilities. The installer needs internet connectivity.
No administrator install or global npm install was performed.

Inside a new `hello-seseragi` directory, save `main.ssrg` as:

```seseragi
pub effect fn main = println "Hello, Seseragi!"
```

Results:

| Command | Exit | stdout | stderr |
| --- | --- | --- | --- |
| `seseragi lint main.ssrg` | 0 | empty | empty |
| `seseragi run main.ssrg` | 0 | `Hello, Seseragi!\n` | empty |
| `seseragi build main.ssrg` (additional check) | 0 | `Built main.ssrg -> dist\n` | empty |
| `bun run dist/entry.ts` (additional check) | 0 | `Hello, Seseragi!\n` | empty |

The same complete source passed lint/run with the locally built main CLI.
Lint also succeeded with Bun and Rust absent from PATH. Run requires Bun.

Verified recovery diagnostics (all exit 2): missing Bun reports failure to
launch the Bun target adapter; a missing file reports `lint path does not
exist`; `.ssrg.txt` is rejected as the wrong source extension; a private main
reports that main must be public; assigning a String to an Int annotation
reports `SES-T0101` with expected Int and actual String.

## Platform limits

- Linux x64 is the only OS/CPU executed in this verification. The distribution
  contract requires GNU/glibc 2.34 or newer; this run tested glibc 2.41, not
  every distribution or the minimum glibc environment.
- The same release publishes `darwin-arm64`, `darwin-x64`, and `win32-x64`
  artifacts. Their presence was verified. The macOS commands were checked
  against the documented tar/shasum procedure and artifact names; the Windows
  commands against the documented PowerShell download/hash/Expand-Archive
  procedure. Bun's pinned install syntax was checked against its official
  installation page. Installation/execution on these platforms was not run.
  The page uses shared program/check/run commands, with separate PowerShell
  directory-creation and PATH recovery commands.
- No Linux ARM64 or musl archive is published by this release.
- Bun 1.3.9 is a tested version, not a claimed minimum or universal promise
  for all future Bun releases.
- Source/Cargo installation is not claimed as a clean tested installation
  route by this page.

## Scoped checks and remaining review

- `bun test apps/site/tests/first-run.test.ts`: focused production-component
  render of both first-run pages and their Docs entry links, exact localized
  command parity, canonical source parity, isolated lint/run and diagnostics.
  The Linux test can use `SESERAGI_FIRST_RUN_BIN` to select an extracted
  release CLI; it passes both with the published v0.61.19 binary and the
  inspected main build. The first Linux-only version passed 94 assertions;
  subsequent platform-command parity checks are source checks, not execution
  of macOS or Windows commands.
- `bun scripts/check-readme.ts`, `bun apps/site/scripts/check-content-map.ts`,
  focused TypeScript checks, Biome, Seseragi formatting, and `git diff --check`
  passed during implementation. The reference ledger stays at 351 leaves;
  the new procedure is not counted among them.
- The focused renderer compiles the real page/layout import closure and emits
  only the two task pages and two Docs entries. It does not stand in for a
  deterministic full-site build or global link validation.
- Browser regression coverage was added for both locales at 320, 390 and
  1280 pixels, with and without JavaScript: exact program/commands, legible
  code size, horizontally scrollable command panels, page overflow, next links,
  and navigation from Docs. This browser coverage was **not run** in the
  constrained cloud environment.
- The full `check:site`, all-page rendering, desktop/mobile visual review,
  screenshot review, and Japanese rendered-prose lint remain **not run** here;
  the known full-site render exceeds this cloud's memory budget. No alternate
  browser route was used. Run the normal site gate in a suitable environment
  before claiming full QA or completion of #699's display acceptance.
- #702 must distinguish this author/agent self-review from an actual practical
  TypeScript reader's first-use feedback. No reader-acceptance checkbox is
  marked by this record.
