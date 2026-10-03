import { describe, expect, test } from "bun:test"
import seed from "../../../examples/spec/fixtures/playground/binding-workspace.json"
import type { BindingConversionResponse } from "../src/compiler/interop-types"
import {
  applyBindingConversion,
  bindingWorkspaceDiagnostics,
  workspaceBindingRequest,
  workspaceBindingsAreCurrent,
} from "../src/workspace/binding-conversion"
import {
  activateWorkspaceFile,
  createWorkspace,
  deleteWorkspacePath,
  renameWorkspacePath,
  setWorkspaceEntryFile,
  updateWorkspaceFileSource,
} from "../src/workspace/model"
import {
  persistWorkspace,
  restoreWorkspace,
  type WorkspaceStorage,
} from "../src/workspace/persistence"
import { workspaceProjectRequest } from "../src/workspace/project-request"

const responseFor = (
  state = createWorkspace(seed)
): BindingConversionResponse => ({
  schema: 1,
  revision: workspaceBindingRequest(state).revision,
  status: "success",
  generated: [
    {
      entry: "api",
      declaration: "host/index.d.ts",
      output: "fixture-api",
      source: "pub let version: Int = 1\n",
      metadata: '{"schema":1}',
      report: '{"diagnostics":[]}',
    },
  ],
  diagnostics: [],
})

function storage(): WorkspaceStorage {
  const values = new Map<string, string>()
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value)
    },
    removeItem: (key) => {
      values.delete(key)
    },
  }
}

const origin = { id: "playground-blank", workspaceHash: "workspace:blank-v1" }

describe("binding workspace artifacts", () => {
  test("restores every conversion input and generated artifact without a hidden form", () => {
    const initial = createWorkspace(seed)
    const converted = applyBindingConversion(
      initial,
      responseFor(initial)
    ).state
    const saved = storage()
    expect(persistWorkspace(saved, origin, converted, "").status).toBe("saved")
    const restored = restoreWorkspace(saved, [origin])
    expect(restored.status).toBe("restored")
    if (restored.status !== "restored") throw new Error("restore failed")
    expect(workspaceBindingRequest(restored.workspace)).toEqual(
      workspaceBindingRequest(converted)
    )
    expect(workspaceBindingsAreCurrent(restored.workspace)).toBe(true)
    expect(
      restored.workspace.files.some(
        ({ path }) => path === "gen/fixture-api.binding.json"
      )
    ).toBe(true)
    expect(restored.workspace.activeFile).toBe("gen/fixture-api.ssrg")
  })

  test("rejects late results after edits, rename, deletion, or entry changes", () => {
    const initial = createWorkspace(seed)
    const response = responseFor(initial)
    const mutations = [
      updateWorkspaceFileSource(initial, "main.ssrg", "new source"),
      updateWorkspaceFileSource(initial, "host/index.d.ts", "new declaration"),
      renameWorkspacePath(initial, "host/index.d.ts", "host/renamed.d.ts"),
      deleteWorkspacePath(initial, "host/index.d.ts"),
      setWorkspaceEntryFile(initial, undefined),
    ]
    for (const changed of mutations) {
      const applied = applyBindingConversion(changed, response)
      expect(applied.status).toBe("stale")
      expect(applied.state).toBe(changed)
      expect(
        applied.state.files.some(({ path }) => path.startsWith("gen/"))
      ).toBe(false)
    }
    const switched = activateWorkspaceFile(initial, "seseragi.toml")
    expect(applyBindingConversion(switched, response).status).toBe("applied")
  })

  test("compiles only source artifacts and protects generated code from stale inputs", () => {
    const initial = createWorkspace(seed)
    const converted = applyBindingConversion(
      initial,
      responseFor(initial)
    ).state
    const request = workspaceProjectRequest(converted)
    expect(request.manifest).toBe(
      seed.files.find(({ path }) => path === "seseragi.toml")?.source
    )
    expect(request.files.map(({ path, root }) => [path, root])).toEqual([
      ["gen/fixture-api.ssrg", "generated"],
      ["main.ssrg", undefined],
    ])
    const sourceEdit = updateWorkspaceFileSource(
      converted,
      "main.ssrg",
      "new source"
    )
    expect(workspaceBindingsAreCurrent(sourceEdit)).toBe(true)
    expect(workspaceProjectRequest(sourceEdit).files.length).toBe(2)
    const changed = renameWorkspacePath(
      converted,
      "host/index.d.ts",
      "host/renamed.d.ts"
    )
    expect(workspaceBindingsAreCurrent(changed)).toBe(false)
    expect(() => workspaceProjectRequest(changed)).toThrow(
      "Convert bindings again"
    )
    expect(() => setWorkspaceEntryFile(initial, "host/index.d.ts")).toThrow(
      ".ssrg"
    )
  })

  test("a failed conversion cannot replace previous output and diagnostics preserve identity", () => {
    const initial = createWorkspace(seed)
    const converted = applyBindingConversion(
      initial,
      responseFor(initial)
    ).state
    const failed: BindingConversionResponse = {
      ...responseFor(converted),
      status: "failure",
      generated: [],
      diagnostics: [
        {
          entry: "api",
          code: "SES-D1001",
          severity: "error",
          message: "Unsupported declaration",
          path: "host/index.d.ts",
          start: 7,
          end: 11,
          symbol: "Config",
        },
      ],
    }
    const applied = applyBindingConversion(converted, failed)
    expect(applied.status).toBe("failure")
    expect(applied.state).toBe(converted)
    const [diagnostic] = bindingWorkspaceDiagnostics(converted, failed)
    expect(diagnostic?.path).toBe("host/index.d.ts")
    expect(diagnostic?.diagnostic.primary).toEqual({ start: 7, end: 11 })
    expect(diagnostic?.diagnostic.notes).toEqual([
      "Binding entry: api",
      "Declaration: Config",
    ])
  })
})
