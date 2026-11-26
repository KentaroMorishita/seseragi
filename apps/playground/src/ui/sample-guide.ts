import type { PlaygroundSample } from "../samples"
import { renderGuideMarkdown } from "./guide-markdown"
import { type UiLocale, UNTRANSLATED_GUIDE_NOTICE, uiMessages } from "./i18n"

type SampleGuideElements = {
  readonly button: HTMLButtonElement
  readonly panel: HTMLElement
  readonly closeButton: HTMLButtonElement
  readonly category: HTMLElement
  readonly title: HTMLElement
  readonly summary: HTMLElement
  readonly untranslatedNotice?: HTMLElement | null
  readonly topics: HTMLUListElement
  readonly body: HTMLElement
  readonly source: HTMLElement
  // 見出しの上に出す、例のグループ名（「ブラウザの画面」など）
  readonly groupTitle: (sample: PlaygroundSample) => string
  readonly locale?: () => UiLocale
}

export function connectSampleGuide(elements: SampleGuideElements): {
  readonly setSample: (sample: PlaygroundSample) => void
  readonly setBlank: () => void
  readonly refreshLocale: () => void
} {
  const ownerDocument = elements.panel.ownerDocument
  let currentSample: PlaygroundSample | undefined
  const readLocale = (): UiLocale => elements.locale?.() ?? "ja"

  const applyLocaleState = (): void => {
    const locale = readLocale()
    elements.button.title = uiMessages(locale).guideButtonTitle
    const isUntranslated = locale === "en" && currentSample !== undefined
    if (elements.untranslatedNotice) {
      elements.untranslatedNotice.hidden = !isUntranslated
      elements.untranslatedNotice.textContent = isUntranslated
        ? UNTRANSLATED_GUIDE_NOTICE
        : ""
    }
    const contentTargets = [
      elements.category,
      elements.title,
      elements.summary,
      elements.body,
    ]
    for (const target of contentTargets) {
      if (isUntranslated) target.setAttribute("lang", "ja")
      else target.removeAttribute("lang")
    }
  }

  const setOpen = (open: boolean): void => {
    elements.panel.hidden = !open
    elements.button.setAttribute("aria-expanded", String(open))
  }

  elements.button.addEventListener("click", () => {
    setOpen(elements.panel.hidden)
  })
  elements.closeButton.addEventListener("click", () => {
    setOpen(false)
    elements.button.focus()
  })
  ownerDocument.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || elements.panel.hidden) return
    setOpen(false)
    elements.button.focus()
  })
  ownerDocument.addEventListener("pointerdown", (event) => {
    if (elements.panel.hidden || !(event.target instanceof Node)) return
    if (
      elements.panel.contains(event.target) ||
      elements.button.contains(event.target)
    ) {
      return
    }
    setOpen(false)
  })

  return {
    setSample: (sample) => {
      currentSample = sample
      elements.button.hidden = false
      // 内部の分類名（Recipe など）は出さず、例のグループ名を出す
      elements.category.textContent = elements.groupTitle(sample)
      elements.title.textContent = sample.title
      elements.summary.textContent = sample.summary
      renderGuideMarkdown(elements.body, sample.guide)
      elements.source.textContent = sample.sourcePath
      // 英語のトピック名は出さない。例のデータに「使っている考え方」（Tour の
      // レッスンへのリンク）が入ったら、ここに並べる（#1350 の全体設計 3）。
      elements.topics.replaceChildren()
      applyLocaleState()
      setOpen(false)
    },
    setBlank: () => {
      currentSample = undefined
      setOpen(false)
      elements.button.hidden = true
      elements.category.textContent = ""
      elements.title.textContent = ""
      elements.summary.textContent = ""
      elements.topics.replaceChildren()
      elements.body.replaceChildren()
      elements.source.textContent = ""
      applyLocaleState()
    },
    refreshLocale: applyLocaleState,
  }
}
