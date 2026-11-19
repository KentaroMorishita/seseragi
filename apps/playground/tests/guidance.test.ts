import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"
import { applyTryNext, createGuidancePreferences } from "../src/ui/guidance"

function memoryStorage(): Pick<Storage, "getItem" | "setItem" | "removeItem"> {
  const values = new Map<string, string>()
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => void values.set(key, value),
    removeItem: (key) => void values.delete(key),
  }
}

const guided = samples.filter(({ guidance }) => guidance !== undefined)

describe("example guidance (#1350 design 2)", () => {
  test("opens the Playground on the weather example with a hint and auto-run", () => {
    const weather = samples.find(({ id }) => id === "weather-advice")
    expect(weather?.guidance?.autoRun).toBe(true)
    expect(weather?.guidance?.hint?.anchor).toBe("Rainy 3")
    expect(weather?.guidance?.tryNext.length).toBeGreaterThanOrEqual(2)
    // 10 行前後で type・match・|> が一度に見える
    expect(weather?.source).toContain("type Weather")
    expect(weather?.source).toContain("match weather")
    expect(weather?.source).toContain("|>")
    expect(weather?.source.trim().split("\n").length).toBeLessThanOrEqual(14)
  })

  test("points every hint at text that is in the example", () => {
    for (const sample of guided) {
      const anchor = sample.guidance?.hint?.anchor
      if (anchor !== undefined)
        expect(sample.source, sample.id).toContain(anchor)
    }
  })

  test("applies every Try next chip alone and after the others", () => {
    for (const sample of guided) {
      const chips = sample.guidance?.tryNext ?? []
      for (const chip of chips) {
        expect(applyTryNext(sample.source, chip), chip.label).toBeDefined()
      }
      let all: string | undefined = sample.source
      for (const chip of chips) {
        all = all === undefined ? undefined : applyTryNext(all, chip)
      }
      expect(all, `${sample.id}: all chips in order`).toBeDefined()
    }
  })

  test("refuses a chip whose text is no longer in the code", () => {
    expect(
      applyTryNext("let a = 1\n", {
        label: "x",
        edits: [{ find: "let b", replace: "let c" }],
      })
    ).toBeUndefined()
  })

  test("remembers the coach, Hide, and turning Hints off and on", () => {
    const storage = memoryStorage()
    const preferences = createGuidancePreferences(storage)
    expect(preferences.hintsEnabled()).toBe(true)
    expect(preferences.coachDone()).toBe(false)
    preferences.markCoachDone()
    preferences.setTryNextCollapsed(true)
    const reopened = createGuidancePreferences(storage)
    expect(reopened.coachDone()).toBe(true)
    expect(reopened.tryNextCollapsed()).toBe(true)

    reopened.setHintsEnabled(false)
    expect(createGuidancePreferences(storage).hintsEnabled()).toBe(false)
    // オンに戻すと、ひとことをもう一度出せる
    reopened.setHintsEnabled(true)
    expect(createGuidancePreferences(storage).coachDone()).toBe(false)
  })
})
