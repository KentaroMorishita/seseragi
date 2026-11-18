import { describe, expect, test } from "bun:test"
import { createRequire } from "node:module"
import { fontPackages } from "../src/ui/design/font-packages"
import {
  applyThemePreference,
  createThemeController,
  parseThemePreference,
  readThemePreference,
  resolveTheme,
  themeBootScript,
  themeStorageKey,
  writeThemePreference,
} from "../src/ui/theme"

const designRoot = new URL("../src/ui/design/", import.meta.url)
const read = (name: string): Promise<string> =>
  Bun.file(new URL(name, designRoot)).text()

const tokensCss = await read("tokens.css")
const componentsCss = await read("components.css")
const fontsCss = await read("fonts.css")
const indexCss = await read("index.css")

function block(css: string, opener: string): string {
  const start = css.indexOf(opener)
  if (start < 0) throw new Error(`missing block: ${opener}`)
  let depth = 0
  for (let index = css.indexOf("{", start); index < css.length; index += 1) {
    if (css[index] === "{") depth += 1
    if (css[index] === "}") depth -= 1
    if (depth === 0) return css.slice(css.indexOf("{", start) + 1, index)
  }
  throw new Error(`unclosed block: ${opener}`)
}

function declarations(css: string): Map<string, string> {
  const found = new Map<string, string>()
  for (const match of css.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/gu))
    found.set(match[1]!, match[2]!.replace(/\s+/gu, " ").trim())
  return found
}

const light = declarations(block(tokensCss, ":root {"))
const darkFromSystem = declarations(
  block(block(tokensCss, "@media (prefers-color-scheme: dark)"), ":root")
)
const darkFromChoice = declarations(
  block(tokensCss, ':root[data-theme="dark"]')
)

const themes = [
  ["light", light],
  ["dark", darkFromChoice],
] as const

function luminance(hex: string): number {
  const value = /^#([0-9a-f]{6})$/iu.exec(hex)?.[1]
  if (!value) throw new Error(`not an opaque hex color: ${hex}`)
  const [red, green, blue] = [0, 2, 4].map((offset) => {
    const channel = Number.parseInt(value.slice(offset, offset + 2), 16) / 255
    return channel <= 0.03928
      ? channel / 12.92
      : ((channel + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * red! + 0.7152 * green! + 0.0722 * blue!
}

function contrast(foreground: string, background: string): number {
  const [lighter, darker] = [luminance(foreground), luminance(background)].sort(
    (left, right) => right - left
  )
  return (lighter! + 0.05) / (darker! + 0.05)
}

describe("design tokens", () => {
  test("the dark values are the same whether they come from the OS or from a saved choice", () => {
    expect([...darkFromChoice]).toEqual([...darkFromSystem])
  })

  test("every color, shadow and syntax token has a dark value", () => {
    const themed = [...light].filter(([, value]) =>
      /#[0-9a-f]{3,8}\b|rgba?\(/iu.test(value)
    )
    expect(themed.length).toBeGreaterThan(30)
    for (const [name] of themed) expect(darkFromChoice.has(name)).toBe(true)
    for (const name of darkFromChoice.keys()) expect(light.has(name)).toBe(true)
    expect(light.get("--ui-color-scheme")).toBe("light")
    expect(darkFromChoice.get("--ui-color-scheme")).toBe("dark")
  })

  test("text keeps a 4.5:1 contrast on every surface in both themes", () => {
    const text = [
      "--fg-1",
      "--fg-2",
      "--fg-3",
      "--accent-ink",
      "--ok",
      "--warn",
      "--error",
    ]
    const surfaces = ["--canvas", "--surface", "--surface-2"]
    for (const [theme, tokens] of themes) {
      for (const foreground of text)
        for (const background of surfaces) {
          const ratio = contrast(
            tokens.get(foreground)!,
            tokens.get(background)!
          )
          if (ratio < 4.5)
            throw new Error(
              `${theme}: ${foreground} on ${background} is ${ratio.toFixed(2)}:1`
            )
        }
    }
  })

  test("text on filled parts keeps a 4.5:1 contrast", () => {
    const pairs = [
      ["--on-primary", "--primary"],
      ["--on-primary-accent", "--primary"],
      ["--surface", "--error"],
    ] as const
    for (const [theme, tokens] of themes)
      for (const [foreground, background] of pairs) {
        const ratio = contrast(tokens.get(foreground)!, tokens.get(background)!)
        if (ratio < 4.5)
          throw new Error(
            `${theme}: ${foreground} on ${background} is ${ratio.toFixed(2)}:1`
          )
      }
  })

  test("narrow screens only change sizes", () => {
    const narrow = declarations(
      block(block(tokensCss, "@media (max-width: 760px)"), ":root")
    )
    expect([...narrow.keys()].sort()).toEqual([
      "--code-line",
      "--code-size",
      "--text-display",
      "--top-h",
    ])
  })
})

describe("shared components", () => {
  test("only use tokens for colors and shadows", () => {
    const withoutComments = componentsCss.replace(/\/\*[\s\S]*?\*\//gu, "")
    expect(withoutComments).not.toMatch(/#[0-9a-f]{3,8}\b/iu)
    expect(withoutComments).not.toMatch(/\b(?:rgba?|hsla?|oklch)\(/iu)
  })

  test("reference only tokens that exist", () => {
    const used = new Set(
      [...componentsCss.matchAll(/var\((--[a-z0-9-]+)/gu)].map(
        (match) => match[1]!
      )
    )
    expect(used.size).toBeGreaterThan(20)
    for (const name of used)
      if (!light.has(name)) throw new Error(`unknown token ${name}`)
  })

  test("are namespaced so they cannot restyle the current screens", () => {
    const withoutComments = componentsCss.replace(/\/\*[\s\S]*?\*\//gu, "")
    const selectors = [
      ...withoutComments.matchAll(/(?:^|\})\s*([^{}@]+?)\s*\{/gu),
    ].flatMap((match) => match[1]!.split(","))
    expect(selectors.length).toBeGreaterThan(40)
    for (const selector of selectors) {
      const trimmed = selector.trim()
      if (!/^\.ui-[a-z]/u.test(trimmed))
        throw new Error(`selector outside the ui- namespace: ${trimmed}`)
    }
  })

  test("stop motion when reduced motion is requested", () => {
    expect(componentsCss).toContain("@media (prefers-reduced-motion: reduce)")
  })

  test("load fonts, then tokens, then components", () => {
    expect(
      [...indexCss.matchAll(/@import "([^"]+)"/gu)].map((match) => match[1])
    ).toEqual(["./fonts.css", "./tokens.css", "./components.css"])
  })

  test("are loaded by every page", async () => {
    for (const [entry, path] of [
      ["../src/main.ts", "./ui/design/index.css"],
      ["../src/tour/main.ts", "../ui/design/index.css"],
      ["../src/deep-dive/main.ts", "../ui/design/index.css"],
    ] as const) {
      const source = await Bun.file(new URL(entry, import.meta.url)).text()
      expect(source).toContain(`import "${path}"`)
    }
  })
})

describe("self-hosted fonts", () => {
  const require = createRequire(import.meta.url)

  test("come from the bundled packages, never from a font service", () => {
    expect(
      [...fontsCss.matchAll(/@import "([^"]+)"/gu)].map((match) => match[1])
    ).toEqual(fontPackages.map((font) => font.packageName))
    for (const css of [fontsCss, tokensCss, componentsCss])
      expect(css).not.toMatch(/https?:\/\//u)
  })

  test("are the families the font tokens ask for", async () => {
    for (const font of fontPackages) {
      const css = await Bun.file(require.resolve(font.packageName)).text()
      expect(css).toContain(`font-family: '${font.family}'`)
      expect(css).toContain("unicode-range")
      expect(css).not.toMatch(/url\(https?:/u)
    }
    expect(light.get("--font-ui")).toStartWith(
      '"Geist Variable", "Noto Sans JP Variable"'
    )
    expect(light.get("--font-code")).toStartWith('"Geist Mono Variable"')
  })

  test("ship with the SIL Open Font License", async () => {
    for (const font of fontPackages) {
      const license = await Bun.file(
        require.resolve(`${font.packageName}/LICENSE`)
      ).text()
      expect(license).toContain("SIL Open Font License, Version 1.1")
      expect(font.licenseFile).toMatch(/^fonts\/LICENSE-[A-Za-z]+\.txt$/u)
    }
  })
})

function memoryStorage(initial: Record<string, string> = {}) {
  const values = new Map(Object.entries(initial))
  return {
    values,
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => void values.set(key, value),
    removeItem: (key: string) => void values.delete(key),
  }
}

function colorSchemeQuery(matches: boolean) {
  const listeners = new Set<() => void>()
  return {
    matches,
    listeners,
    addEventListener: (_type: string, listener: () => void) =>
      void listeners.add(listener),
    removeEventListener: (_type: string, listener: () => void) =>
      void listeners.delete(listener),
    change(next: boolean) {
      this.matches = next
      for (const listener of listeners) listener()
    },
  }
}

describe("theme", () => {
  test("follows the OS unless a theme was chosen", () => {
    expect(parseThemePreference(null)).toBe("system")
    expect(parseThemePreference("sepia")).toBe("system")
    expect(parseThemePreference("dark")).toBe("dark")
    expect(resolveTheme("system", true)).toBe("dark")
    expect(resolveTheme("system", false)).toBe("light")
    expect(resolveTheme("light", true)).toBe("light")
    expect(resolveTheme("dark", false)).toBe("dark")
  })

  test("stores a chosen theme and forgets it when going back to System", () => {
    const storage = memoryStorage()
    writeThemePreference(storage, "dark")
    expect(storage.values.get(themeStorageKey)).toBe("dark")
    expect(readThemePreference(storage)).toBe("dark")
    writeThemePreference(storage, "system")
    expect(storage.values.has(themeStorageKey)).toBe(false)
  })

  test("keeps working when storage is unavailable", () => {
    const broken = {
      getItem: () => {
        throw new Error("blocked")
      },
      setItem: () => {
        throw new Error("blocked")
      },
      removeItem: () => {
        throw new Error("blocked")
      },
    }
    expect(readThemePreference(broken)).toBe("system")
    expect(() => writeThemePreference(broken, "dark")).not.toThrow()
  })

  test("puts a chosen theme on <html> and removes it for System", () => {
    const root = { dataset: {} as DOMStringMap }
    applyThemePreference(root, "light")
    expect(root.dataset.theme).toBe("light")
    applyThemePreference(root, "system")
    expect("theme" in root.dataset).toBe(false)
  })

  test("the controller applies, saves and reports the theme", () => {
    const storage = memoryStorage({ [themeStorageKey]: "light" })
    const root = { dataset: {} as DOMStringMap }
    const query = colorSchemeQuery(true)
    const theme = createThemeController({ storage, root, query })
    expect(root.dataset.theme).toBe("light")
    expect(theme.get()).toEqual({ preference: "light", theme: "light" })

    const seen: string[] = []
    theme.subscribe((state) => seen.push(`${state.preference}:${state.theme}`))
    theme.set("system")
    expect("theme" in root.dataset).toBe(false)
    expect(storage.values.has(themeStorageKey)).toBe(false)
    query.change(false)
    theme.set("dark")
    query.change(false)
    expect(seen).toEqual(["system:dark", "system:light", "dark:dark"])

    theme.dispose()
    expect(query.listeners.size).toBe(0)
  })

  test("the boot script applies the saved theme before the page loads", () => {
    for (const [saved, expected] of [
      ["dark", "dark"],
      ["light", "light"],
      ["sepia", undefined],
      [null, undefined],
    ] as const) {
      const documentElement = { dataset: {} as DOMStringMap }
      const localStorage = {
        getItem: (key: string) => (key === themeStorageKey ? saved : null),
      }
      new Function("localStorage", "document", themeBootScript)(localStorage, {
        documentElement,
      })
      expect(documentElement.dataset.theme).toBe(expected)
    }
    expect(() =>
      new Function("localStorage", "document", themeBootScript)(
        {
          getItem: () => {
            throw new Error("blocked")
          },
        },
        { documentElement: { dataset: {} } }
      )
    ).not.toThrow()
  })
})
