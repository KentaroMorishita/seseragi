// 上のバーのテーマの切り替え。
// - 押すと、今見えているテーマの反対（ライト ↔ ダーク）に切り替え、その選択を保存する。
//   OS の設定に合わせている（System）ときも、押せば必ず見た目が変わる。
// - ラベルは押したときの動作を表す（"Switch to dark theme"）。
// - System に戻す操作は More メニューに置き、System 以外を選んでいるときだけ出す。
import type { ThemeController, ThemeState } from "./theme"

export function themeToggleTarget(state: ThemeState): "light" | "dark" {
  return state.theme === "dark" ? "light" : "dark"
}

export function themeButtonLabel(state: ThemeState): string {
  return `Switch to ${themeToggleTarget(state)} theme`
}

export function connectThemeButton(
  button: HTMLButtonElement,
  theme: ThemeController,
  systemButton?: HTMLButtonElement
): () => void {
  const render = (): void => {
    const state = theme.get()
    const label = themeButtonLabel(state)
    button.setAttribute("aria-label", label)
    button.title = label
    button.dataset.theme = state.theme
    if (systemButton) systemButton.hidden = state.preference === "system"
  }
  const onClick = (): void => theme.set(themeToggleTarget(theme.get()))
  const onSystem = (): void => theme.set("system")
  button.addEventListener("click", onClick)
  systemButton?.addEventListener("click", onSystem)
  const unsubscribe = theme.subscribe(render)
  render()
  return () => {
    button.removeEventListener("click", onClick)
    systemButton?.removeEventListener("click", onSystem)
    unsubscribe()
  }
}

// localStorage は、プライベートな閲覧などで触っただけで例外になることがある。
export function browserStorage(): Pick<
  Storage,
  "getItem" | "setItem" | "removeItem"
> {
  try {
    const storage = window.localStorage
    storage.getItem("")
    return storage
  } catch {
    const values = new Map<string, string>()
    return {
      getItem: (key) => values.get(key) ?? null,
      setItem: (key, value) => void values.set(key, value),
      removeItem: (key) => void values.delete(key),
    }
  }
}
