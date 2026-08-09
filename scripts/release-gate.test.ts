import { afterEach, describe, expect, test } from "bun:test"
import { execFileSync } from "node:child_process"
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { repositoryRoot, verifyReleaseCommit } from "./release-gate"

const temporaryRepositories: string[] = []

function git(root: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim()
}

async function repository(): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "seseragi-release-gate-"))
  temporaryRepositories.push(root)
  git(root, "init", "-b", "main")
  git(root, "config", "user.name", "Seseragi release test")
  git(root, "config", "user.email", "release-test@example.invalid")
  await writeFile(path.join(root, "fixture.txt"), "main\n")
  git(root, "add", "fixture.txt")
  git(root, "commit", "-m", "main release candidate")
  return root
}

afterEach(async () => {
  await Promise.all(
    temporaryRepositories
      .splice(0)
      .map((root) => rm(root, { recursive: true, force: true }))
  )
})

describe("release publish gate", () => {
  test("accepts the checked-out tag commit when main contains it", async () => {
    const root = await repository()
    git(root, "tag", "-a", "v0.4.0", "-m", "release v0.4.0")

    expect(verifyReleaseCommit("v0.4.0", "refs/heads/main", root)).toBe(
      git(root, "rev-parse", "HEAD")
    )
  })

  test("rejects a tag commit that is not contained in main", async () => {
    const root = await repository()
    git(root, "switch", "-c", "release-side")
    await writeFile(path.join(root, "fixture.txt"), "side\n")
    git(root, "commit", "-am", "unmerged release candidate")
    git(root, "tag", "-a", "v0.4.1", "-m", "release v0.4.1")

    expect(() =>
      verifyReleaseCommit("v0.4.1", "refs/heads/main", root)
    ).toThrow("is not contained in refs/heads/main")
  })

  test("rejects a checkout that differs from the pushed tag", async () => {
    const root = await repository()
    git(root, "tag", "-a", "v0.4.0", "-m", "release v0.4.0")
    await writeFile(path.join(root, "fixture.txt"), "newer main\n")
    git(root, "commit", "-am", "advance main")

    expect(() =>
      verifyReleaseCommit("v0.4.0", "refs/heads/main", root)
    ).toThrow("but the checked-out release commit is")
  })

  test("pins every release artifact and publish download to the gated SHA", async () => {
    const workflow = await readFile(
      path.join(repositoryRoot, ".github/workflows/release.yml"),
      "utf8"
    )
    const releaseShaExpression = [
      "$",
      "{{ needs.gate.outputs.release_sha }}",
    ].join("")
    const matrixTargetExpression = ["$", "{{ matrix.target }}"].join("")

    expect(workflow).toContain("run: bun run check:wasm")
    expect(workflow).toContain(
      "run: ./scripts/check-scoped.sh release-gate-after-wasm"
    )
    expect(workflow.indexOf("run: bun run check:wasm")).toBeLessThan(
      workflow.indexOf("release-gate-after-wasm")
    )
    expect(workflow).toContain("gate:\n    runs-on: macos-15")
    expect(workflow).toContain("Upload regenerated canonical WASM")
    expect(workflow).toContain("seseragi-wasm-freshness-")
    expect(workflow).toContain(
      `seseragi-release-${releaseShaExpression}-native-${matrixTargetExpression}`
    )
    expect(workflow).toContain(
      `pattern: seseragi-release-${releaseShaExpression}-*`
    )
    expect(
      workflow.match(/ref: \$\{\{ needs\.gate\.outputs\.release_sha \}\}/gu)
        ?.length
    ).toBeGreaterThanOrEqual(6)
    expect(workflow).toContain(
      "needs: [gate, native-verify, vscode, vscode-legacy, wasm-runtime]"
    )
    expect(workflow.match(/release-gate\.ts check-main/gu)?.length).toBe(2)
    expect(workflow).toContain("bun scripts/release-readiness.ts check")
    expect(workflow).toContain("version: v0.15.0")
    expect(workflow.match(/toolchain: 1\.97\.1/gu)?.length).toBe(3)
    expect(workflow).not.toContain("VSCE_PAT")
    expect(workflow).not.toContain("vsce publish")
    expect(workflow).not.toContain("marketplace:")
    expect(workflow).toContain("target/release/seseragi-v*")
    expect(workflow).toContain("seseragi-legacy-migration-v")
    expect(workflow).toContain(
      "cp -R apps/playground/src/wasm/pkg target/release/wasm"
    )
    expect(workflow).not.toContain(
      "./scripts/build-playground-wasm.sh target/release/wasm"
    )

    const wasmBuild = await readFile(
      path.join(repositoryRoot, "scripts/build-playground-wasm.sh"),
      "utf8"
    )
    expect(wasmBuild).toContain("--remap-path-prefix=$RUST_CARGO_HOME=/cargo")
  })

  test("runs shared extension tests once alongside platform packaging", async () => {
    const workflow = await readFile(
      path.join(repositoryRoot, ".github/workflows/vscode-extension.yml"),
      "utf8"
    )

    expect(workflow).toContain("push:\n    branches: [main]\n    paths:")
    expect(workflow).toContain("name: shared extension and LSP tests")
    expect(workflow.match(/cargo test -p seseragi-lsp/gu)?.length).toBe(1)
    expect(workflow).not.toContain("needs: test")
    expect(workflow).toContain("run: bun scripts/package-extension.ts")
  })
})
