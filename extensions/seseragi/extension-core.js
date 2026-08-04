const fs = require("node:fs")
const path = require("node:path")
const { execFile } = require("node:child_process")
const { version: EXPECTED_TOOLCHAIN_VERSION } = require("./package.json")

const EXTENSION_ID = "seseragi-dev.seseragi"
const LEGACY_EXTENSION_ID = "seseragi-dev.seseragi-spec-preview"
const LEGACY_STUB_KIND = "seseragi-legacy-migration-stub"
const EXPECTED_PROTOCOL_VERSION = 1
const EXPECTED_ANALYSIS_SCHEMA_VERSION = 1
const RESTART_LIMIT = 3
const SERVER_TARGET_TRIPLES = Object.freeze({
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-x64": "x86_64-pc-windows-msvc",
})

function legacyFormatterTargets(vscode) {
  const configuration = vscode.workspace.getConfiguration("editor", {
    languageId: "seseragi",
  })
  const inspected = configuration.inspect("defaultFormatter")
  const candidates = [
    ["globalLanguageValue", vscode.ConfigurationTarget.Global],
    ["workspaceLanguageValue", vscode.ConfigurationTarget.Workspace],
    [
      "workspaceFolderLanguageValue",
      vscode.ConfigurationTarget.WorkspaceFolder,
    ],
  ]
  return candidates
    .filter(([key]) => inspected?.[key] === LEGACY_EXTENSION_ID)
    .map(([, target]) => target)
}

async function migrateLegacyFormatterSettings(vscode) {
  const configuration = vscode.workspace.getConfiguration("editor", {
    languageId: "seseragi",
  })
  const targets = legacyFormatterTargets(vscode)
  for (const target of targets) {
    await configuration.update("defaultFormatter", EXTENSION_ID, target, true)
  }
  return targets.length
}

async function allowOfficialServer(vscode, log) {
  const legacy = vscode.extensions?.getExtension?.(LEGACY_EXTENSION_ID)
  if (!legacy) return true

  try {
    const api = legacy.isActive ? legacy.exports : await legacy.activate()
    if (api?.kind === LEGACY_STUB_KIND) return true
  } catch (error) {
    log(`legacy extension was unavailable: ${error.message || error}`)
    return true
  }

  const action = await vscode.window.showWarningMessage(
    "A legacy Seseragi extension is active. Disable or uninstall it before using the official Seseragi extension so that only one language server runs.",
    "Open Extensions"
  )
  if (action === "Open Extensions") {
    await vscode.commands.executeCommand(
      "workbench.extensions.search",
      `@id:${LEGACY_EXTENSION_ID}`
    )
  }
  return false
}

async function offerLegacyFormatterMigration(vscode, log) {
  if (legacyFormatterTargets(vscode).length === 0) return
  const action = await vscode.window.showInformationMessage(
    "Seseragi's formatter extension ID changed. Migrate the existing [seseragi] default formatter setting?",
    "Migrate Settings"
  )
  if (action !== "Migrate Settings") return
  const updated = await migrateLegacyFormatterSettings(vscode)
  log(`migrated ${updated} legacy formatter setting${updated === 1 ? "" : "s"}`)
}

function platformTarget(platform = process.platform, arch = process.arch) {
  const targets = {
    "darwin:arm64": "darwin-arm64",
    "darwin:x64": "darwin-x64",
    "linux:x64": "linux-x64",
    "win32:x64": "win32-x64",
  }
  return targets[`${platform}:${arch}`]
}

function serverTargetTriple(platform = process.platform, arch = process.arch) {
  const target = platformTarget(platform, arch)
  return target === undefined ? undefined : SERVER_TARGET_TRIPLES[target]
}

function serverBinaryName(platform = process.platform) {
  return platform === "win32" ? "seseragi-lsp.exe" : "seseragi-lsp"
}

function resolveServerCommand({
  context,
  configuredPath = "",
  platform = process.platform,
  arch = process.arch,
  existsSync = fs.existsSync,
}) {
  const override = configuredPath.trim()
  if (override.length > 0) {
    return { command: override, source: "seseragi.languageServer.path" }
  }

  const target = platformTarget(platform, arch)
  if (target === undefined) {
    throw new Error(
      `Seseragi Language Server does not support ${platform}/${arch}. ` +
        "Set seseragi.languageServer.path to a compatible binary."
    )
  }
  const command = context.asAbsolutePath(
    path.join("server", target, serverBinaryName(platform))
  )
  if (!existsSync(command)) {
    throw new Error(
      `Bundled Seseragi Language Server is missing at ${command} for ${target}. ` +
        "Reinstall the matching VSIX or set seseragi.languageServer.path."
    )
  }
  return { command, source: `bundled ${target}` }
}

function assertServerExecutable(
  command,
  platform = process.platform,
  accessSync = fs.accessSync
) {
  if (platform === "win32") return
  try {
    accessSync(command, fs.constants.X_OK)
  } catch (error) {
    throw new Error(
      `Seseragi Language Server is not executable at ${command}: ` +
        `${error.message || error}. Reinstall the matching VSIX or set ` +
        "seseragi.languageServer.path."
    )
  }
}

function readServerVersion(command, run = execFile) {
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
              `Seseragi Language Server returned invalid version metadata: ${parseError.message}`
            )
          )
        }
      }
    )
  })
}

function validateServerVersion(version, expectedTarget) {
  if (version?.name !== "seseragi-lsp") {
    throw new Error("The selected executable is not seseragi-lsp.")
  }
  if (
    version.version !== EXPECTED_TOOLCHAIN_VERSION ||
    version.protocolVersion !== EXPECTED_PROTOCOL_VERSION ||
    version.analysisSchemaVersion !== EXPECTED_ANALYSIS_SCHEMA_VERSION
  ) {
    throw new Error(
      "Seseragi extension/server version mismatch: " +
        `expected version ${EXPECTED_TOOLCHAIN_VERSION}, protocol ` +
        `${EXPECTED_PROTOCOL_VERSION}, and analysis schema ` +
        `${EXPECTED_ANALYSIS_SCHEMA_VERSION}; received version ` +
        `${version.version ?? "unknown"}, protocol ` +
        `${version.protocolVersion ?? "unknown"}, and analysis schema ` +
        `${version.analysisSchemaVersion ?? "unknown"}.`
    )
  }
  if (expectedTarget !== undefined && version.target !== expectedTarget) {
    throw new Error(
      "Seseragi extension/server target mismatch: " +
        `expected ${expectedTarget}; received ${version.target ?? "unknown"}. ` +
        "Install the VSIX for this platform or set seseragi.languageServer.path."
    )
  }
  return version
}

function validateInitializeResult(result) {
  const contract = result?.experimental?.seseragi
  validateServerVersion({
    name: result?.serverInfo?.name,
    version: result?.serverInfo?.version,
    protocolVersion: contract?.protocolVersion,
    analysisSchemaVersion: contract?.analysisSchemaVersion,
  })
}

function createExtensionController({
  vscode,
  LanguageClient,
  TransportKind,
  State,
  RevealOutputChannelOn,
  ErrorAction,
  CloseAction,
  existsSync = fs.existsSync,
  accessSync = fs.accessSync,
  versionReader = readServerVersion,
  platform = process.platform,
  arch = process.arch,
}) {
  let client
  let output
  let status
  let context
  let restartCount = 0

  const log = (message) => output?.appendLine(`[Seseragi] ${message}`)

  function updateStatus(state, detail) {
    if (!status) return
    const states = {
      starting: ["$(sync~spin) Seseragi", "starting"],
      ready: ["$(check) Seseragi", "ready"],
      stopped: ["$(debug-disconnect) Seseragi", "stopped"],
      error: ["$(error) Seseragi", "error"],
    }
    const [text, label] = states[state]
    status.text = text
    status.tooltip = `Seseragi Language Server: ${detail || label}`
  }

  async function reportFailure(error) {
    const message = error instanceof Error ? error.message : String(error)
    log(`ERROR ${message}`)
    updateStatus("error", message)
    const action = await vscode.window.showErrorMessage(
      `Seseragi Language Server could not start. ${message}`,
      "Show Output",
      "Open Settings"
    )
    if (action === "Show Output") output.show(true)
    if (action === "Open Settings") {
      await vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "seseragi.languageServer.path"
      )
    }
  }

  async function stopClient() {
    if (client?.needsStop?.() || client?.isRunning?.()) {
      await client.stop()
    }
    client = undefined
  }

  async function startClient() {
    await stopClient()
    updateStatus("starting")

    const configuredPath = vscode.workspace
      .getConfiguration("seseragi")
      .get("languageServer.path", "")
    const server = resolveServerCommand({
      context,
      configuredPath,
      platform,
      arch,
      existsSync,
    })
    log(`command: ${server.command}`)
    log(`source: ${server.source}`)
    const workspace = (vscode.workspace.workspaceFolders || [])
      .map((folder) => folder.uri.fsPath)
      .join(", ")
    log(`workspace: ${workspace || "single file / untitled"}`)

    const expectedTarget = serverTargetTriple(platform, arch)
    assertServerExecutable(server.command, platform, accessSync)
    const version = validateServerVersion(
      await versionReader(server.command),
      expectedTarget
    )
    log(
      `binary: ${version.name} ${version.version}; target ` +
        `${version.target ?? "unknown"}; protocol ${version.protocolVersion}; ` +
        `analysis schema ${version.analysisSchemaVersion}`
    )

    const serverOptions = {
      command: server.command,
      args: [],
      transport: TransportKind.stdio,
    }
    const clientOptions = {
      documentSelector: [
        { scheme: "file", language: "seseragi" },
        { scheme: "untitled", language: "seseragi" },
      ],
      synchronize: {
        fileEvents: [
          vscode.workspace.createFileSystemWatcher("**/*.ssrg"),
          vscode.workspace.createFileSystemWatcher("**/seseragi.toml"),
        ],
      },
      outputChannel: output,
      revealOutputChannelOn: RevealOutputChannelOn.Never,
      initializationFailedHandler(error) {
        log(`initialize failed: ${error.message || error}`)
        return false
      },
      errorHandler: {
        error(error) {
          log(`transport error: ${error.message || error}`)
          return { action: ErrorAction.Continue }
        },
        closed() {
          restartCount += 1
          if (restartCount <= RESTART_LIMIT) {
            log(`server exited; restarting (${restartCount}/${RESTART_LIMIT})`)
            return { action: CloseAction.Restart }
          }
          const message =
            "Seseragi Language Server stopped repeatedly. Run Restart Language Server or inspect the output."
          log(message)
          updateStatus("error", "stopped repeatedly")
          void vscode.window
            .showErrorMessage(message, "Show Output")
            .then((action) => {
              if (action === "Show Output") output.show(true)
            })
          return { action: CloseAction.DoNotRestart }
        },
      },
    }

    client = new LanguageClient(
      "seseragi",
      "Seseragi Language Server",
      serverOptions,
      clientOptions
    )
    context.subscriptions.push(client)
    context.subscriptions.push(
      client.onDidChangeState(({ newState }) => {
        if (newState === State.Starting) updateStatus("starting")
        if (newState === State.Running) updateStatus("ready")
        if (newState === State.Stopped) updateStatus("stopped")
      })
    )
    await client.start()
    validateInitializeResult(client.initializeResult)
    restartCount = 0
    const encoding =
      client.initializeResult?.capabilities?.positionEncoding || "utf-16"
    log(`initialized; position encoding ${encoding}`)
    updateStatus("ready", `${version.version}; ${encoding}`)
  }

  async function activate(activationContext) {
    context = activationContext
    output = vscode.window.createOutputChannel("Seseragi Language Server")
    status = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      100
    )
    status.command = "seseragi.showLanguageServerOutput"
    status.show()
    context.subscriptions.push(output, status)
    context.subscriptions.push(
      vscode.commands.registerCommand("seseragi.showLanguageServerOutput", () =>
        output.show(true)
      ),
      vscode.commands.registerCommand(
        "seseragi.migrateLegacySettings",
        async () => {
          const updated = await migrateLegacyFormatterSettings(vscode)
          if (updated === 0) {
            await vscode.window.showInformationMessage(
              "No legacy Seseragi formatter settings need migration."
            )
            return
          }
          log(
            `migrated ${updated} legacy formatter setting${
              updated === 1 ? "" : "s"
            }`
          )
        }
      ),
      vscode.commands.registerCommand(
        "seseragi.restartLanguageServer",
        async () => {
          log("manual restart requested")
          try {
            await startClient()
          } catch (error) {
            await reportFailure(error)
          }
        }
      )
    )
    await offerLegacyFormatterMigration(vscode, log)
    if (!(await allowOfficialServer(vscode, log))) {
      updateStatus("stopped", "legacy extension needs migration")
      return
    }
    try {
      await startClient()
    } catch (error) {
      await reportFailure(error)
    }
  }

  async function deactivate() {
    await stopClient()
  }

  return { activate, deactivate }
}

module.exports = {
  EXTENSION_ID,
  EXPECTED_ANALYSIS_SCHEMA_VERSION,
  EXPECTED_PROTOCOL_VERSION,
  EXPECTED_TOOLCHAIN_VERSION,
  LEGACY_EXTENSION_ID,
  LEGACY_STUB_KIND,
  SERVER_TARGET_TRIPLES,
  allowOfficialServer,
  assertServerExecutable,
  createExtensionController,
  legacyFormatterTargets,
  migrateLegacyFormatterSettings,
  offerLegacyFormatterMigration,
  platformTarget,
  readServerVersion,
  resolveServerCommand,
  serverBinaryName,
  serverTargetTriple,
  validateInitializeResult,
  validateServerVersion,
}
