import { expect, test } from "bun:test"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"

test.each([true, false])(
  "full gate completes all checks with injected failures=%s",
  (failures) => {
    const root = mkdtempSync(join(tmpdir(), "gate-collection-"))
    try {
      for (const dir of [
        "scripts",
        "bin",
        "node_modules/.bin",
        "apps/playground/node_modules/.bin",
        "extensions/seseragi/node_modules/.bin",
      ]) {
        mkdirSync(join(root, dir), { recursive: true })
      }
      copyFileSync(
        join(import.meta.dir, "check-scoped.sh"),
        join(root, "scripts/check-scoped.sh")
      )
      const stub = `#!/bin/bash
printf '%s %s\\n' "$(basename "$0")" "$*" >> "$GATE_TRACE"
case "$*" in
  *check-readme.ts*|*samples:check*) exit ${failures ? 7 : 0} ;;
esac
exit 0
`
      for (const executable of [
        "bin/bun",
        "bin/cargo",
        "bin/git",
        "bin/uname",
        "node_modules/.bin/biome",
        "apps/playground/node_modules/.bin/tsc",
        "extensions/seseragi/node_modules/.bin/esbuild",
        "extensions/seseragi/node_modules/.bin/vsce",
        "scripts/build-playground-wasm.sh",
      ]) {
        writeFileSync(join(root, executable), stub, { mode: 0o755 })
      }
      const result = Bun.spawnSync(
        ["bash", "scripts/check-scoped.sh", "full"],
        {
          cwd: root,
          env: {
            ...process.env,
            PATH: `${root}/bin:${process.env.PATH}`,
            GATE_TRACE: join(root, "trace"),
          },
        }
      )
      expect(result.exitCode).toBe(failures ? 1 : 0)
      const output = result.stderr.toString()
      if (failures) {
        expect(output).toContain("Full gate failures (2)")
        expect(output).toContain("check-readme.ts")
        expect(output).toContain("samples:check")
        expect(result.stdout.toString()).not.toContain("All checks passed")
      } else {
        expect(result.stdout.toString()).toContain("All checks passed")
      }
      const trace = require("node:fs").readFileSync(join(root, "trace"), "utf8")
      expect(trace).toContain("bun run build:extension")
      expect(trace).toContain("cargo test --no-fail-fast --workspace")
      expect(trace).toContain("tsc --noEmit")
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  }
)

test("macOS runner executes later binaries after a failed binary", () => {
  const root = mkdtempSync(join(tmpdir(), "cargo-collection-"))
  try {
    writeFileSync(join(root, "codesign"), "#!/bin/sh\nexit 0\n", {
      mode: 0o755,
    })
    const first = join(root, "first")
    const last = join(root, "last")
    writeFileSync(first, "#!/bin/sh\necho first\nexit 1\n", { mode: 0o755 })
    writeFileSync(last, "#!/bin/sh\necho last\n", { mode: 0o755 })
    const manifest = join(root, "manifest")
    writeFileSync(
      manifest,
      [first, last]
        .map((executable) =>
          JSON.stringify({
            reason: "compiler-artifact",
            profile: { test: true },
            executable,
          })
        )
        .join("\n")
    )
    const result = Bun.spawnSync(
      [
        process.execPath,
        join(import.meta.dir, "run-macos-cargo-tests.ts"),
        manifest,
      ],
      { env: { ...process.env, PATH: `${root}:${process.env.PATH}` } }
    )
    expect(result.exitCode).toBe(1)
    expect(result.stdout.toString()).toContain("last")
    expect(result.stderr.toString()).toContain("Failed 1 Cargo test artifacts")
  } finally {
    rmSync(root, { recursive: true, force: true })
  }
})
