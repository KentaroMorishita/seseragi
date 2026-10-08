import { expect, test } from "bun:test"
import {
  type BindingResponse,
  publishWorkspaceBindings,
  workspaceBindingRequest,
} from "../src/workspace/bindings"
import {
  createWorkspace,
  deleteWorkspacePath,
  renameWorkspacePath,
  updateWorkspaceFileSource,
  type WorkspaceState,
} from "../src/workspace/model"
import {
  persistWorkspace,
  restoreWorkspace,
  workspacePersistenceKey,
} from "../src/workspace/persistence"
import {
  workspaceAnalysisRequest,
  workspaceProjectRequest,
} from "../src/workspace/project-request"

const initial = () =>
  createWorkspace({
    files: [
      { path: "main.ssrg", source: 'import * as api from "gen/api"\n' },
      { path: "input.d.ts", source: "export function value(): number;" },
      { path: "seseragi.toml", source: "config" },
    ],
    entryFile: "main.ssrg",
    activeFile: "main.ssrg",
    openFiles: ["main.ssrg"],
  })
function publish(state: WorkspaceState, outputs = ["api"]) {
  const request = workspaceBindingRequest(state)
  const response: BindingResponse = {
    schema: 1,
    revision: request.revision,
    status: "success",
    generatedRoot: "custom/bindings",
    diagnostics: [],
    generated: outputs.map((output) => ({
      id: output,
      output,
      source: "pub let value: Int = 42\n",
      metadata: "{}",
      report: "{}",
    })),
  }
  return publishWorkspaceBindings(state, request, response)
}

test("publication uses a separate owned set shared by analysis and compilation", () => {
  const before = initial(),
    state = publish(before)
  expect(state.files).toEqual(before.files)
  expect(state.openFiles).toEqual(before.openFiles)
  expect(state.entryFile).toBe("main.ssrg")
  expect(workspaceProjectRequest(state).generatedFiles).toEqual([
    {
      module: "api",
      path: "custom/bindings/api.ssrg",
      source: "pub let value: Int = 42\n",
    },
  ])
  expect(workspaceAnalysisRequest(state).project).toEqual(
    workspaceProjectRequest(state)
  )
  expect(
    workspaceProjectRequest(
      updateWorkspaceFileSource(state, "main.ssrg", "pub let answer: Int = 1\n")
    ).generatedFiles
  ).toEqual(state.generatedBindings?.files)
})

test("replacement removes disappeared entries and failed regeneration clears ownership", () => {
  const state = publish(publish(initial(), ["api", "old"]), ["new"])
  expect(state.generatedBindings?.files.map((file) => file.module)).toEqual([
    "new",
  ])
  const request = workspaceBindingRequest(state)
  const failed = publishWorkspaceBindings(state, request, {
    schema: 1,
    revision: request.revision,
    status: "failure",
    generated: [],
    diagnostics: [],
  })
  expect(workspaceProjectRequest(failed).generatedFiles).toBeUndefined()
})

test("declaration/config edits, renames and deletes invalidate generated artifacts, including edit-undo", () => {
  const state = publish(initial())
  const changed = updateWorkspaceFileSource(state, "input.d.ts", "changed")
  for (const invalid of [
    changed,
    updateWorkspaceFileSource(state, "seseragi.toml", "changed"),
    renameWorkspacePath(state, "input.d.ts", "renamed.d.ts"),
    deleteWorkspacePath(state, "input.d.ts"),
    updateWorkspaceFileSource(
      changed,
      "input.d.ts",
      "export function value(): number;"
    ),
  ]) {
    expect(invalid.generatedBindings).toBeUndefined()
    expect(workspaceProjectRequest(invalid).generatedFiles).toBeUndefined()
  }
})

test("stale requests and partial entry results cannot replace an owned set", () => {
  const state = publish(initial()),
    request = workspaceBindingRequest(state)
  const result: BindingResponse = {
    schema: 1,
    revision: request.revision,
    status: "success",
    generatedRoot: "generated",
    generated: [],
    diagnostics: [],
  }
  const changed = updateWorkspaceFileSource(state, "main.ssrg", "changed")
  expect(publishWorkspaceBindings(changed, request, result)).toBe(changed)
  expect(() =>
    publishWorkspaceBindings(state, { ...request, entry: "api" }, result)
  ).toThrow("all binding entries")
  expect(() =>
    publishWorkspaceBindings(state, request, { ...result, revision: "wrong" })
  ).toThrow("snapshot")
})

test("persistence restores owned artifacts only with the matching external inputs", () => {
  const values = new Map<string, string>()
  const storage = {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value)
    },
    removeItem: (key: string) => {
      values.delete(key)
    },
  }
  const origin = { id: "playground-blank", workspaceHash: "workspace:blank-v1" }
  const state = publish(initial())
  expect(persistWorkspace(storage, origin, state, "").status).toBe("saved")
  const restored = restoreWorkspace(storage, [origin])
  expect(restored.status).toBe("restored")
  if (restored.status !== "restored") throw new Error("restore failed")
  expect(restored.workspace.generatedBindings).toEqual(state.generatedBindings)
  const json = JSON.parse(values.get(workspacePersistenceKey) ?? "{}")
  json.workspace.files.find(
    (file: { path: string }) => file.path === "input.d.ts"
  ).source = "stale"
  values.set(workspacePersistenceKey, JSON.stringify(json))
  const stale = restoreWorkspace(storage, [origin])
  expect(stale.status).toBe("restored")
  if (stale.status !== "restored") throw new Error("restore failed")
  expect(stale.workspace.generatedBindings).toBeUndefined()
  json.workspace.generatedBindings = {
    inputRevision: "invalid",
    files: [{ path: "../escape.ssrg" }],
  }
  values.set(workspacePersistenceKey, JSON.stringify(json))
  const corrupted = restoreWorkspace(storage, [origin])
  expect(corrupted.status).toBe("restored")
  if (corrupted.status !== "restored") throw new Error("restore failed")
  expect(corrupted.workspace.generatedBindings).toBeUndefined()
  expect(corrupted.workspace.files).toEqual(json.workspace.files)
})
