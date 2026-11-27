import { describe, expect, test } from "bun:test"
import {
  dismissEnglishBanner,
  ENGLISH_BANNER_DISMISSED_KEY,
  isJapaneseLanguage,
  JAPANESE_ONLY_LABEL,
  localeFromPathname,
  localizedPath,
  playgroundHref,
  shouldShowEnglishBanner,
  tourHref,
  UNTRANSLATED_GUIDE_NOTICE,
  UNTRANSLATED_LESSON_NOTICE,
  uiMessages,
} from "../src/ui/i18n"

function memoryStorage(): Storage {
  const data = new Map<string, string>()
  return {
    get length() {
      return data.size
    },
    clear: () => data.clear(),
    getItem: (key) => data.get(key) ?? null,
    key: (index) => Array.from(data.keys())[index] ?? null,
    removeItem: (key) => {
      data.delete(key)
    },
    setItem: (key, value) => {
      data.set(key, value)
    },
  }
}

describe("Playground and Tour i18n (#1360)", () => {
  test("detects locale from URL pathname", () => {
    expect(localeFromPathname("/")).toBe("ja")
    expect(localeFromPathname("/tour/")).toBe("ja")
    expect(localeFromPathname("/en")).toBe("en")
    expect(localeFromPathname("/en/")).toBe("en")
    expect(localeFromPathname("/en/tour/")).toBe("en")
  })

  test("converts paths between ja and en without losing subpaths", () => {
    expect(localizedPath("/", "en")).toBe("/en/")
    expect(localizedPath("/en/", "ja")).toBe("/")
    expect(localizedPath("/en", "ja")).toBe("/")
    expect(localizedPath("/tour/", "en")).toBe("/en/tour/")
    expect(localizedPath("/en/tour/", "ja")).toBe("/tour/")
    expect(playgroundHref("ja")).toBe("/")
    expect(playgroundHref("en")).toBe("/en/")
    expect(tourHref("ja")).toBe("/tour/")
    expect(tourHref("en")).toBe("/en/tour/")
  })

  test("shows the English banner once for non-Japanese browsers on Japanese pages", () => {
    const storage = memoryStorage()
    expect(isJapaneseLanguage(["ja", "en-US"])).toBe(true)
    expect(isJapaneseLanguage(["ja-JP"])).toBe(true)
    expect(isJapaneseLanguage(["en-US", "ja"])).toBe(false)

    expect(shouldShowEnglishBanner("ja", ["ja-JP"], storage)).toBe(false)
    expect(shouldShowEnglishBanner("en", ["en-US"], storage)).toBe(false)
    expect(shouldShowEnglishBanner("ja", ["en-US"], storage)).toBe(true)

    dismissEnglishBanner(storage)
    expect(storage.getItem(ENGLISH_BANNER_DISMISSED_KEY)).toBe("1")
    expect(shouldShowEnglishBanner("ja", ["en-US"], storage)).toBe(false)
  })

  test("provides complete English UI messages and exact untranslated notices", () => {
    const ja = uiMessages("ja")
    const en = uiMessages("en")
    expect(Object.keys(en).sort()).toEqual(Object.keys(ja).sort())
    expect(en.autoRunNote).not.toBe(ja.autoRunNote)
    expect(en.outputPlaceholder).not.toBe(ja.outputPlaceholder)
    expect(en.networkConsentTitle).not.toBe(ja.networkConsentTitle)
    expect(en.tourExpectedSummary).not.toBe(ja.tourExpectedSummary)
    expect(JAPANESE_ONLY_LABEL).toBe("Japanese only")
    expect(UNTRANSLATED_LESSON_NOTICE).toBe(
      "This lesson isn't translated yet. Its text is shown in the original Japanese. Buttons, code, and output work the same."
    )
    expect(UNTRANSLATED_GUIDE_NOTICE).toContain(
      "Its text is shown in the original Japanese."
    )
  })
})
