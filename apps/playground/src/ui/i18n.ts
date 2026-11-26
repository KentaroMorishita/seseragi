// Playground と Tour の日本語 / 英語切り替え（#1360）。
// 言語は URL パス（/・/tour/ と /en/・/en/tour/）で表し、保存した好みによる自動リダイレクトは行わない。

export type UiLocale = "ja" | "en"

export const ENGLISH_BANNER_DISMISSED_KEY =
  "seseragi.i18n.english-banner-dismissed"

export const UNTRANSLATED_LESSON_NOTICE =
  "This lesson isn't translated yet. Its text is shown in the original Japanese. Buttons, code, and output work the same."

export const UNTRANSLATED_GUIDE_NOTICE =
  "This guide isn't translated yet. Its text is shown in the original Japanese. Buttons, code, and output work the same."

export const JAPANESE_ONLY_LABEL = "Japanese only"

/**
 * 初回ペイント前に <html lang> を URL パスに合わせる小さな起動スクリプト。
 * vite.config.ts が <head> 先頭へ埋め込む。
 */
export const localeBootScript =
  '(function(){try{var p=location.pathname;document.documentElement.lang=p==="/en"||p.indexOf("/en/")===0?"en":"ja"}catch(e){}})();'

export function localeFromPathname(pathname: string): UiLocale {
  return pathname === "/en" || pathname.startsWith("/en/") ? "en" : "ja"
}

export function localizedPath(pathname: string, target: UiLocale): string {
  const withoutLocale =
    pathname === "/en"
      ? "/"
      : pathname.startsWith("/en/")
        ? pathname.slice(3) || "/"
        : pathname || "/"
  const normalized = withoutLocale.startsWith("/")
    ? withoutLocale
    : `/${withoutLocale}`
  if (target === "ja") return normalized
  return normalized === "/" ? "/en/" : `/en${normalized}`
}

export function playgroundHref(locale: UiLocale): string {
  return locale === "en" ? "/en/" : "/"
}

export function tourHref(locale: UiLocale): string {
  return locale === "en" ? "/en/tour/" : "/tour/"
}

export function isJapaneseLanguage(languages: readonly string[]): boolean {
  const primary = languages.find((entry) => entry.trim().length > 0)
  if (primary === undefined) return true
  const normalized = primary.trim().toLowerCase()
  return normalized === "ja" || normalized.startsWith("ja-")
}

type BannerStorage = Pick<Storage, "getItem" | "setItem">

export function shouldShowEnglishBanner(
  locale: UiLocale,
  languages: readonly string[],
  storage: BannerStorage | null
): boolean {
  if (locale !== "ja") return false
  if (isJapaneseLanguage(languages)) return false
  try {
    return storage?.getItem(ENGLISH_BANNER_DISMISSED_KEY) !== "1"
  } catch {
    return true
  }
}

export function dismissEnglishBanner(storage: BannerStorage | null): void {
  try {
    storage?.setItem(ENGLISH_BANNER_DISMISSED_KEY, "1")
  } catch {
    // ストレージが使えない環境でも、この画面を開いている間は閉じたままにする。
  }
}

export type UiMessages = Readonly<{
  autoRunNote: string
  outputPlaceholder: string
  noErrorsRunPrompt: string
  stdinPlaceholder: string
  guideButtonTitle: string
  coachTourLink: string
  tryNextTourLink: string
  tryNextApplied: (label: string) => string
  searchLoadingLessons: string
  searchEmpty: string
  networkConsentTitle: string
  networkConsentLead: (label: string) => string
  networkConsentBullets: readonly [string, string, string]
  networkBlockedNotice: string
  networkSettingsLead: string
  networkSettingsEmpty: string
  tourExpectedSummary: string
  tourGuideTitle: string
  tourFocusTitle: string
  tourIntroducedTitle: string
  tourDiagnosticTitle: string
  tourDiagnosticButton: string
  tourRecapTitle: string
  tourPrerequisiteTitle: string
  tourNextTitle: string
  tourDeepDiveTitle: string
  tourRunCheckOk: string
  tourAnswerOpened: string
  tourDiagnosticOpened: string
  tourOutputCloseLabel: string
  tourOutputOpenLabel: string
}>

const jaMessages: UiMessages = {
  autoRunNote: "開いたときに自動で実行しました",
  outputPlaceholder: "Run を押すと、結果がここに出ます。",
  noErrorsRunPrompt: "エラーはありません。Run を押すと実行します。",
  stdinPlaceholder: "プログラムが標準入力から読む内容",
  guideButtonTitle: "このサンプルの説明",
  coachTourLink: "Tour で基本から →",
  tryNextTourLink: "基本から順に学ぶ →",
  tryNextApplied: (label) => `${label}（入れました）`,
  searchLoadingLessons: "レッスンを読み込んでいます。",
  searchEmpty:
    "見つかりませんでした。言葉を変えるか、範囲を All に戻してください。",
  networkConsentTitle: "外部と通信してよいですか",
  networkConsentLead: (label) =>
    `「${label}」は、外部のサイトから画像を読み込んだり、外部と通信したりします。`,
  networkConsentBullets: [
    "入力した内容や、このブラウザに保存したデータが、外部へ送られることがあります。",
    "許可は、この例（または作業場所）についてだけ、このブラウザに覚えます。あとでコードを書き換えても、許可は残ります。",
    "Search の Settings にある Network access から、いつでも取り消せます。",
  ],
  networkBlockedNotice: "外部の画像と通信を止めています。",
  networkSettingsLead:
    "外部の画像と通信について答えた例と作業場所です。取り消すと、次に要るときにもう一度聞きます。",
  networkSettingsEmpty: "まだ、どの例にも答えていません。",
  tourExpectedSummary: "期待する出力",
  tourGuideTitle: "読み方",
  tourFocusTitle: "このレッスンで学ぶこと",
  tourIntroducedTitle: "新しく出てきたもの",
  tourDiagnosticTitle: "よくある間違い",
  tourDiagnosticButton: "間違いの例をエディターで開く",
  tourRecapTitle: "振り返り",
  tourPrerequisiteTitle: "前提のレッスン",
  tourNextTitle: "次につながること",
  tourDeepDiveTitle: "設計の背景を読む",
  tourRunCheckOk: "✓ 期待どおり",
  tourAnswerOpened:
    "答えのコードを開きました。Run を押すと、書き換えたあとの出力になります。",
  tourDiagnosticOpened:
    "間違いの例を開きました。Run を押して診断を確かめてください。",
  tourOutputCloseLabel: "Outputを閉じる",
  tourOutputOpenLabel: "Outputを開く",
}

const enMessages: UiMessages = {
  autoRunNote: "Ran automatically when opened.",
  outputPlaceholder: "Press Run to see the output here.",
  noErrorsRunPrompt: "No errors. Press Run to execute.",
  stdinPlaceholder: "Input read from standard input",
  guideButtonTitle: "About this example",
  coachTourLink: "Learn the basics in the Tour →",
  tryNextTourLink: "Learn step by step in the Tour →",
  tryNextApplied: (label) => `${label} (applied)`,
  searchLoadingLessons: "Loading lessons…",
  searchEmpty: "No matches. Try another query or switch the scope back to All.",
  networkConsentTitle: "Allow network access?",
  networkConsentLead: (label) =>
    `"${label}" loads external images or connects to external hosts.`,
  networkConsentBullets: [
    "Code you enter or data saved in this browser may be sent to external hosts.",
    "Permission is remembered in this browser only for this example or workspace, and stays active after you edit the code.",
    "You can revoke permission anytime from Network access in Search → Settings.",
  ],
  networkBlockedNotice: "External images and network requests are blocked.",
  networkSettingsLead:
    "Examples and workspaces where you answered the network prompt. Remove an entry to be asked again next time.",
  networkSettingsEmpty: "No saved network decisions yet.",
  tourExpectedSummary: "Expected output",
  tourGuideTitle: "Guide",
  tourFocusTitle: "What you will learn",
  tourIntroducedTitle: "Introduced here",
  tourDiagnosticTitle: "Common mistake",
  tourDiagnosticButton: "Open broken example in editor",
  tourRecapTitle: "Recap",
  tourPrerequisiteTitle: "Prerequisites",
  tourNextTitle: "What comes next",
  tourDeepDiveTitle: "Deep Dive",
  tourRunCheckOk: "✓ Matches expected",
  tourAnswerOpened:
    "Opened the answer code. Press Run to see the updated output.",
  tourDiagnosticOpened:
    "Opened the broken example. Press Run to inspect the diagnostic.",
  tourOutputCloseLabel: "Hide output",
  tourOutputOpenLabel: "Show output",
}

export function uiMessages(locale: UiLocale): UiMessages {
  return locale === "en" ? enMessages : jaMessages
}

export type LocaleController = Readonly<{
  current: () => UiLocale
  messages: () => UiMessages
  setLocale: (next: UiLocale) => void
  subscribe: (listener: (locale: UiLocale) => void) => () => void
}>

export function connectLocaleController(
  options: Readonly<{
    window: Window
    jaButton: HTMLButtonElement
    enButton: HTMLButtonElement
    banner: HTMLElement | null
    bannerSwitchButton: HTMLButtonElement | null
    bannerDismissButton: HTMLButtonElement | null
    playgroundLinks: readonly HTMLAnchorElement[]
    tourLinks: readonly HTMLAnchorElement[]
  }>
): LocaleController {
  const { window: win } = options
  const listeners = new Set<(locale: UiLocale) => void>()
  let locale: UiLocale = localeFromPathname(win.location.pathname)

  const readStorage = (): BannerStorage | null => {
    try {
      return win.localStorage
    } catch {
      return null
    }
  }

  const readNavigatorLanguages = (): readonly string[] => {
    const langs = win.navigator.languages
    if (langs && langs.length > 0) return langs
    return win.navigator.language ? [win.navigator.language] : []
  }

  const syncDom = (): void => {
    win.document.documentElement.lang = locale
    options.jaButton.setAttribute("aria-pressed", String(locale === "ja"))
    options.enButton.setAttribute("aria-pressed", String(locale === "en"))
    const pgHref = playgroundHref(locale)
    const tHref = tourHref(locale)
    for (const link of options.playgroundLinks)
      link.setAttribute("href", pgHref)
    for (const link of options.tourLinks) link.setAttribute("href", tHref)
    if (options.banner !== null) {
      options.banner.hidden = !shouldShowEnglishBanner(
        locale,
        readNavigatorLanguages(),
        readStorage()
      )
    }
  }

  const applyLocale = (next: UiLocale, pushHistory: boolean): void => {
    const changed = next !== locale
    locale = next
    if (pushHistory && changed) {
      const nextPath = localizedPath(win.location.pathname, next)
      const nextUrl = `${nextPath}${win.location.search}${win.location.hash}`
      win.history.pushState(null, "", nextUrl)
    }
    if (next === "en") {
      dismissEnglishBanner(readStorage())
    }
    syncDom()
    if (changed) {
      for (const listener of listeners) listener(locale)
    }
  }

  options.jaButton.addEventListener("click", () => applyLocale("ja", true))
  options.enButton.addEventListener("click", () => applyLocale("en", true))
  options.bannerSwitchButton?.addEventListener("click", () =>
    applyLocale("en", true)
  )
  options.bannerDismissButton?.addEventListener("click", () => {
    dismissEnglishBanner(readStorage())
    if (options.banner !== null) options.banner.hidden = true
  })
  win.addEventListener("popstate", () => {
    applyLocale(localeFromPathname(win.location.pathname), false)
  })

  syncDom()

  return {
    current: () => locale,
    messages: () => uiMessages(locale),
    setLocale: (next) => applyLocale(next, true),
    subscribe: (listener) => {
      listeners.add(listener)
      return () => {
        listeners.delete(listener)
      }
    },
  }
}
