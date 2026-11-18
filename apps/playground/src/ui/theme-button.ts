// 上のバーのテーマの切り替え。押すたびに System → Light → Dark → System と進む。
import type { ThemeController, ThemePreference } from "./theme"

const order: readonly ThemePreference[] = ["system", "light", "dark"]
const names: Readonly<Record<ThemePreference, string>> = {
  system: "System",
  light: "Light",
  dark: "Dark",
}

export function nextThemePreference(
  preference: ThemePreference
): ThemePreference {
  return order[(order.indexOf(preference) + 1) % order.length]!
}

export function themeButtonLabel(preference: ThemePreference): string {
  return `Theme: ${names[preference]}`
}

export function connectThemeButton(
  button: HTMLButtonElement,
  theme: ThemeController
): () => void {
  const render = (): void => {
    const label = themeButtonLabel(theme.get().preference)
    button.setAttribute("aria-label", label)
    button.title = label
    button.dataset.themePreference = theme.get().preference
  }
  const onClick = (): void =>
    theme.set(nextThemePreference(theme.get().preference))
  button.addEventListener("click", onClick)
  const unsubscribe = theme.subscribe(render)
  render()
  return () => {
    button.removeEventListener("click", onClick)
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
