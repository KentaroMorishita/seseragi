// 開発サーバーの /ui-kit.html（部品の見本）だけが読み込む。ビルドには含めない。
import "./index.css"
import { createThemeController, type ThemePreference } from "../theme"

const swatchTokens = [
  "canvas",
  "surface",
  "surface-2",
  "line-strong",
  "fg-1",
  "fg-2",
  "fg-3",
  "primary",
  "accent",
  "accent-ink",
  "ok",
  "warn",
  "error",
]

const swatches = document.querySelector("#swatches")
for (const token of swatchTokens) {
  const item = document.createElement("div")
  item.className = "kit-swatch"
  const color = document.createElement("span")
  color.style.background = `var(--${token})`
  item.append(color, `--${token}`)
  swatches?.append(item)
}

const theme = createThemeController({
  storage: localStorage,
  root: document.documentElement,
  query: matchMedia("(prefers-color-scheme: dark)"),
  eventTarget: window,
})
const options = document.querySelectorAll<HTMLButtonElement>(
  "[data-theme-option]"
)
const render = (): void => {
  const { preference } = theme.get()
  for (const option of options)
    option.setAttribute(
      "aria-pressed",
      String(option.dataset.themeOption === preference)
    )
}
for (const option of options)
  option.addEventListener("click", () =>
    theme.set(option.dataset.themeOption as ThemePreference)
  )
theme.subscribe(render)
render()
