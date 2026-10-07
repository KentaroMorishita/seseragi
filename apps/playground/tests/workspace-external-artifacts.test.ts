import { expect, test } from "bun:test"
import {
  activateWorkspaceFile,
  createWorkspace,
  createWorkspaceFile,
  deleteWorkspacePath,
  renameWorkspacePath,
  setWorkspaceEntryFile,
  updateActiveWorkspaceSource,
} from "../src/workspace/model"
import {
  persistWorkspace,
  restoreWorkspace,
} from "../src/workspace/persistence"
import { workspaceProjectRequest } from "../src/workspace/project-request"

const seed = () =>
  createWorkspace({
    files: [
      { path: "main.ssrg", source: "pub fn main -> Int = 1" },
      { path: "host/index.d.ts", source: "export const value: number;" },
      { path: "bindings.toml", source: "schema = 1" },
      {
        path: "host/package.json",
        source: '{"name":"example","version":"1.0.0"}',
      },
    ],
    entryFile: "main.ssrg",
    activeFile: "host/index.d.ts",
    openFiles: ["main.ssrg", "host/index.d.ts"],
  })

test("external artifacts participate in editing and folder lifecycle without becoming source entries", () => {
  let state = seed()
  state = updateActiveWorkspaceSource(state, "export const value: string;")
  expect(state.dirtyFiles).toEqual(["host/index.d.ts"])
  state = renameWorkspacePath(state, "host", "vendor")
  expect(state.activeFile).toBe("vendor/index.d.ts")
  expect(state.entryFile).toBe("main.ssrg")
  expect(() => setWorkspaceEntryFile(state, "vendor/index.d.ts")).toThrow()
  expect(workspaceProjectRequest(state).files.map(({ path }) => path)).toEqual([
    "main.ssrg",
  ])
  state = deleteWorkspacePath(state, "vendor")
  expect(state.files.map(({ path }) => path)).toEqual([
    "bindings.toml",
    "main.ssrg",
  ])
  state = createWorkspaceFile(state, "custom.settings", "settings")
  expect(activateWorkspaceFile(state, "custom.settings").activeFile).toBe(
    "custom.settings"
  )
})

test("schema-1 persistence reproduces external inputs, active editor and dirty paths", () => {
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
  const sample = { id: "external", workspaceHash: "stable" }
  const state = updateActiveWorkspaceSource(
    seed(),
    "export const changed: number;"
  )
  expect(persistWorkspace(storage, sample, state, "").status).toBe("saved")
  const restored = restoreWorkspace(storage, [sample])
  expect(restored.status).toBe("restored")
  if (restored.status === "restored") expect(restored.workspace).toEqual(state)
})

test("artifact paths retain traversal and NFC collision protection", () => {
  for (const path of [
    "../index.d.ts",
    "/package.json",
    "a\\b.toml",
    "a//b.json",
    "x\0.json",
  ]) {
    expect(() => createWorkspace({ files: [{ path, source: "" }] })).toThrow()
  }
  expect(() =>
    createWorkspace({
      files: [
        { path: "café.d.ts", source: "a" },
        { path: "cafe\u0301.d.ts", source: "b" },
      ],
    })
  ).toThrow("Duplicate")
  expect(() =>
    createWorkspace({ ...seed(), entryFile: "host/index.d.ts" })
  ).toThrow()
})
