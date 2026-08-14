import { describe, expect, test } from "bun:test"
import { EventEmitter } from "node:events"
import path from "node:path"
import manifest from "../package.json"

const {
  createProjectCommandController,
  resolveProjectRoot,
  validateCliVersion,
} = require("../project-commands")

class MockStream extends EventEmitter {
  write(text: string) {
    this.emit("data", Buffer.from(text))
  }
}

class MockChild extends EventEmitter {
  stdout = new MockStream()
  stderr = new MockStream()
  killedWith: string | undefined

  kill(signal?: string) {
    this.killedWith = signal
    queueMicrotask(() => this.emit("close", 0, signal ?? null))
    return true
  }
}

function projectHarness() {
  const commands = new Map<string, (...args: unknown[]) => unknown>()
  const children: MockChild[] = []
  const spawns: Array<{
    command: string
    arguments: string[]
    options: { cwd: string }
  }> = []
  const lines: string[] = []
  const opened: string[] = []
  const information: string[] = []
  const errors: string[] = []
  const watchers: Array<{
    pattern: string
    change?: (uri: { fsPath: string }) => void
  }> = []
  let workspaceChange: (() => void) | undefined
  const status = { text: "", tooltip: "", command: "", show() {} }
  const output = {
    append(value: string) {
      lines.push(value)
    },
    appendLine(value: string) {
      lines.push(`${value}\n`)
    },
    show() {},
    dispose() {},
  }
  const workspace = path.resolve("/workspace")
  const project = path.join(workspace, "apps", "site")
  const source = path.join(project, "src/main.ssrg")
  const vscode = {
    StatusBarAlignment: { Left: 1 },
    Uri: {
      parse(value: string) {
        return { value }
      },
    },
    env: {
      async openExternal(uri: { value: string }) {
        opened.push(uri.value)
        return true
      },
    },
    workspace: {
      workspaceFolders: [{ uri: { fsPath: workspace } }],
      getConfiguration() {
        return {
          get(_key: string, fallback: string) {
            return "/tools/seseragi" || fallback
          },
        }
      },
      createFileSystemWatcher(pattern: string) {
        const watcher: {
          pattern: string
          change?: (uri: { fsPath: string }) => void
        } = { pattern }
        watchers.push(watcher)
        return {
          onDidChange(callback: (uri: { fsPath: string }) => void) {
            watcher.change = callback
          },
          onDidCreate() {},
          onDidDelete() {},
          dispose() {},
        }
      },
      onDidChangeWorkspaceFolders(callback: () => void) {
        workspaceChange = callback
        return { dispose() {} }
      },
    },
    window: {
      activeTextEditor: {
        document: { uri: { scheme: "file", fsPath: source } },
      },
      createOutputChannel() {
        return output
      },
      createStatusBarItem() {
        return status
      },
      async showInformationMessage(message: string) {
        information.push(message)
      },
      async showErrorMessage(message: string) {
        errors.push(message)
      },
    },
    commands: {
      registerCommand(
        command: string,
        callback: (...args: unknown[]) => unknown
      ) {
        commands.set(command, callback)
        return { dispose() {} }
      },
      async executeCommand() {},
    },
  }
  const controller = createProjectCommandController({
    vscode,
    processSpawner(
      command: string,
      arguments_: string[],
      options: { cwd: string }
    ) {
      const child = new MockChild()
      children.push(child)
      spawns.push({ command, arguments: arguments_, options })
      return child
    },
    versionReader: async () => ({
      name: "seseragi",
      version: manifest.version,
      target: "aarch64-apple-darwin",
    }),
    expectedTarget: "aarch64-apple-darwin",
    existsSync(candidate: string) {
      return candidate === path.join(project, "seseragi.toml")
    },
    platform: "darwin",
  })
  const context = { subscriptions: [] as unknown[] }
  return {
    children,
    commands,
    context,
    controller,
    errors,
    information,
    lines,
    opened,
    output,
    project,
    source,
    spawns,
    status,
    watchers,
    workspaceClosed() {
      vscode.workspace.workspaceFolders = []
      workspaceChange?.()
    },
  }
}

async function waitFor(condition: () => boolean) {
  for (let index = 0; index < 20; index += 1) {
    if (condition()) return
    await new Promise((resolve) => setTimeout(resolve, 0))
  }
  throw new Error("timed out waiting for test condition")
}

describe("VS Code project command integration", () => {
  test("validates CLI identity, version, and target", () => {
    const version = {
      name: "seseragi",
      version: manifest.version,
      target: "aarch64-apple-darwin",
    }
    expect(validateCliVersion(version, "aarch64-apple-darwin")).toEqual(
      version
    )
    expect(() =>
      validateCliVersion({ ...version, name: "seseragi-lsp" })
    ).toThrow("not the Seseragi CLI")
    expect(() =>
      validateCliVersion({ ...version, version: "0.3.0" })
    ).toThrow("extension/CLI version mismatch")
    expect(() =>
      validateCliVersion(version, "x86_64-apple-darwin")
    ).toThrow("extension/CLI target mismatch")
  })

  test("resolves the nearest manifest inside the owning workspace", () => {
    const workspace = path.resolve("/workspace")
    const project = path.join(workspace, "apps", "site")
    expect(
      resolveProjectRoot({
        resourcePath: path.join(project, "src", "main.ssrg"),
        workspaceFolders: [{ uri: { fsPath: workspace } }],
        existsSync(candidate: string) {
          return candidate === path.join(project, "seseragi.toml")
        },
      })
    ).toBe(project)
    expect(() =>
      resolveProjectRoot({
        resourcePath: path.resolve("/outside/main.ssrg"),
        workspaceFolders: [{ uri: { fsPath: workspace } }],
      })
    ).toThrow("outside the open workspace")
  })

  test("registers commands and runs canonical CLI processes without terminals", async () => {
    const harness = projectHarness()
    await harness.controller.activate(harness.context)
    expect([...harness.commands.keys()]).toEqual([
      "seseragi.showProjectOutput",
      "seseragi.runProject",
      "seseragi.buildWebApp",
      "seseragi.startDevelopmentServer",
      "seseragi.stopDevelopmentServer",
      "seseragi.openInBrowser",
    ])

    const run = harness.commands.get("seseragi.runProject")?.()
    await waitFor(() => harness.spawns.length === 1)
    expect(harness.spawns[0]).toEqual({
      command: "/tools/seseragi",
      arguments: ["run", harness.project],
      options: expect.objectContaining({ cwd: harness.project }),
    })
    harness.children[0].stdout.write("program output\n")
    harness.children[0].emit("close", 0, null)
    await run

    const build = harness.commands.get("seseragi.buildWebApp")?.()
    await waitFor(() => harness.spawns.length === 2)
    expect(harness.spawns[1].arguments).toEqual([
      "build",
      harness.project,
      "--target",
      "web",
      "--out-dir",
      "dist",
    ])
    harness.children[1].emit("close", 0, null)
    await build
    expect(harness.lines.join("")).toContain("program output")
  })

  test("owns one dev process, exposes its URL, rebuild state, and cleanup", async () => {
    const harness = projectHarness()
    await harness.controller.activate(harness.context)
    await harness.commands.get("seseragi.startDevelopmentServer")?.()
    const child = harness.children[0]
    expect(harness.spawns[0].arguments).toEqual([
      "dev",
      harness.project,
    ])
    child.stdout.write("Built web app (100 ms; reload 1)\n")
    child.stdout.write("Dev server: http://127.0.0.1:3000/\n")
    expect(harness.status.text).toContain("Seseragi Dev")
    expect(harness.status.tooltip).toContain("http://127.0.0.1:3000/")

    harness.watchers[0].change?.({ fsPath: harness.source })
    expect(harness.status.tooltip).toContain("rebuilding")
    child.stdout.write("Built web app (90 ms; reload 2)\n")
    expect(harness.status.tooltip).toContain("http://127.0.0.1:3000/")
    child.stderr.write("error[SES-N0001]\nBuild failed\n")
    expect(harness.status.tooltip).toContain("compiler diagnostics")
    child.stdout.write("Built web app (80 ms; reload 3)\n")
    expect(harness.status.tooltip).toContain("http://127.0.0.1:3000/")

    await harness.commands.get("seseragi.openInBrowser")?.()
    expect(harness.opened).toEqual(["http://127.0.0.1:3000/"])
    await harness.commands.get("seseragi.startDevelopmentServer")?.()
    expect(harness.children).toHaveLength(1)
    expect(harness.information.join("\n")).toContain("already running")

    harness.workspaceClosed()
    expect(child.killedWith).toBe("SIGINT")
    await harness.controller.deactivate()
    expect(harness.children).toHaveLength(1)
  })
})
