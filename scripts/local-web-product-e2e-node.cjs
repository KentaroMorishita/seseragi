const fs = require("node:fs/promises")
const path = require("node:path")

async function runProductJourney(config, sdk) {
  if (process.versions.bun) {
    throw new Error("The VS Code SDK product journey requires Node.js")
  }
  sdk ??= require("@vscode/test-electron")
  const cachePath = await fs.mkdtemp(path.join(config.temporary, "vscode-"))
  const receipt = path.join(config.output, "provisioning.json")
  const report = {
    schema: 1,
    node: process.version,
    vscodeVersion: config.vscodeVersion,
    cachePath,
    freshCache: true,
    downloadCompleted: false,
    journeyCompleted: false,
  }
  const save = () =>
    fs.writeFile(receipt, `${JSON.stringify(report, null, 2)}\n`)
  try {
    await save()
    let vscodeExecutablePath = await sdk.downloadAndUnzipVSCode({
      version: config.vscodeVersion,
      cachePath,
    })
    // The SDK returns only after its length/checksum and extraction checks.
    // A private fresh cache prevents an incomplete earlier install being reused.
    report.downloadCompleted = true
    if (process.platform === "darwin") {
      const currentExecutable = path.join(
        path.dirname(vscodeExecutablePath),
        "Code"
      )
      if (
        await fs.stat(currentExecutable).then(
          () => true,
          () => false
        )
      ) {
        vscodeExecutablePath = currentExecutable
      }
    }
    report.vscodeExecutablePath = vscodeExecutablePath
    await save()
    await sdk.runVSCodeCommand(
      [
        "--install-extension",
        config.vsix,
        "--force",
        `--extensions-dir=${config.extensionsDirectory}`,
        `--user-data-dir=${config.userDataDirectory}`,
      ],
      { version: config.vscodeVersion, cachePath }
    )
    await sdk.runTests({
      extensionDevelopmentPath: path.join(
        config.repositoryRoot,
        "scripts",
        "fixtures",
        "local-web-e2e-driver"
      ),
      extensionTestsEnv: {
        SESERAGI_E2E_CLI: config.cli,
        SESERAGI_E2E_OUTPUT: config.output,
        SESERAGI_E2E_PLAYWRIGHT: path.join(
          config.repositoryRoot,
          "apps",
          "playground",
          "node_modules",
          "playwright"
        ),
        SESERAGI_E2E_PROJECT: config.project,
        SESERAGI_E2E_VERSION: config.version,
        SESERAGI_E2E_VSIX: config.vsix,
      },
      extensionTestsPath: path.join(
        config.repositoryRoot,
        "scripts",
        "local-web-product-e2e-extension.cjs"
      ),
      launchArgs: [
        config.project,
        `--extensions-dir=${config.extensionsDirectory}`,
        `--user-data-dir=${config.userDataDirectory}`,
      ],
      vscodeExecutablePath,
    })
    const product = JSON.parse(
      await fs.readFile(path.join(config.output, "report.json"), "utf8")
    )
    if (product.schema !== 1 || product.result !== "passed") {
      throw new Error("Extension Host did not produce a passing product report")
    }
    report.journeyCompleted = true
    await save()
    return report
  } catch (error) {
    report.error = String(error)
    await save()
    throw error
  } finally {
    await fs.rm(cachePath, { recursive: true, force: true })
  }
}

module.exports = { runProductJourney }
if (require.main === module) {
  ;(async () => {
    const config = JSON.parse(await fs.readFile(process.argv[2], "utf8"))
    await runProductJourney(config)
  })().catch((error) => {
    console.error(error)
    process.exitCode = 1
  })
}
