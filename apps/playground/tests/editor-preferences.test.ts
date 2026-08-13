import { describe, expect, test } from "bun:test"
import {
  createEditorPreferencesStore,
  defaultEditorPreferences,
  editorPreferencesStorageKey,
  loadEditorPreferences,
} from "../src/preferences/editor-preferences"

class MemoryStorage {
  readonly values = new Map<string, string>()

  getItem(key: string): string | null {
    return this.values.get(key) ?? null
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value)
  }
}

describe("shared editor preferences", () => {
  test("defaults to Auto width with the canonical fixed fallback", () => {
    expect(loadEditorPreferences(new MemoryStorage())).toEqual(
      defaultEditorPreferences
    )
    expect(defaultEditorPreferences.formatWidth).toEqual({
      mode: "auto",
      fixed: 88,
    })
  })

  test("migrates the previous Playground whitespace preference", () => {
    const storage = new MemoryStorage()
    storage.setItem("seseragi.playground.showWhitespace", "true")

    expect(loadEditorPreferences(storage).showWhitespace).toBe(true)
  })

  test("round-trips one preference model for Playground and Tour", () => {
    const storage = new MemoryStorage()
    const playground = createEditorPreferencesStore(storage)
    playground.set({
      showWhitespace: true,
      formatWidth: { mode: "fixed", fixed: 72 },
    })

    const tour = createEditorPreferencesStore(storage)
    expect(tour.get()).toEqual({
      showWhitespace: true,
      formatWidth: { mode: "fixed", fixed: 72 },
    })
    expect(storage.getItem(editorPreferencesStorageKey)).not.toBeNull()
  })

  test("recovers invalid fixed values without persisting an unsafe request", () => {
    const storage = new MemoryStorage()
    storage.setItem(
      editorPreferencesStorageKey,
      JSON.stringify({
        showWhitespace: true,
        formatWidth: { mode: "fixed", fixed: 12 },
      })
    )

    expect(loadEditorPreferences(storage)).toEqual({
      showWhitespace: true,
      formatWidth: { mode: "fixed", fixed: 88 },
    })
  })
})
