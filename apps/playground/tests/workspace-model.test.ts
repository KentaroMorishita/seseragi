import { describe, expect, test } from "bun:test"
import {
  activateWorkspaceFile,
  activeWorkspaceSource,
  closeWorkspaceFile,
  createSingleFileWorkspace,
  createWorkspace,
  createWorkspaceFile,
  createWorkspaceFolder,
  deleteWorkspacePath,
  markWorkspaceFileClean,
  maximumExplorerWidth,
  minimumExplorerWidth,
  renameWorkspacePath,
  setWorkspaceEntryFile,
  setWorkspaceExplorer,
  setWorkspaceFolderExpanded,
  updateActiveWorkspaceSource,
  workspacePath,
  workspaceSourceIdentity,
} from "../src/workspace/model"

describe("Playground virtual workspace", () => {
  test("loads a single-file sample as a clean main module", () => {
    const state = createSingleFileWorkspace('pub effect fn main = println "ok"')

    expect(state.files).toEqual([
      { path: "main.ssrg", source: 'pub effect fn main = println "ok"' },
    ])
    expect(state.folders).toEqual([])
    expect(state.entryFile).toBe("main.ssrg")
    expect(state.entryModule).toBe("main")
    expect(state.activeFile).toBe("main.ssrg")
    expect(state.openFiles).toEqual(["main.ssrg"])
    expect(state.dirtyFiles).toEqual([])
    expect(state.expandedFolders).toEqual([])
    expect(state.explorer).toEqual({ visible: false, width: 240 })
    expect(activeWorkspaceSource(state)).toContain("println")
  })

  test("rejects paths that are not normalized workspace-relative paths", () => {
    expect(workspacePath("feature/counter.ssrg")).toBe("feature/counter.ssrg")
    expect(workspacePath("feature/cafe\u0301")).toBe("feature/café")
    for (const path of [
      "",
      "/main.ssrg",
      "main.ssrg/",
      "feature//main.ssrg",
      "./main.ssrg",
      "feature/../main.ssrg",
      "feature\\main.ssrg",
    ]) {
      expect(() => workspacePath(path)).toThrow()
    }
  })

  test("matches the compiler source identity before state is created", () => {
    expect(workspaceSourceIdentity("domain/cafe\u0301.ssrg")).toEqual({
      path: "domain/café.ssrg",
      module: "domain/café",
    })

    for (const path of [
      "",
      ".ssrg",
      "main",
      "main.ssrg.ssrg",
      "/main.ssrg",
      "main.ssrg/",
      "feature//main.ssrg",
      "./main.ssrg",
      "feature/../main.ssrg",
      "feature\\main.ssrg",
      "main\0.ssrg",
    ]) {
      expect(() => workspaceSourceIdentity(path)).toThrow()
    }
  })

  test("rejects normalized path and file-folder collisions without changing state", () => {
    expect(() =>
      createWorkspace({
        files: [
          { path: "domain/café.ssrg", source: "first" },
          { path: "domain/cafe\u0301.ssrg", source: "second" },
        ],
      })
    ).toThrow("Duplicate workspace file path")
    expect(() =>
      createWorkspace({
        files: [{ path: "main.ssrg", source: "main" }],
        folders: ["main.ssrg"],
      })
    ).toThrow("both a file and folder")

    const initial = createSingleFileWorkspace("main")
    expect(() => createWorkspaceFile(initial, ".ssrg")).toThrow()
    expect(() => renameWorkspacePath(initial, "main.ssrg", "main")).toThrow(
      "end in .ssrg"
    )
    expect(initial.files).toEqual([{ path: "main.ssrg", source: "main" }])

    const named = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "café.ssrg", source: "café" },
      ],
    })
    expect(() =>
      renameWorkspacePath(named, "main.ssrg", "cafe\u0301.ssrg")
    ).toThrow("Duplicate workspace file path")
    expect(named.files.map(({ path }) => path)).toEqual([
      "café.ssrg",
      "main.ssrg",
    ])
  })

  test("normalizes nested file references and remaps them through a folder rename", () => {
    const initial = createWorkspace({
      files: [{ path: "feature/cafe\u0301.ssrg", source: "broken" }],
      entryFile: "feature/cafe\u0301.ssrg",
      activeFile: "feature/cafe\u0301.ssrg",
      openFiles: ["feature/cafe\u0301.ssrg"],
      dirtyFiles: ["feature/cafe\u0301.ssrg"],
      expandedFolders: ["feature"],
    })
    const renamed = renameWorkspacePath(initial, "feature", "domain")

    expect(initial.files.map(({ path }) => path)).toEqual(["feature/café.ssrg"])
    expect(renamed.files.map(({ path }) => path)).toEqual(["domain/café.ssrg"])
    expect(renamed.entryFile).toBe("domain/café.ssrg")
    expect(renamed.entryModule).toBe("domain/café")
    expect(renamed.activeFile).toBe("domain/café.ssrg")
    expect(renamed.openFiles).toEqual(["domain/café.ssrg"])
    expect(renamed.dirtyFiles).toEqual(["domain/café.ssrg"])
    expect(renamed.expandedFolders).toEqual(["domain"])
  })

  test("creates and edits files and folders without mutating prior state", () => {
    const initial = createSingleFileWorkspace("main")
    const withFolder = createWorkspaceFolder(initial, "feature")
    const withFile = createWorkspaceFile(
      withFolder,
      "feature/counter.ssrg",
      "counter"
    )
    const edited = updateActiveWorkspaceSource(withFile, "counter updated")

    expect(initial.folders).toEqual([])
    expect(withFolder.files).toHaveLength(1)
    expect(withFile.activeFile).toBe("feature/counter.ssrg")
    expect(withFile.openFiles).toEqual(["main.ssrg", "feature/counter.ssrg"])
    expect(edited.dirtyFiles).toEqual(["feature/counter.ssrg"])
    expect(activeWorkspaceSource(edited)).toBe("counter updated")
    expect(
      markWorkspaceFileClean(edited, "feature/counter.ssrg").dirtyFiles
    ).toEqual([])
    expect(() => createWorkspaceFile(withFile, "feature/counter.ssrg")).toThrow(
      "already exists"
    )
    expect(() => createWorkspaceFile(initial, "missing/counter.ssrg")).toThrow(
      "parent folder does not exist"
    )
  })

  test("renames a folder and every file, tab, dirty and entry reference", () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
        { path: "feature/nested/view.ssrg", source: "view" },
      ],
      entryFile: "feature/counter.ssrg",
      activeFile: "feature/counter.ssrg",
      openFiles: [
        "main.ssrg",
        "feature/counter.ssrg",
        "feature/nested/view.ssrg",
      ],
      dirtyFiles: ["feature/counter.ssrg", "feature/nested/view.ssrg"],
      expandedFolders: ["feature", "feature/nested"],
    })
    const renamed = renameWorkspacePath(initial, "feature", "modules")

    expect(renamed.files.map(({ path }) => path)).toEqual([
      "main.ssrg",
      "modules/counter.ssrg",
      "modules/nested/view.ssrg",
    ])
    expect(renamed.folders).toEqual(["modules", "modules/nested"])
    expect(renamed.entryFile).toBe("modules/counter.ssrg")
    expect(renamed.entryModule).toBe("modules/counter")
    expect(renamed.activeFile).toBe("modules/counter.ssrg")
    expect(renamed.openFiles).toEqual([
      "main.ssrg",
      "modules/counter.ssrg",
      "modules/nested/view.ssrg",
    ])
    expect(renamed.dirtyFiles).toEqual([
      "modules/counter.ssrg",
      "modules/nested/view.ssrg",
    ])
    expect(renamed.expandedFolders).toEqual(["modules", "modules/nested"])
    expect(initial.entryFile).toBe("feature/counter.ssrg")
  })

  test("selects the right tab, then left tab when deleting the active file", () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "a.ssrg", source: "a" },
        { path: "b.ssrg", source: "b" },
        { path: "c.ssrg", source: "c" },
      ],
      entryFile: "b.ssrg",
      activeFile: "b.ssrg",
      openFiles: ["main.ssrg", "a.ssrg", "b.ssrg", "c.ssrg"],
      dirtyFiles: ["b.ssrg"],
    })
    const withoutB = deleteWorkspacePath(initial, "b.ssrg")

    expect(withoutB.activeFile).toBe("c.ssrg")
    expect(withoutB.entryFile).toBeUndefined()
    expect(withoutB.entryModule).toBeUndefined()
    expect(withoutB.dirtyFiles).toEqual([])

    const withoutC = deleteWorkspacePath(withoutB, "c.ssrg")
    expect(withoutC.activeFile).toBe("a.ssrg")

    const onlyClosedFilesRemain = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "z.ssrg", source: "z" },
      ],
      activeFile: "z.ssrg",
      openFiles: ["z.ssrg"],
    })
    const fallback = deleteWorkspacePath(onlyClosedFilesRemain, "z.ssrg")
    expect(fallback.activeFile).toBe("main.ssrg")
    expect(fallback.openFiles).toEqual(["main.ssrg"])
  })

  test("deletes a folder subtree and clears every removed reference", () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
        { path: "feature/view.ssrg", source: "view" },
        { path: "other/keep.ssrg", source: "keep" },
      ],
      entryFile: "feature/counter.ssrg",
      activeFile: "feature/counter.ssrg",
      openFiles: ["main.ssrg", "feature/counter.ssrg", "other/keep.ssrg"],
      dirtyFiles: ["feature/counter.ssrg", "feature/view.ssrg"],
      expandedFolders: ["feature"],
    })
    const deleted = deleteWorkspacePath(initial, "feature")

    expect(deleted.files.map(({ path }) => path)).toEqual([
      "main.ssrg",
      "other/keep.ssrg",
    ])
    expect(deleted.folders).toEqual(["other"])
    expect(deleted.entryFile).toBeUndefined()
    expect(deleted.activeFile).toBe("other/keep.ssrg")
    expect(deleted.openFiles).toEqual(["main.ssrg", "other/keep.ssrg"])
    expect(deleted.dirtyFiles).toEqual([])
    expect(deleted.expandedFolders).toEqual([])
  })

  test("keeps rename and delete failures atomic", () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
        { path: "other/view.ssrg", source: "view" },
      ],
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
    })

    expect(() => renameWorkspacePath(initial, "feature", "other")).toThrow()
    expect(() =>
      renameWorkspacePath(initial, "feature", "feature/nested")
    ).toThrow("inside itself")
    expect(() => deleteWorkspacePath(initial, "missing.ssrg")).toThrow(
      "does not exist"
    )
    expect(() =>
      createWorkspace({
        files: [{ path: "main.ssrg", source: "main" }],
        folders: ["feature", "feature"],
      })
    ).toThrow("Duplicate workspace folder path")
    expect(initial.files.map(({ path }) => path)).toEqual([
      "feature/counter.ssrg",
      "main.ssrg",
      "other/view.ssrg",
    ])
  })

  test("tracks open files, entry selection and explorer state in one model", () => {
    let state = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
      ],
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
    })
    state = activateWorkspaceFile(state, "feature/counter.ssrg")
    state = closeWorkspaceFile(state, "feature/counter.ssrg")
    state = setWorkspaceEntryFile(state, "feature/counter.ssrg")
    state = setWorkspaceFolderExpanded(state, "feature", true)
    state = setWorkspaceExplorer(state, { visible: true, width: 999 })

    expect(state.activeFile).toBe("main.ssrg")
    expect(state.openFiles).toEqual(["main.ssrg"])
    expect(state.entryModule).toBe("feature/counter")
    expect(state.expandedFolders).toEqual(["feature"])
    expect(state.explorer).toEqual({
      visible: true,
      width: maximumExplorerWidth,
    })
    expect(setWorkspaceExplorer(state, { width: 1 }).explorer.width).toBe(
      minimumExplorerWidth
    )
  })

  test("drives every sample through the persisted workspace", async () => {
    const mainSource = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(mainSource).toContain("createWorkspace(defaultSample.workspace)")
    expect(mainSource).toContain('createSingleFileWorkspace("")')
    expect(mainSource).toContain("activeWorkspaceSource(workspaceState)")
    expect(mainSource).toContain("updateActiveWorkspaceSource")
    expect(mainSource).toContain("restoreWorkspace(localStorage, [")
    expect(mainSource).toContain("blankWorkspaceOrigin")
    expect(mainSource).toContain("persistCurrentWorkspace()")
    expect(mainSource).not.toMatch(/let source =/)
  })
})
