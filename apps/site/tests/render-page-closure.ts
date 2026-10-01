import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname, join, relative, resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
const sourceRoot = join(root, "apps/site/src")

// Render real production modules with only their relative import closure.
// This bounds a focused content test; it does not replace the full catalog,
// generated library, CSS/layout or browser gates. The caller owns temp cleanup.
export function renderPageClosure<T>(options: {
  directory: string
  modules: string[]
  entry: string
  cli?: string
  timeoutMs?: number
  stdin?: string
}): T {
  const { directory, modules, entry } = options
  const cli = resolve(
    root,
    options.cli ?? process.env.SESERAGI_BIN ?? "target/debug/seseragi"
  )
  const copied = new Set<string>()
  function copyModule(path: string) {
    assert.ok(path.startsWith(`${sourceRoot}/`), `Outside site source: ${path}`)
    if (copied.has(path)) return
    copied.add(path)
    const source = readFileSync(path, "utf8")
    const destination = join(directory, "src", relative(sourceRoot, path))
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(path, destination)
    for (const [, specifier] of source.matchAll(/from\s+"(\.[^"]+)"/gu))
      copyModule(
        resolve(
          dirname(path),
          specifier.endsWith(".ssrg") ? specifier : `${specifier}.ssrg`
        )
      )
  }
  for (const module of modules) copyModule(join(sourceRoot, `${module}.ssrg`))
  writeFileSync(
    join(directory, "seseragi.toml"),
    '[package]\nname = "site-page-verification"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "verify"\ntarget = "process"\n'
  )
  writeFileSync(join(directory, "src/verify.ssrg"), entry)
  function checked(args: string[], input?: string) {
    const result = spawnSync(cli, args, {
      cwd: directory,
      encoding: "utf8",
      input,
      timeout: options.timeoutMs ?? 90_000,
      maxBuffer: 16 * 1024 * 1024,
    })
    assert.equal(result.status, 0, result.stderr || result.error?.message)
    return result.stdout
  }
  checked(["lock", "update", directory])
  return JSON.parse(checked(["run", directory], options.stdin)) as T
}
