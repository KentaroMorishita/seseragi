const fs = require("node:fs")
const path = require("node:path")
const { execFile, spawn } = require("node:child_process")
const { version: EXPECTED_TOOLCHAIN_VERSION } = require("./package.json")

const DEV_URL_PATTERN = /^Dev server: (https?:\/\/\S+)$/m

function readCliVersion(command, run = execFile) {
  return new Promise((resolve, reject) => {
    run(
      command,
      ["--version-json"],
      { encoding: "utf8", timeout: 5000, windowsHide: true },
      (error, stdout, stderr) => {
        if (error) {
          reject(
            new Error(
              `Could not run ${command} --version-json: ${stderr || error.message}`
            )
          )
          return
        }
        try {
          resolve(JSON.parse(stdout))
        } catch (parseError) {
          reject(
            new Error(
              `Seseragi CLI returned invalid version metadata: ${parseError.message}`
            )
          )
        }
      }
    )
  })
}

function validateCliVersion(version, expectedTarget) {
  if (version?.name !== "seseragi") {
    throw new Error("The selected executable is not the Seseragi CLI.")
  }
  if (version.version !== EXPECTED_TOOLCHAIN_VERSION) {
    throw new Error(
      "Seseragi extension/CLI version mismatch: " +
        `expected ${EXPECTED_TOOLCHAIN_VERSION}; received ` +
        `${version.version ?? "unknown"}. Install the matching Seseragi CLI ` +
        "or set seseragi.cli.path."
    )
  }
  if (expectedTarget !== undefined && version.target !== expectedTarget) {
    throw new Error(
      "Seseragi extension/CLI target mismatch: " +
        `expected ${expectedTarget}; received ${version.target ?? "unknown"}. ` +
        "Install the CLI for this platform or set seseragi.cli.path."
    )
  }
  return version
}

function isInside(candidate, root) {
  const relative = path.relative(root, candidate)
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative))
}

function resolveProjectRoot({
  resourcePath,
  workspaceFolders = [],
  existsSync = fs.existsSync,
}) {
  const roots = workspaceFolders.map((folder) => path.resolve(folder.uri.fsPath))
  if (resourcePath) {
    const resource = path.resolve(resourcePath)
    const workspace = roots
      .filter((root) => isInside(resource, root))
      .sort((left, right) => right.length - left.length)[0]
    if (!workspace) {
      throw new Error("The selected Seseragi file is outside the open workspace.")
    }
    let current = path.extname(resource) ? path.dirname(resource) : resource
    while (isInside(current, workspace)) {
      if (existsSync(path.join(current, "seseragi.toml"))) return current
      if (current === workspace) break
      current = path.dirname(current)
    }
    throw new Error(
      `No seseragi.toml was found between ${resource} and workspace ${workspace}.`
    )
  }

  const candidates = roots.filter((root) =>
    existsSync(path.join(root, "seseragi.toml"))
  )
  if (candidates.length === 1) return candidates[0]
  if (candidates.length === 0) {
    throw new Error(
      "Open a Seseragi package or focus a .ssrg file inside a package first."
    )
  }
  throw new Error("Focus a .ssrg file to choose one Seseragi package.")
}

function createProjectCommandController({
  vscode,
  processSpawner = spawn,
  versionReader = readCliVersion,
  expectedTarget,
  existsSync = fs.existsSync,
  platform = process.platform,
}) {
  let output
  let status
  let context
  let devProcess
  let devRoot
  let devUrl
  let runSequence = Promise.resolve()

  const log = (message) => output?.appendLine(`[Seseragi Project] ${message}`)

  function updateStatus(state, detail) {
    if (!status) return
    const states = {
      idle: ["$(circle-outline) Seseragi Dev", "stopped"],
      starting: ["$(sync~spin) Seseragi Dev", "starting"],
      running: ["$(broadcast) Seseragi Dev", "running"],
      rebuilding: ["$(sync~spin) Seseragi Dev", "rebuilding"],
      failed: ["$(error) Seseragi Dev", "failed"],
    }
    const [text, label] = states[state]
    status.text = text
    status.tooltip = `Seseragi Development Server: ${label}${
      detail ? `; ${detail}` : ""
    }`
    status.command =
      state === "running" ? "seseragi.openInBrowser" : "seseragi.showProjectOutput"
  }

  function configuredCli() {
    const override = vscode.workspace
      .getConfiguration("seseragi")
      .get("cli.path", "")
      .trim()
    return override
      ? { command: override, source: "seseragi.cli.path" }
      : { command: platform === "win32" ? "seseragi.exe" : "seseragi", source: "PATH" }
  }

  async function checkedCli() {
    const cli = configuredCli()
    const version = validateCliVersion(
      await versionReader(cli.command),
      expectedTarget
    )
    log(
      `CLI: ${cli.command} (${cli.source}); version ${version.version}; ` +
        `target ${version.target ?? "unknown"}`
    )
    return cli.command
  }

  function selectedResource(resource) {
    if (resource?.fsPath) return resource.fsPath
    const editor = vscode.window.activeTextEditor
    return editor?.document?.uri?.scheme === "file"
      ? editor.document.uri.fsPath
      : undefined
  }

  function projectRoot(resource) {
    return resolveProjectRoot({
      resourcePath: selectedResource(resource),
      workspaceFolders: vscode.workspace.workspaceFolders || [],
      existsSync,
    })
  }

  function appendChunk(stream, prefix, onText) {
    if (!stream?.on) return
    stream.on("data", (chunk) => {
      const text = String(chunk)
      output?.append(text)
      onText?.(text)
      if (prefix && !text.endsWith("\n")) output?.append("\n")
    })
  }

  function spawnCli(command, arguments_, root) {
    log(`command: ${command} ${arguments_.join(" ")}`)
    log(`project: ${root}`)
    return processSpawner(command, arguments_, {
      cwd: root,
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"],
    })
  }

  async function runOnce(kind, arguments_, resource) {
    const root = projectRoot(resource)
    const command = await checkedCli()
    const child = spawnCli(command, arguments_(root), root)
    appendChunk(child.stdout)
    appendChunk(child.stderr)
    output.show(true)
    await new Promise((resolve, reject) => {
      child.once("error", reject)
      child.once("close", (code, signal) => {
        if (code === 0) {
          log(`${kind} completed`)
          resolve()
        } else {
          reject(
            new Error(
              `${kind} failed${signal ? ` with signal ${signal}` : ` with exit code ${code}`}`
            )
          )
        }
      })
    })
  }

  async function reportFailure(error) {
    const message = error instanceof Error ? error.message : String(error)
    log(`ERROR ${message}`)
    updateStatus("failed", message)
    const action = await vscode.window.showErrorMessage(
      `Seseragi project command failed. ${message}`,
      "Show Output",
      "Open Settings"
    )
    if (action === "Show Output") output.show(true)
    if (action === "Open Settings") {
      await vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "seseragi.cli.path"
      )
    }
  }

  function queue(action) {
    runSequence = runSequence.then(action, action).catch(reportFailure)
    return runSequence
  }

  async function startDev(resource) {
    if (devProcess) {
      await vscode.window.showInformationMessage(
        `Seseragi development server is already running for ${devRoot}.`
      )
      return
    }
    const root = projectRoot(resource)
    const command = await checkedCli()
    updateStatus("starting", path.basename(root))
    devRoot = root
    devUrl = undefined
    const child = spawnCli(command, ["dev", root], root)
    devProcess = child
    let stdout = ""
    appendChunk(child.stdout, undefined, (text) => {
      stdout = `${stdout}${text}`.slice(-8192)
      const match = stdout.match(DEV_URL_PATTERN)
      if (match) {
        devUrl = match[1]
        updateStatus("running", `${path.basename(root)} at ${devUrl}`)
        log(`browser: ${devUrl}`)
      } else if (text.includes("Built web app") && devUrl) {
        updateStatus("running", `${path.basename(root)} at ${devUrl}`)
      }
    })
    appendChunk(child.stderr, undefined, (text) => {
      if (text.includes("Build failed") || text.includes("error[")) {
        updateStatus("failed", `compiler diagnostics in ${path.basename(root)}`)
      }
    })
    child.once("error", (error) => {
      if (devProcess === child) {
        devProcess = undefined
        devRoot = undefined
        devUrl = undefined
      }
      void reportFailure(error)
    })
    child.once("close", (code, signal) => {
      if (devProcess !== child) return
      devProcess = undefined
      const stoppedRoot = devRoot
      devRoot = undefined
      devUrl = undefined
      if (code === 0 || signal) {
        updateStatus("idle", stoppedRoot ? path.basename(stoppedRoot) : undefined)
        log("development server stopped")
      } else {
        void reportFailure(
          new Error(`development server exited with code ${code}`)
        )
      }
    })
    output.show(true)
  }

  async function stopDev() {
    const child = devProcess
    if (!child) {
      updateStatus("idle")
      await vscode.window.showInformationMessage(
        "No Seseragi development server is running."
      )
      return
    }
    log("stopping development server")
    child.kill(platform === "win32" ? undefined : "SIGINT")
  }

  async function openInBrowser() {
    if (!devUrl) {
      await vscode.window.showErrorMessage(
        "The Seseragi development server has not reported a browser URL yet."
      )
      return
    }
    await vscode.env.openExternal(vscode.Uri.parse(devUrl))
  }

  async function activate(activationContext) {
    context = activationContext
    output = vscode.window.createOutputChannel("Seseragi Projects")
    status = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      99
    )
    updateStatus("idle")
    status.show()
    context.subscriptions.push(output, status)
    for (const pattern of ["**/*.ssrg", "**/seseragi.toml"]) {
      const watcher = vscode.workspace.createFileSystemWatcher(pattern)
      watcher.onDidChange?.((uri) => {
        if (devProcess && devRoot && isInside(uri.fsPath, devRoot)) {
          updateStatus("rebuilding", path.basename(devRoot))
        }
      })
      watcher.onDidCreate?.((uri) => {
        if (devProcess && devRoot && isInside(uri.fsPath, devRoot)) {
          updateStatus("rebuilding", path.basename(devRoot))
        }
      })
      watcher.onDidDelete?.((uri) => {
        if (devProcess && devRoot && isInside(uri.fsPath, devRoot)) {
          updateStatus("rebuilding", path.basename(devRoot))
        }
      })
      context.subscriptions.push(watcher)
    }
    context.subscriptions.push(
      vscode.workspace.onDidChangeWorkspaceFolders?.(() => {
        if (
          devProcess &&
          devRoot &&
          !(vscode.workspace.workspaceFolders || []).some((folder) =>
            isInside(devRoot, folder.uri.fsPath)
          )
        ) {
          log("owning workspace closed; stopping development server")
          devProcess.kill(platform === "win32" ? undefined : "SIGINT")
        }
      }) || { dispose() {} }
    )
    context.subscriptions.push(
      vscode.commands.registerCommand("seseragi.showProjectOutput", () =>
        output.show(true)
      ),
      vscode.commands.registerCommand("seseragi.runProject", (resource) =>
        queue(() => runOnce("run", (root) => ["run", root], resource))
      ),
      vscode.commands.registerCommand("seseragi.buildWebApp", (resource) =>
        queue(() =>
          runOnce(
            "Web build",
            (root) => ["build", root, "--target", "web", "--out-dir", "dist"],
            resource
          )
        )
      ),
      vscode.commands.registerCommand(
        "seseragi.startDevelopmentServer",
        (resource) => queue(() => startDev(resource))
      ),
      vscode.commands.registerCommand("seseragi.stopDevelopmentServer", () =>
        stopDev()
      ),
      vscode.commands.registerCommand("seseragi.openInBrowser", () =>
        openInBrowser()
      )
    )
  }

  async function deactivate() {
    if (devProcess) {
      const child = devProcess
      child.kill(platform === "win32" ? undefined : "SIGINT")
      await new Promise((resolve) => {
        const timeout = setTimeout(resolve, 2000)
        child.once("close", () => {
          clearTimeout(timeout)
          resolve()
        })
      })
      if (devProcess === child) {
        child.kill()
        devProcess = undefined
      }
    }
  }

  return { activate, deactivate }
}

module.exports = {
  DEV_URL_PATTERN,
  EXPECTED_TOOLCHAIN_VERSION,
  createProjectCommandController,
  readCliVersion,
  resolveProjectRoot,
  validateCliVersion,
}
