import { describe, expect, test } from "bun:test"
import path from "node:path"
import manifest from "../package.json"

const {
  createExtensionController,
  platformTarget,
  resolveServerCommand,
  serverBinaryName,
  validateInitializeResult,
  validateServerVersion,
} = require("../extension-core")

const version = {
  name: "seseragi-lsp",
  version: "0.3.0",
  protocolVersion: 1,
  analysisSchemaVersion: 1,
}

function extensionHarness(
  options: {
    configuredPath?: string
    binaryExists?: boolean
    serverVersion?: typeof version
  } = {}
) {
  const lines: string[] = []
  const errors: string[] = []
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
    workspace: {
      workspaceFolders: [{ uri: { fsPath: "/workspace/example" } }],
      getConfiguration() {
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
      serverInfo: { name: "seseragi-lsp", version: "0.3.0" },
      capabilities: { positionEncoding: "utf-8" },
      experimental: {
        seseragi: { protocolVersion: 1, analysisSchemaVersion: 1 },
      },
    }
    running = false
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
      this.listener?.({ newState: State.Starting })
      this.running = true
      this.listener?.({ newState: State.Running })
    }

    needsStop() {
      return this.running
    }

    isRunning() {
      return this.running
    }

    async stop() {
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
    versionReader: async () => options.serverVersion ?? version,
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
    lines,
    MockLanguageClient,
    output,
    status,
  }
}

describe("official VS Code extension contract", () => {
  test("registers every .ssrg file and keeps the upgrade-compatible extension ID", () => {
    expect(`${manifest.publisher}.${manifest.name}`).toBe(
      "seseragi-dev.seseragi-spec-preview"
    )
    expect(manifest.displayName).toBe("Seseragi")
    expect(manifest.contributes.languages).toEqual([
      expect.objectContaining({
        id: "seseragi",
        aliases: ["Seseragi"],
        extensions: [".ssrg"],
      }),
    ])
    expect(JSON.stringify(manifest)).not.toContain("examples/spec/**")
    expect(
      manifest.contributes.configuration.properties[
        "seseragi.languageServer.path"
      ].default
    ).toBe("")
  })

  test("maps every release platform to one bundled server", () => {
    expect(platformTarget("darwin", "arm64")).toBe("darwin-arm64")
    expect(platformTarget("darwin", "x64")).toBe("darwin-x64")
    expect(platformTarget("linux", "x64")).toBe("linux-x64")
    expect(platformTarget("win32", "x64")).toBe("win32-x64")
    expect(platformTarget("linux", "arm64")).toBeUndefined()
    expect(serverBinaryName("win32")).toBe("seseragi-lsp.exe")
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
    expect(validateServerVersion(version)).toEqual(version)
    expect(() =>
      validateServerVersion({ ...version, protocolVersion: 2 })
    ).toThrow("extension/server version mismatch")
    expect(() =>
      validateInitializeResult({
        serverInfo: { name: "seseragi-lsp", version: "0.3.0" },
        experimental: {
          seseragi: { protocolVersion: 1, analysisSchemaVersion: 2 },
        },
      })
    ).toThrow("extension/server version mismatch")
  })

  test("starts the bundled server for file and untitled documents with visible state", async () => {
    const harness = extensionHarness()
    await harness.controller.activate(harness.context)

    expect(harness.errors).toEqual([])
    expect(harness.MockLanguageClient.instances).toHaveLength(1)
    const instance = harness.MockLanguageClient.instances[0]
    expect(instance.serverOptions).toEqual({
      command: path.join(
        "/extension",
        "server",
        "darwin-arm64",
        "seseragi-lsp"
      ),
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
    expect(harness.lines.join("\n")).toContain(
      "initialized; position encoding utf-8"
    )
    expect(harness.status.text).toContain("Seseragi")
    expect(harness.status.tooltip).toContain("0.3.0")
    expect(harness.commands.has("seseragi.restartLanguageServer")).toBe(true)
    expect(harness.commands.has("seseragi.showLanguageServerOutput")).toBe(true)

    await harness.commands.get("seseragi.restartLanguageServer")?.()
    expect(instance.stopped).toBe(true)
    expect(harness.MockLanguageClient.instances).toHaveLength(2)
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
