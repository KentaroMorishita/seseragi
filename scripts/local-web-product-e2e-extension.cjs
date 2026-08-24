const crypto = require("node:crypto")
const fs = require("node:fs")
const http = require("node:http")
const path = require("node:path")
const { spawnSync } = require("node:child_process")
const vscode = require("vscode")

const timeoutMs = 30_000

function requiredEnvironment(name) {
  const value = process.env[name]
  if (!value) throw new Error(`missing ${name}`)
  return value
}

async function waitFor(label, condition, timeout = timeoutMs) {
  const started = Date.now()
  let lastError
  while (Date.now() - started < timeout) {
    try {
      const value = await condition()
      if (value) return value
    } catch (error) {
      lastError = error
    }
    await new Promise((resolve) => setTimeout(resolve, 100))
  }
  throw new Error(
    `timed out waiting for ${label}${lastError ? `: ${lastError}` : ""}`
  )
}

async function responseText(url) {
  const response = await fetch(url)
  return { status: response.status, text: await response.text() }
}

async function replaceDocument(document, source) {
  const edit = new vscode.WorkspaceEdit()
  edit.replace(
    document.uri,
    new vscode.Range(
      document.positionAt(0),
      document.positionAt(document.getText().length)
    ),
    source
  )
  if (!(await vscode.workspace.applyEdit(edit))) {
    throw new Error(`could not edit ${document.uri.fsPath}`)
  }
  if (!(await document.save()))
    throw new Error(`could not save ${document.uri.fsPath}`)
}

function projectSourceHash(project) {
  const files = []
  const visit = (directory) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name)
      if (entry.isDirectory()) visit(absolute)
      else if (entry.name.endsWith(".ssrg")) files.push(absolute)
    }
  }
  visit(path.join(project, "src"))
  const hash = crypto.createHash("sha256")
  for (const file of files.sort()) {
    hash.update(path.relative(project, file).replaceAll(path.sep, "/"))
    hash.update("\0")
    hash.update(fs.readFileSync(file))
    hash.update("\0")
  }
  return `sha256:${hash.digest("hex")}`
}

function serveStatic(root) {
  const server = http.createServer((request, response) => {
    const pathname = new URL(request.url, "http://127.0.0.1").pathname
    const requested = pathname === "/" ? "index.html" : pathname.slice(1)
    const file = path.resolve(root, requested)
    if (
      !file.startsWith(`${path.resolve(root)}${path.sep}`) ||
      !fs.existsSync(file)
    ) {
      response.writeHead(404).end("not found")
      return
    }
    const extension = path.extname(file)
    const contentType =
      {
        ".css": "text/css",
        ".html": "text/html",
        ".js": "text/javascript",
        ".map": "application/json",
      }[extension] || "application/octet-stream"
    response.writeHead(200, { "content-type": contentType })
    fs.createReadStream(file).pipe(response)
  })
  return new Promise((resolve, reject) => {
    server.once("error", reject)
    server.listen(0, "127.0.0.1", () => {
      const address = server.address()
      resolve({ server, url: `http://127.0.0.1:${address.port}/` })
    })
  })
}

async function run() {
  const cli = requiredEnvironment("SESERAGI_E2E_CLI")
  const output = requiredEnvironment("SESERAGI_E2E_OUTPUT")
  const playwrightPath = requiredEnvironment("SESERAGI_E2E_PLAYWRIGHT")
  const project = requiredEnvironment("SESERAGI_E2E_PROJECT")
  const version = requiredEnvironment("SESERAGI_E2E_VERSION")
  const vsix = requiredEnvironment("SESERAGI_E2E_VSIX")
  const { chromium } = require(playwrightPath)
  fs.mkdirSync(output, { recursive: true })
  const observations = []
  const observe = (phase, detail) => observations.push({ phase, detail })
  let browser
  let devRunning = false
  let productionServer
  try {
    const extension = vscode.extensions.getExtension("seseragi-dev.seseragi")
    if (!extension)
      throw new Error("installed Seseragi VSIX was not discovered")
    if (extension.packageJSON.version !== version) {
      throw new Error(
        `VSIX version ${extension.packageJSON.version} != CLI ${version}`
      )
    }
    await vscode.workspace
      .getConfiguration("seseragi")
      .update("cli.path", cli, vscode.ConfigurationTarget.Workspace)
    await extension.activate()
    observe(
      "install",
      `activated ${extension.id} ${version} from ${path.basename(vsix)}`
    )

    const appUri = vscode.Uri.file(path.join(project, "src", "app.ssrg"))
    let initialDiagnosticsPublished = false
    const diagnosticSubscription = vscode.languages.onDidChangeDiagnostics(
      (event) => {
        if (event.uris.some((uri) => uri.toString() === appUri.toString())) {
          initialDiagnosticsPublished = true
        }
      }
    )
    const document = await vscode.workspace.openTextDocument(appUri)
    await vscode.window.showTextDocument(document)
    await waitFor("initial LSP diagnostics", () => initialDiagnosticsPublished)
    diagnosticSubscription.dispose()
    const initialDiagnostics = vscode.languages.getDiagnostics(appUri)
    if (initialDiagnostics.length !== 0) {
      throw new Error(
        `initial LSP diagnostics: ${initialDiagnostics.map((item) => item.message).join("; ")}`
      )
    }
    observe(
      "lsp",
      "generated canonical multi-module project and standard imports are clean"
    )

    const initialHash = projectSourceHash(project)
    await vscode.commands.executeCommand(
      "seseragi.startDevelopmentServer",
      appUri
    )
    devRunning = true
    const devUrl = "http://127.0.0.1:3000/"
    await waitFor(
      "VS Code development server",
      async () => (await responseText(devUrl)).status === 200
    )
    const firstVersion = (await responseText(`${devUrl}__seseragi_dev/version`))
      .text
    observe("dev", `running at ${devUrl}; reload ${firstVersion.trim()}`)

    browser = await chromium.launch({ headless: true })
    let page = await browser.newPage({
      viewport: { width: 1280, height: 900 },
    })
    await page.goto(devUrl)
    await page.getByText("Count: 0", { exact: true }).waitFor()
    await page.getByRole("button", { name: "Count one more" }).click()
    await page.getByText("Count: 1", { exact: true }).waitFor()
    await page.screenshot({
      path: path.join(output, "dev-interaction.png"),
      fullPage: true,
    })
    observe("browser", "initial render and Signal interaction passed")

    const original = document.getText()
    const changed = original.replace(
      "Hello from Seseragi",
      "Hello from my Seseragi app"
    )
    if (changed === original) throw new Error("canonical heading was not found")
    await replaceDocument(document, changed)
    const secondVersion = await waitFor("source rebuild", async () => {
      const value = (await responseText(`${devUrl}__seseragi_dev/version`)).text
      return value !== firstVersion ? value : undefined
    })
    await page
      .getByText("Hello from my Seseragi app", { exact: true })
      .waitFor()
    observe("edit", `browser auto-reloaded to ${secondVersion.trim()}`)

    const broken = `${changed}\nmissingProductE2eName\n`
    await replaceDocument(document, broken)
    const diagnostic = await waitFor("matching LSP diagnostic", () =>
      vscode.languages
        .getDiagnostics(appUri)
        .find((item) => String(item.code) === "SES-N0001")
    )
    const selected = document.getText(diagnostic.range)
    if (selected !== "missingProductE2eName") {
      throw new Error(`diagnostic range selected ${JSON.stringify(selected)}`)
    }
    const staleBuild = spawnSync(
      cli,
      ["build", project, "--target", "web", "--out-dir", "diagnostic-dist"],
      { encoding: "utf8" }
    )
    if (staleBuild.status === 0 || !staleBuild.stderr.includes("SES-K0102")) {
      throw new Error(
        `CLI did not enforce the stale lock: ${staleBuild.stderr}`
      )
    }
    const brokenLockUpdate = spawnSync(cli, ["lock", "update", project], {
      encoding: "utf8",
    })
    if (brokenLockUpdate.status !== 0) {
      throw new Error(
        `explicit lock update rejected the diagnostic project: ${brokenLockUpdate.stderr}`
      )
    }
    const cliFailure = spawnSync(
      cli,
      ["build", project, "--target", "web", "--out-dir", "diagnostic-dist"],
      { encoding: "utf8" }
    )
    if (cliFailure.status === 0 || !cliFailure.stderr.includes("SES-N0001")) {
      throw new Error(`CLI/LSP diagnostic mismatch: ${cliFailure.stderr}`)
    }
    fs.writeFileSync(path.join(output, "diagnostics.log"), cliFailure.stderr)
    const failedVersion = (
      await responseText(`${devUrl}__seseragi_dev/version`)
    ).text
    if (failedVersion !== secondVersion)
      throw new Error("failed rebuild replaced last success")
    observe("diagnostics", "LSP range and CLI build agree on SES-N0001")

    await replaceDocument(document, changed)
    await waitFor(
      "LSP diagnostic recovery",
      () => vscode.languages.getDiagnostics(appUri).length === 0
    )
    await waitFor(
      "development recovery",
      async () =>
        (await responseText(`${devUrl}__seseragi_dev/version`)).text !==
        secondVersion
    )
    observe("recovery", "LSP and dev server recovered without restarting")

    const recoveredLockUpdate = spawnSync(cli, ["lock", "update", project], {
      encoding: "utf8",
    })
    if (recoveredLockUpdate.status !== 0) {
      throw new Error(
        `explicit lock update rejected the recovered project: ${recoveredLockUpdate.stderr}`
      )
    }

    await vscode.commands.executeCommand("seseragi.buildWebApp", appUri)
    const dist = path.join(project, "dist")
    await waitFor("production dist", () =>
      fs.existsSync(path.join(dist, "index.html"))
    )
    const sourceMapFile = path.join(dist, "assets", "app.js.map")
    const sourceMap = JSON.parse(fs.readFileSync(sourceMapFile, "utf8"))
    if (
      !sourceMap.sources.some((source) =>
        source.endsWith("hello-web/0.0.0/app.ts")
      )
    ) {
      throw new Error(
        "production source map does not name the generated app module"
      )
    }
    observe(
      "build",
      "VS Code command produced standalone dist with source maps"
    )

    await vscode.commands.executeCommand("seseragi.stopDevelopmentServer")
    devRunning = false
    await waitFor("development server stop", async () => {
      try {
        await fetch(devUrl)
        return false
      } catch {
        return true
      }
    })
    productionServer = await serveStatic(dist)
    await page.close()
    page = await browser.newPage({ viewport: { width: 1280, height: 900 } })
    await page.goto(productionServer.url)
    await page
      .getByText("Hello from my Seseragi app", { exact: true })
      .waitFor()
    await page.getByRole("button", { name: "Count one more" }).click()
    await page.getByText("Count: 1", { exact: true }).waitFor()
    await page.screenshot({
      path: path.join(output, "production-interaction.png"),
      fullPage: true,
    })
    observe("production", `standalone dist passed at ${productionServer.url}`)

    fs.writeFileSync(
      path.join(output, "report.json"),
      `${JSON.stringify(
        {
          schema: 1,
          result: "passed",
          project: "seseragi new web hello-web",
          sourceHash: initialHash,
          cliVersion: version,
          extensionVersion: extension.packageJSON.version,
          sameSource: true,
          observations,
        },
        null,
        2
      )}\n`
    )
  } catch (error) {
    fs.writeFileSync(
      path.join(output, "failure.log"),
      `${error?.stack ? error.stack : error}\n`
    )
    throw error
  } finally {
    if (browser) {
      await browser.close()
      browser = undefined
    }
    if (productionServer) {
      productionServer.server.closeAllConnections?.()
      await new Promise((resolve) => productionServer.server.close(resolve))
    }
    if (devRunning) {
      await vscode.commands.executeCommand("seseragi.stopDevelopmentServer")
    }
  }
}

module.exports = { run }
