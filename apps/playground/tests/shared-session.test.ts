import { expect, test } from "bun:test"
import { playgroundSession } from "../src/workspace/shared-session"
import { createSingleFileWorkspace } from "../src/workspace/model"
import {
  persistWorkspace,
  restoreWorkspace,
  workspacePersistenceKey,
} from "../src/workspace/persistence"

test("shared links have independent save slots and reload the edited copy", () => {
  const data = new Map<string, string>()
  const storage = {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => {
      data.set(k, v)
    },
    removeItem: (k: string) => {
      data.delete(k)
    },
  }
  const origin = { id: "blank", workspaceHash: "blank-v1" }
  persistWorkspace(
    storage,
    origin,
    createSingleFileWorkspace("original"),
    "private-marker"
  )
  const original = data.get(workspacePersistenceKey)
  const imported = playgroundSession(
    "https://example.test/?source=shared#editor"
  )
  expect(imported.persistenceKey).not.toBe(workspacePersistenceKey)
  expect(
    restoreWorkspace(storage, [origin], imported.persistenceKey).status
  ).toBe("empty")
  persistWorkspace(storage, origin, createSingleFileWorkspace("edited"), "", {
    key: imported.persistenceKey,
    applicationId: "workspace-one",
  })
  expect(data.get(workspacePersistenceKey)).toBe(original)
  const reloaded = playgroundSession(imported.importedUrl!)
  expect(reloaded.linkedSource).toBeUndefined()
  expect(reloaded.persistenceKey).toBe(imported.persistenceKey)
  const restored = restoreWorkspace(storage, [origin], reloaded.persistenceKey)
  expect(restored.status).toBe("restored")
  if (restored.status === "restored") {
    expect(
      restored.workspace.files.find(({ path }) => path === "main.ssrg")?.source
    ).toBe("edited")
    expect(restored.stdin).toBe("")
    expect(restored.applicationId).toBe("workspace-one")
  }
  expect(new URL(imported.importedUrl!).hash).toBe("#editor")
  expect(
    playgroundSession("https://example.test/?workspace=../../other")
      .persistenceKey
  ).toBe(workspacePersistenceKey)
})
