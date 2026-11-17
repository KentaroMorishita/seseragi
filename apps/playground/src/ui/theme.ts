// テーマ（apps/playground/DESIGN.md「4. デザイントークン」、#1350 の判断点 2）
//
// - 既定は OS の設定に合わせる（"system"）。tokens.css が prefers-color-scheme で切り替える。
// - 選んだテーマ（"light" / "dark"）はこのブラウザに保存し、<html data-theme> として
//   OS の設定より優先する。
// - 保存した選択は、描画の前に themeBootScript で当て、開いた瞬間のちらつきを防ぐ。

export const themeStorageKey = "seseragi.theme.v1"

export const themePreferences = ["system", "light", "dark"] as const
export type ThemePreference = (typeof themePreferences)[number]
export type Theme = "light" | "dark"

type ThemeStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">
type ThemeRoot = Readonly<{ dataset: DOMStringMap }>
type ColorSchemeQuery = Readonly<{
  matches: boolean
  addEventListener: (type: "change", listener: () => void) => void
  removeEventListener: (type: "change", listener: () => void) => void
}>
type ThemeListener = (state: ThemeState) => void

export type ThemeState = Readonly<{
  preference: ThemePreference
  theme: Theme
}>

export type ThemeController = Readonly<{
  get: () => ThemeState
  set: (preference: ThemePreference) => void
  subscribe: (listener: ThemeListener) => () => void
  dispose: () => void
}>

export function parseThemePreference(value: unknown): ThemePreference {
  return value === "light" || value === "dark" ? value : "system"
}

export function readThemePreference(storage: ThemeStorage): ThemePreference {
  try {
    return parseThemePreference(storage.getItem(themeStorageKey))
  } catch {
    return "system"
  }
}

export function writeThemePreference(
  storage: ThemeStorage,
  preference: ThemePreference
): void {
  try {
    if (preference === "system") storage.removeItem(themeStorageKey)
    else storage.setItem(themeStorageKey, preference)
  } catch {
    // 保存できなくても、このページの中では選んだテーマを使い続ける。
  }
}

export function resolveTheme(
  preference: ThemePreference,
  systemPrefersDark: boolean
): Theme {
  if (preference === "system") return systemPrefersDark ? "dark" : "light"
  return preference
}

export function applyThemePreference(
  root: ThemeRoot,
  preference: ThemePreference
): void {
  if (preference === "system") delete root.dataset.theme
  else root.dataset.theme = preference
}

export function createThemeController(
  options: Readonly<{
    storage: ThemeStorage
    root: ThemeRoot
    query: ColorSchemeQuery
    eventTarget?: Pick<Window, "addEventListener" | "removeEventListener">
  }>
): ThemeController {
  const { storage, root, query, eventTarget } = options
  let preference = readThemePreference(storage)
  const listeners = new Set<ThemeListener>()

  const state = (): ThemeState => ({
    preference,
    theme: resolveTheme(preference, query.matches),
  })
  const notify = (): void => {
    const current = state()
    for (const listener of listeners) listener(current)
  }
  const onSystemChange = (): void => {
    if (preference === "system") notify()
  }
  const onStorage = (event: Event): void => {
    if ((event as StorageEvent).key !== themeStorageKey) return
    preference = readThemePreference(storage)
    applyThemePreference(root, preference)
    notify()
  }

  applyThemePreference(root, preference)
  query.addEventListener("change", onSystemChange)
  eventTarget?.addEventListener("storage", onStorage)

  return {
    get: state,
    set(next) {
      preference = parseThemePreference(next)
      writeThemePreference(storage, preference)
      applyThemePreference(root, preference)
      notify()
    },
    subscribe(listener) {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    dispose() {
      listeners.clear()
      query.removeEventListener("change", onSystemChange)
      eventTarget?.removeEventListener("storage", onStorage)
    },
  }
}

// <head> の先頭に置く。モジュールを読み込む前に、保存したテーマを <html> に当てる。
export const themeBootScript = `(function(){try{var t=localStorage.getItem(${JSON.stringify(themeStorageKey)});if(t==="light"||t==="dark")document.documentElement.dataset.theme=t}catch(e){}})()`
