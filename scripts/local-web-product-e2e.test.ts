import { expect, test } from "bun:test"
import path from "node:path"
import { repositoryRoot } from "./release-contract"

const runner = await Bun.file(
  path.join(repositoryRoot, "scripts", "local-web-product-e2e.ts")
).text()
const extensionTest = await Bun.file(
  path.join(repositoryRoot, "scripts", "local-web-product-e2e-extension.cjs")
).text()
const pullRequestWorkflow = await Bun.file(
  path.join(repositoryRoot, ".github", "workflows", "local-web-product-e2e.yml")
).text()
const releaseWorkflow = await Bun.file(
  path.join(repositoryRoot, ".github", "workflows", "release.yml")
).text()

test("uses installed artifacts and one canonical project for the product journey", () => {
  expect(runner).toContain("--install-extension")
  expect(runner).toContain('SESERAGI_E2E_VSCODE_VERSION ?? "1.133.0"')
  expect(runner).toContain("process.exitCode = 0")
  expect(runner).toContain("nativeArchiveName")
  expect(runner).toContain('"examples", "samples", "project-flow-app"')
  expect(extensionTest).toContain('getExtension("seseragi-dev.seseragi")')
  expect(extensionTest).toContain('"seseragi.startDevelopmentServer"')
  expect(extensionTest).toContain('"seseragi.buildWebApp"')
  expect(extensionTest).toContain('"seseragi.stopDevelopmentServer"')
  expect(extensionTest).toContain('String(item.code) === "SES-N0001"')
  expect(extensionTest).toContain("chromium.launch({ headless: true })")
  expect(extensionTest).toContain("sameSource: true")
})

test("runs for pull requests and downloaded tag release artifacts", () => {
  expect(pullRequestWorkflow).toContain("test:local-web-e2e")
  expect(pullRequestWorkflow).toContain("target/local-web-product-e2e")
  expect(releaseWorkflow).toContain("local-web-product-e2e:")
  expect(releaseWorkflow).toContain(
    "seseragi-release-$" +
      "{{ needs.gate.outputs.release_sha }}-native-linux-x64"
  )
  expect(releaseWorkflow).toContain(
    "seseragi-release-$" +
      "{{ needs.gate.outputs.release_sha }}-vscode-linux-x64"
  )
  expect(releaseWorkflow).toContain(
    "native-verify, vscode, vscode-legacy, wasm-runtime, local-web-product-e2e"
  )
})
