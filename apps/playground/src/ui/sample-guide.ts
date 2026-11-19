import type { PlaygroundSample } from "../samples"
import { renderGuideMarkdown } from "./guide-markdown"

type SampleGuideElements = {
  readonly button: HTMLButtonElement
  readonly panel: HTMLElement
  readonly closeButton: HTMLButtonElement
  readonly category: HTMLElement
  readonly title: HTMLElement
  readonly summary: HTMLElement
  readonly topics: HTMLUListElement
  readonly body: HTMLElement
  readonly source: HTMLElement
  // 見出しの上に出す、例のグループ名（「ブラウザの画面」など）
  readonly groupTitle: (sample: PlaygroundSample) => string
}

export function connectSampleGuide(elements: SampleGuideElements): {
  readonly setSample: (sample: PlaygroundSample) => void
  readonly setBlank: () => void
} {
  const ownerDocument = elements.panel.ownerDocument

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
      elements.button.hidden = false
      // 内部の分類名（Recipe など）は出さず、例のグループ名を出す
      elements.category.textContent = elements.groupTitle(sample)
      elements.title.textContent = sample.title
      elements.summary.textContent = sample.summary
      renderGuideMarkdown(elements.body, sample.guide)
      elements.source.textContent = sample.sourcePath
      elements.topics.replaceChildren(
        ...sample.topics.map((topic) => {
          const item = ownerDocument.createElement("li")
          item.textContent = topic
          return item
        })
      )
      setOpen(false)
    },
    setBlank: () => {
      setOpen(false)
      elements.button.hidden = true
      elements.category.textContent = ""
      elements.title.textContent = ""
      elements.summary.textContent = ""
      elements.topics.replaceChildren()
      elements.body.replaceChildren()
      elements.source.textContent = ""
    },
  }
}
