export const editorPreferencesStorageKey = "seseragi.editor.preferences.v1"
export const minimumFormatWidth = 40
export const maximumFormatWidth = 160
export const defaultFormatWidth = 88

const legacyWhitespaceStorageKey = "seseragi.playground.showWhitespace"

export type FormatWidthPreference = Readonly<{
  mode: "auto" | "fixed"
  fixed: number
}>

export type EditorPreferences = Readonly<{
  showWhitespace: boolean
  formatWidth: FormatWidthPreference
}>

export const defaultEditorPreferences: EditorPreferences = {
  showWhitespace: false,
  formatWidth: { mode: "auto", fixed: defaultFormatWidth },
}

type PreferencesStorage = Pick<Storage, "getItem" | "setItem">
type PreferencesListener = (preferences: EditorPreferences) => void

export type EditorPreferencesStore = Readonly<{
  get: () => EditorPreferences
  set: (preferences: EditorPreferences) => void
  update: (
    updatePreferences: (preferences: EditorPreferences) => EditorPreferences
  ) => void
  subscribe: (listener: PreferencesListener) => () => void
}>

export function createEditorPreferencesStore(
  storage: PreferencesStorage,
  eventTarget?: Window
): EditorPreferencesStore {
  let current = loadEditorPreferences(storage)
  const listeners = new Set<PreferencesListener>()

  const notify = (): void => {
    for (const listener of listeners) listener(current)
  }

  const set = (preferences: EditorPreferences): void => {
    current = normalizeEditorPreferences(preferences)
    try {
      storage.setItem(editorPreferencesStorageKey, JSON.stringify(current))
    } catch {
      // Preferences remain usable for this page when persistence is unavailable.
    }
    notify()
  }

  eventTarget?.addEventListener("storage", (event) => {
    if (event.key !== editorPreferencesStorageKey) return
    current = parseEditorPreferences(event.newValue) ?? defaultEditorPreferences
    notify()
  })

  return {
    get: () => current,
    set,
    update: (updatePreferences) => set(updatePreferences(current)),
    subscribe: (listener) => {
      listeners.add(listener)
      listener(current)
      return () => listeners.delete(listener)
    },
  }
}

export function loadEditorPreferences(
  storage: Pick<Storage, "getItem">
): EditorPreferences {
  try {
    const stored = parseEditorPreferences(
      storage.getItem(editorPreferencesStorageKey)
    )
    if (stored !== undefined) return stored
    return {
      ...defaultEditorPreferences,
      showWhitespace: storage.getItem(legacyWhitespaceStorageKey) === "true",
    }
  } catch {
    return defaultEditorPreferences
  }
}

export function isValidFixedFormatWidth(value: number): boolean {
  return (
    Number.isInteger(value) &&
    value >= minimumFormatWidth &&
    value <= maximumFormatWidth
  )
}

function parseEditorPreferences(
  value: string | null
): EditorPreferences | undefined {
  if (value === null) return undefined
  try {
    return normalizeEditorPreferences(JSON.parse(value) as unknown)
  } catch {
    return undefined
  }
}

function normalizeEditorPreferences(value: unknown): EditorPreferences {
  if (typeof value !== "object" || value === null) {
    return defaultEditorPreferences
  }
  const candidate = value as {
    showWhitespace?: unknown
    formatWidth?: { mode?: unknown; fixed?: unknown }
  }
  const fixed = Number(candidate.formatWidth?.fixed)
  return {
    showWhitespace:
      typeof candidate.showWhitespace === "boolean"
        ? candidate.showWhitespace
        : defaultEditorPreferences.showWhitespace,
    formatWidth: {
      mode: candidate.formatWidth?.mode === "fixed" ? "fixed" : "auto",
      fixed: isValidFixedFormatWidth(fixed) ? fixed : defaultFormatWidth,
    },
  }
}
