import { describe, expect, test } from "bun:test"
import path from "node:path"
import manifest from "../package.json"

const {
  EXTENSION_ID,
  LEGACY_EXTENSION_ID,
  LEGACY_STUB_KIND,
  createExtensionController,
  legacyFormatterTargets,
  migrateLegacyFormatterSettings,
  platformTarget,
  resolveServerCommand,
  serverBinaryName,
  serverTargetTriple,
  validateInitializeResult,
  validateServerVersion,
} = require("../extension-core")

const version = {
  name: "seseragi-lsp",
  version: manifest.version,
  target: "aarch64-apple-darwin",
  protocolVersion: 1,
  analysisSchemaVersion: 1,
}

const bundledServerPath = path.join(
  "/extension",
  "server",
  "darwin-arm64",
  "seseragi-lsp"
)

function extensionHarness(
  options: {
    configuredPath?: string
    binaryExists?: boolean
    binaryExecutable?: boolean
    legacyExtension?: {
      api?: unknown
      activateError?: Error
      isActive?: boolean
    }
    legacyFormatterValues?: Record<string, string>
    serverVersion?: typeof version
    versionReader?: () => Promise<typeof version>
    initializeVersion?: string
    startHook?: () => Promise<void>
  } = {}
) {
  const lines: string[] = []
  const errors: string[] = []
  const formatterUpdates: Array<{
    key: string
    target: number
    value: string
    overrideInLanguage: boolean
  }> = []
  const information: string[] = []
  const commands = new Map<string, () => unknown>()
  const status = { text: "", tooltip: "", command: "", show() {} }
  const output = {
    appendLine(line: string) {
      lines.push(line)
    },
    showCalls: 0,
    show() {
      this.showCalls += 1
    },
    dispose() {},
  }
  const vscode = {
    StatusBarAlignment: { Left: 1 },
    ConfigurationTarget: { Global: 1, Workspace: 2, WorkspaceFolder: 3 },
    workspace: {
      workspaceFolders: [{ uri: { fsPath: "/workspace/example" } }],
      getConfiguration(section: string) {
        if (section === "editor") {
          return {
            inspect() {
              return options.legacyFormatterValues || {}
            },
            async update(
              key: string,
              value: string,
              target: number,
              overrideInLanguage: boolean
            ) {
              formatterUpdates.push({
                key,
                value,
                target,
                overrideInLanguage,
              })
            },
          }
        }
        return {
          get(_key: string, fallback: string) {
            return options.configuredPath ?? fallback
          },
        }
      },
      createFileSystemWatcher(pattern: string) {
        return { pattern, dispose() {} }
      },
    },
    window: {
      createOutputChannel() {
        return output
      },
      createStatusBarItem() {
        return status
      },
      async showErrorMessage(message: string) {
        errors.push(message)
        return undefined
      },
      async showInformationMessage(message: string) {
        information.push(message)
        return undefined
      },
      async showWarningMessage(message: string) {
        information.push(message)
        return undefined
      },
    },
    extensions: {
      getExtension(id: string) {
        if (id !== LEGACY_EXTENSION_ID || !options.legacyExtension) {
          return undefined
        }
        return {
          isActive: options.legacyExtension.isActive ?? false,
          exports: options.legacyExtension.api,
          async activate() {
            if (options.legacyExtension?.activateError) {
              throw options.legacyExtension.activateError
            }
            return options.legacyExtension?.api
          },
        }
      },
    },
    commands: {
      registerCommand(command: string, callback: () => unknown) {
        commands.set(command, callback)
        return { dispose() {} }
      },
      async executeCommand() {},
    },
  }
  const State = { Stopped: 1, Running: 2, Starting: 3 }
  class MockLanguageClient {
    static instances: MockLanguageClient[] = []
    initializeResult = {
      serverInfo: {
        name: "seseragi-lsp",
        version: options.initializeVersion ?? manifest.version,
      },
      capabilities: {
        positionEncoding: "utf-8",
        renameProvider: { prepareProvider: true },
        workspaceSymbolProvider: true,
      },
      experimental: {
        seseragi: { protocolVersion: 1, analysisSchemaVersion: 1 },
      },
    }
    running = false
    starting = false
    startPromise?: Promise<void>
    onStartJoined?: () => void
    listener?: (event: { newState: number }) => void
    stopped = false

    constructor(
      public id: string,
      public name: string,
      public serverOptions: unknown,
      public clientOptions: unknown
    ) {
      MockLanguageClient.instances.push(this)
    }

    onDidChangeState(listener: (event: { newState: number }) => void) {
      this.listener = listener
      return { dispose() {} }
    }

    async start() {
      if (this.startPromise) {
        this.onStartJoined?.()
        return this.startPromise
      }
      const attempt = (async () => {
        this.starting = true
        this.listener?.({ newState: State.Starting })
        try {
          await options.startHook?.()
          this.running = true
          this.listener?.({ newState: State.Running })
        } finally {
          this.starting = false
        }
      })()
      this.startPromise = attempt
      try {
        await attempt
      } finally {
        if (this.startPromise === attempt) this.startPromise = undefined
      }
    }

    needsStop() {
      return this.starting || this.running
    }

    isRunning() {
      return this.running
    }

    async stop() {
      // vscode-languageclient cannot shut down a connection still starting.
      if (this.starting)
        throw new Error("Client is not running and can't be stopped")
      this.running = false
      this.stopped = true
      this.listener?.({ newState: State.Stopped })
    }

    dispose() {}
  }
  const controller = createExtensionController({
    vscode,
    LanguageClient: MockLanguageClient,
    TransportKind: { stdio: "stdio" },
    State,
    RevealOutputChannelOn: { Never: "never" },
    ErrorAction: { Continue: "continue" },
    CloseAction: { Restart: "restart", DoNotRestart: "stop" },
    existsSync: () => options.binaryExists ?? true,
    accessSync: () => {
      if (options.binaryExecutable === false) {
        throw new Error("EACCES: permission denied")
      }
    },
    versionReader:
      options.versionReader ?? (async () => options.serverVersion ?? version),
    platform: "darwin",
    arch: "arm64",
  })
  const context = {
    subscriptions: [] as unknown[],
    asAbsolutePath(relative: string) {
      return path.join("/extension", relative)
    },
  }
  return {
    commands,
    context,
    controller,
    errors,
    formatterUpdates,
    information,
    lines,
    MockLanguageClient,
    output,
    status,
    vscode,
  }
}

describe("official VS Code extension contract", () => {
  test("registers every .ssrg file under the official extension ID", () => {
    expect(`${manifest.publisher}.${manifest.name}`).toBe(EXTENSION_ID)
    expect(manifest.displayName).toBe("Seseragi")
    expect(manifest.contributes.languages).toEqual([
      expect.objectContaining({
        id: "seseragi",
        aliases: ["Seseragi"],
        extensions: [".ssrg"],
      }),
    ])
    expect(manifest.contributes.grammars).toContainEqual(
      expect.objectContaining({
        language: "seseragi",
        scopeName: "source.seseragi",
      })
    )
    expect(JSON.stringify(manifest)).not.toContain("examples/spec/**")
    expect(
      manifest.contributes.configuration.properties[
        "seseragi.languageServer.path"
      ].default
    ).toBe("")
    expect(manifest.contributes.configurationDefaults["[seseragi]"]).toEqual({
      "editor.defaultFormatter": EXTENSION_ID,
    })
    expect(manifest.contributes.commands).toContainEqual(
      expect.objectContaining({ command: "seseragi.migrateLegacySettings" })
    )
  })

  test("maps every release platform to one bundled server", () => {
    expect(platformTarget("darwin", "arm64")).toBe("darwin-arm64")
    expect(platformTarget("darwin", "x64")).toBe("darwin-x64")
    expect(platformTarget("linux", "x64")).toBe("linux-x64")
    expect(platformTarget("win32", "x64")).toBe("win32-x64")
    expect(platformTarget("linux", "arm64")).toBeUndefined()
    expect(serverBinaryName("win32")).toBe("seseragi-lsp.exe")
    expect(serverTargetTriple("darwin", "arm64")).toBe("aarch64-apple-darwin")
    expect(serverTargetTriple("darwin", "x64")).toBe("x86_64-apple-darwin")
    expect(serverTargetTriple("linux", "x64")).toBe("x86_64-unknown-linux-gnu")
    expect(serverTargetTriple("win32", "x64")).toBe("x86_64-pc-windows-msvc")
  })

  test("prefers an explicit server override and fixes the missing binary message", () => {
    expect(
      resolveServerCommand({
        context: { asAbsolutePath: () => "/unused" },
        configuredPath: "/custom/seseragi-lsp",
        existsSync: () => false,
      })
    ).toEqual({
      command: "/custom/seseragi-lsp",
      source: "seseragi.languageServer.path",
    })
    expect(() =>
      resolveServerCommand({
        context: {
          asAbsolutePath: (relative: string) => `/extension/${relative}`,
        },
        configuredPath: "",
        platform: "linux",
        arch: "x64",
        existsSync: () => false,
      })
    ).toThrow("Reinstall the matching VSIX")
  })

  test("rejects a protocol or analysis schema mismatch before startup", () => {
    expect(validateServerVersion(version, "aarch64-apple-darwin")).toEqual(
      version
    )
    expect(() =>
      validateServerVersion({ ...version, protocolVersion: 2 })
    ).toThrow("extension/server version mismatch")
    expect(() =>
      validateServerVersion(
        { ...version, target: "x86_64-apple-darwin" },
        "aarch64-apple-darwin"
      )
    ).toThrow("extension/server target mismatch")
    expect(() =>
      validateServerVersion({ ...version, version: "0.3.0" })
    ).toThrow("extension/server version mismatch")
    expect(() =>
      validateInitializeResult({
        serverInfo: { name: "seseragi-lsp", version: manifest.version },
        experimental: {
          seseragi: { protocolVersion: 1, analysisSchemaVersion: 2 },
        },
      })
    ).toThrow("extension/server version mismatch")
  })

  test("reports a non-executable bundled path before starting a language client", async () => {
    const harness = extensionHarness({ binaryExecutable: false })
    await harness.controller.activate(harness.context)

    expect(harness.MockLanguageClient.instances).toHaveLength(0)
    expect(harness.errors.join("\n")).toContain("is not executable at")
    expect(harness.errors.join("\n")).toContain(bundledServerPath)
    expect(harness.lines.join("\n")).toContain(`command: ${bundledServerPath}`)
  })

  test("reports a wrong-architecture binary before starting a language client", async () => {
    const harness = extensionHarness({
      serverVersion: { ...version, target: "x86_64-apple-darwin" },
    })
    await harness.controller.activate(harness.context)

    expect(harness.MockLanguageClient.instances).toHaveLength(0)
    expect(harness.errors.join("\n")).toContain("target mismatch")
    expect(harness.errors.join("\n")).toContain("aarch64-apple-darwin")
    expect(harness.lines.join("\n")).toContain(`command: ${bundledServerPath}`)
  })

  test("migrates only explicit language-scoped legacy formatter settings", async () => {
    const harness = extensionHarness({
      legacyFormatterValues: {
        globalLanguageValue: LEGACY_EXTENSION_ID,
        workspaceLanguageValue: LEGACY_EXTENSION_ID,
        workspaceFolderLanguageValue: "another.publisher.formatter",
      },
    })
    expect(legacyFormatterTargets(harness.vscode)).toEqual([1, 2])
    expect(await migrateLegacyFormatterSettings(harness.vscode)).toBe(2)
    expect(harness.formatterUpdates).toEqual([
      {
        key: "defaultFormatter",
        value: EXTENSION_ID,
        target: 1,
        overrideInLanguage: true,
      },
      {
        key: "defaultFormatter",
        value: EXTENSION_ID,
        target: 2,
        overrideInLanguage: true,
      },
    ])
  })

  test("does not start a second LSP while a pre-migration extension is active", async () => {
    const harness = extensionHarness({
      legacyExtension: { api: { activate() {}, deactivate() {} } },
    })
    await harness.controller.activate(harness.context)

    expect(harness.MockLanguageClient.instances).toHaveLength(0)
    expect(harness.information.join("\n")).toContain(
      "legacy Seseragi extension is active"
    )
    expect(harness.status.tooltip).toContain("legacy extension needs migration")
    await harness.commands.get("seseragi.restartLanguageServer")?.()
    expect(harness.MockLanguageClient.instances).toHaveLength(0)
  })

  test("starts the official LSP alongside the legacy migration stub", async () => {
    const harness = extensionHarness({
      legacyExtension: { api: { kind: LEGACY_STUB_KIND } },
    })
    await harness.controller.activate(harness.context)

    expect(harness.MockLanguageClient.instances).toHaveLength(1)
  })

  test("starts the bundled server for file and untitled documents with visible state", async () => {
    const harness = extensionHarness()
    await harness.controller.activate(harness.context)

    expect(harness.errors).toEqual([])
    expect(harness.MockLanguageClient.instances).toHaveLength(1)
    const instance = harness.MockLanguageClient.instances[0]
    expect(instance.serverOptions).toEqual({
      command: bundledServerPath,
      args: [],
      transport: "stdio",
    })
    expect(instance.clientOptions).toEqual(
      expect.objectContaining({
        documentSelector: [
          { scheme: "file", language: "seseragi" },
          { scheme: "untitled", language: "seseragi" },
        ],
      })
    )
    expect(
      (
        instance.clientOptions as {
          synchronize: { fileEvents: Array<{ pattern: string }> }
        }
      ).synchronize.fileEvents.map((watcher) => watcher.pattern)
    ).toEqual(["**/*.ssrg", "**/seseragi.toml"])
    expect(harness.lines.join("\n")).toContain(
      "initialized; position encoding utf-8"
    )
    expect(harness.status.text).toContain("Seseragi")
    expect(harness.status.tooltip).toContain(manifest.version)
    expect(harness.commands.has("seseragi.restartLanguageServer")).toBe(true)
    expect(harness.commands.has("seseragi.showLanguageServerOutput")).toBe(true)

    await harness.commands.get("seseragi.restartLanguageServer")?.()
    expect(instance.stopped).toBe(true)
    expect(harness.MockLanguageClient.instances).toHaveLength(2)
  })

  test("connects VS Code rename and workspace symbol providers", async () => {
    const harness = extensionHarness()
    await harness.controller.activate(harness.context)

    const capabilities =
      harness.MockLanguageClient.instances[0].initializeResult.capabilities
    expect(capabilities.renameProvider).toEqual({ prepareProvider: true })
    expect(capabilities.workspaceSymbolProvider).toBe(true)
  })

  test("overlapping manual restarts leave exactly one running language server", async () => {
    const harness = extensionHarness()
    await harness.controller.activate(harness.context)
    const restart = harness.commands.get("seseragi.restartLanguageServer")!
    await Promise.all([restart(), restart()])
    expect(harness.errors).toEqual([])
    expect(
      harness.MockLanguageClient.instances.filter((client) => client.running)
    ).toHaveLength(1)
    await harness.controller.deactivate()
    expect(
      harness.MockLanguageClient.instances.filter((client) => client.running)
    ).toHaveLength(0)
  })

  test("deactivation during version inspection prevents a late server start", async () => {
    let releaseVersion!: (value: typeof version) => void
    let inspectionStarted!: () => void
    const inspected = new Promise<void>((resolve) => {
      inspectionStarted = resolve
    })
    const pendingVersion = new Promise<typeof version>((resolve) => {
      releaseVersion = resolve
    })
    const harness = extensionHarness({
      versionReader: () => {
        inspectionStarted()
        return pendingVersion
      },
    })
    const activation = harness.controller.activate(harness.context)
    await inspected
    const deactivation = harness.controller.deactivate()
    releaseVersion(version)
    await Promise.all([activation, deactivation])
    expect(harness.errors).toEqual([])
    expect(harness.MockLanguageClient.instances).toHaveLength(0)
  })

  test("deactivation during startup awaits and stops the pending client", async () => {
    let releaseStart!: () => void
    let startEntered!: () => void
    const entered = new Promise<void>((resolve) => {
      startEntered = resolve
    })
    const pendingStart = new Promise<void>((resolve) => {
      releaseStart = resolve
    })
    const harness = extensionHarness({
      startHook: () => {
        startEntered()
        return pendingStart
      },
    })
    const activation = harness.controller.activate(harness.context)
    await entered
    const deactivation = harness.controller.deactivate()
    releaseStart()
    await Promise.all([activation, deactivation])
    expect(harness.errors).toEqual([])
    expect(harness.MockLanguageClient.instances).toHaveLength(1)
    expect(harness.MockLanguageClient.instances[0].stopped).toBe(true)
    expect(harness.MockLanguageClient.instances[0].running).toBe(false)
  })

  test("a rejected initialize contract stops the already started server", async () => {
    const harness = extensionHarness({ initializeVersion: "0.0.0" })
    await harness.controller.activate(harness.context)
    expect(harness.errors).toHaveLength(1)
    expect(harness.errors[0]).toContain("extension/server version mismatch")
    expect(harness.MockLanguageClient.instances[0].stopped).toBe(true)
    expect(harness.MockLanguageClient.instances[0].running).toBe(false)
  })

  test("deactivation also joins an automatic client restart before stopping it", async () => {
    const options: { startHook?: () => Promise<void> } = {}
    const harness = extensionHarness(options)
    await harness.controller.activate(harness.context)
    const client = harness.MockLanguageClient.instances[0]
    let releaseStart!: () => void
    const pendingStart = new Promise<void>((resolve) => {
      releaseStart = resolve
    })
    options.startHook = () => pendingStart
    let joinStarted!: () => void
    const joined = new Promise<"joined">((resolve) => {
      joinStarted = () => resolve("joined")
    })
    client.onStartJoined = joinStarted
    client.running = false
    // The client library initiates this recovery itself, outside the controller queue.
    const automaticStart = client.start()
    const deactivation = harness.controller.deactivate().then(
      () => undefined,
      (error: Error) => error
    )
    const boundary = await Promise.race([joined, deactivation])
    releaseStart()
    await automaticStart
    const error = await deactivation
    expect(boundary).toBe("joined")
    expect(error).toBeUndefined()
    expect(client.stopped).toBe(true)
    expect(client.running).toBe(false)
  })

  test("surfaces a missing bundled server instead of failing silently", async () => {
    const harness = extensionHarness({ binaryExists: false })
    await harness.controller.activate(harness.context)
    expect(harness.MockLanguageClient.instances).toHaveLength(0)
    expect(harness.errors).toHaveLength(1)
    expect(harness.errors[0]).toContain(
      "Bundled Seseragi Language Server is missing"
    )
    expect(harness.status.text).toContain("error")
  })

  test("restarts a crashed server with a bounded visible recovery path", async () => {
    const harness = extensionHarness()
    await harness.controller.activate(harness.context)
    const instance = harness.MockLanguageClient.instances[0]
    const errorHandler = (
      instance.clientOptions as {
        errorHandler: { closed(): { action: string } }
      }
    ).errorHandler

    expect(errorHandler.closed().action).toBe("restart")
    expect(errorHandler.closed().action).toBe("restart")
    expect(errorHandler.closed().action).toBe("restart")
    expect(errorHandler.closed().action).toBe("stop")
    await Promise.resolve()

    expect(harness.status.tooltip).toContain("stopped repeatedly")
    expect(harness.errors.join("\n")).toContain(
      "Run Restart Language Server or inspect the output"
    )
  })
})
