import { describe, expect, test } from "bun:test"
import { createWorkspace } from "../src/workspace/model"
import {
  confirmDirtyWorkspaceSwitch,
  persistWorkspace,
  restoreWorkspace,
  type WorkspaceStorage,
  workspacePersistenceKey,
  workspacePersistenceSchema,
} from "../src/workspace/persistence"

const sample = {
  id: "project-greeting",
  workspaceHash: "sha256:sample",
}
const blankOrigin = {
  id: "playground-blank",
  workspaceHash: "workspace:blank-v1",
}

function memoryStorage(): WorkspaceStorage & {
  readonly values: Map<string, string>
} {
  const values = new Map<string, string>()
  return {
    values,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value)
    },
    removeItem: (key) => {
      values.delete(key)
    },
  }
}

describe("workspace local persistence", () => {
  test("round-trips project files, tabs, dirty state and Explorer state", () => {
    const storage = memoryStorage()
    const workspace = createWorkspace({
      files: [
        { path: "main.ssrg", source: "changed main" },
        { path: "feature/greeting.ssrg", source: "changed greeting" },
      ],
      entryFile: "main.ssrg",
      activeFile: "feature/greeting.ssrg",
      openFiles: ["main.ssrg", "feature/greeting.ssrg"],
      dirtyFiles: ["feature/greeting.ssrg"],
      expandedFolders: ["feature"],
      explorer: { visible: true, width: 312 },
    })

    expect(persistWorkspace(storage, sample, workspace, "Morishita\n")).toEqual(
      { status: "saved" }
    )
    expect(restoreWorkspace(storage, [sample])).toEqual({
      status: "restored",
      sampleId: "project-greeting",
      workspace,
      stdin: "Morishita\n",
    })
  })

  test("restores Blank independently from the canonical starter", () => {
    const storage = memoryStorage()
    const workspace = createWorkspace({
      files: [{ path: "main.ssrg", source: "" }],
      entryFile: "main.ssrg",
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
    })

    expect(persistWorkspace(storage, blankOrigin, workspace, "")).toEqual({
      status: "saved",
    })
    expect(restoreWorkspace(storage, [sample, blankOrigin])).toEqual({
      status: "restored",
      sampleId: "playground-blank",
      workspace,
      stdin: "",
    })
  })

  test("drops corrupted, incompatible and stale sample data safely", () => {
    const storage = memoryStorage()
    for (const value of [
      "{broken",
      JSON.stringify({ schema: workspacePersistenceSchema + 1 }),
      JSON.stringify({
        schema: workspacePersistenceSchema,
        sampleId: sample.id,
        sampleHash: "sha256:old",
        workspace: {},
        stdin: "",
      }),
    ]) {
      storage.values.set(workspacePersistenceKey, value)
      expect(restoreWorkspace(storage, [sample])).toMatchObject({
        status: "recovered",
      })
      expect(storage.getItem(workspacePersistenceKey)).toBeNull()
    }
  })

  test("normalizes older NFC-equivalent paths but recovers collision data without overwrite", () => {
    const storage = memoryStorage()
    storage.values.set(
      workspacePersistenceKey,
      JSON.stringify({
        schema: workspacePersistenceSchema,
        sampleId: sample.id,
        sampleHash: sample.workspaceHash,
        workspace: {
          files: [{ path: "feature/cafe\u0301.ssrg", source: "changed" }],
          folders: ["feature"],
          entryFile: "feature/cafe\u0301.ssrg",
          activeFile: "feature/cafe\u0301.ssrg",
          openFiles: ["feature/cafe\u0301.ssrg"],
          dirtyFiles: ["feature/cafe\u0301.ssrg"],
          expandedFolders: ["feature"],
          explorer: { visible: true, width: 240 },
        },
        stdin: "",
      })
    )

    const normalized = restoreWorkspace(storage, [sample])
    expect(normalized.status).toBe("restored")
    if (normalized.status !== "restored") return
    expect(normalized.workspace.files.map(({ path }) => path)).toEqual([
      "feature/café.ssrg",
    ])
    expect(normalized.workspace.entryFile).toBe("feature/café.ssrg")
    expect(normalized.workspace.activeFile).toBe("feature/café.ssrg")
    expect(normalized.workspace.openFiles).toEqual(["feature/café.ssrg"])
    expect(normalized.workspace.dirtyFiles).toEqual(["feature/café.ssrg"])

    storage.values.set(
      workspacePersistenceKey,
      JSON.stringify({
        schema: workspacePersistenceSchema,
        sampleId: sample.id,
        sampleHash: sample.workspaceHash,
        workspace: {
          files: [
            { path: "café.ssrg", source: "first" },
            { path: "cafe\u0301.ssrg", source: "second" },
          ],
          folders: [],
          openFiles: [],
          dirtyFiles: [],
          expandedFolders: [],
          explorer: { visible: false, width: 240 },
        },
        stdin: "",
      })
    )

    expect(restoreWorkspace(storage, [sample])).toMatchObject({
      status: "recovered",
      diagnostic: expect.stringContaining("安全に戻しました"),
    })
    expect(storage.getItem(workspacePersistenceKey)).toBeNull()
  })

  test("reports quota failures without changing the active workspace", () => {
    const storage = memoryStorage()
    storage.setItem = () => {
      throw new DOMException("quota", "QuotaExceededError")
    }
    const workspace = createWorkspace({
      files: [{ path: "main.ssrg", source: "main" }],
      entryFile: "main.ssrg",
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
    })

    expect(persistWorkspace(storage, sample, workspace, "")).toMatchObject({
      status: "failure",
      diagnostic: expect.stringContaining("保存容量"),
    })
    expect(workspace.files[0]?.source).toBe("main")
  })

  test("confirms only when switching away from dirty files", () => {
    const clean = createWorkspace({
      files: [{ path: "main.ssrg", source: "main" }],
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
    })
    const dirty = createWorkspace({
      files: [{ path: "main.ssrg", source: "changed" }],
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
      dirtyFiles: ["main.ssrg"],
    })
    const prompts: string[] = []
    const confirm = (message: string): boolean => {
      prompts.push(message)
      return false
    }

    expect(confirmDirtyWorkspaceSwitch(clean, "Next", confirm)).toBe(true)
    expect(confirmDirtyWorkspaceSwitch(dirty, "Next", confirm)).toBe(false)
    expect(prompts).toEqual([expect.stringContaining("main.ssrg")])
  })
})
