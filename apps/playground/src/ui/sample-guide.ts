import type { PlaygroundSample } from "../samples"

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
}

export function connectSampleGuide(elements: SampleGuideElements): {
  readonly setSample: (sample: PlaygroundSample) => void
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
      elements.category.textContent = `${difficultyLabel(sample.difficulty)} · ${kindLabel(sample.kind)}`
      elements.title.textContent = sample.title
      elements.summary.textContent = sample.summary
      elements.body.textContent = sample.guide.trim()
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
  }
}

function difficultyLabel(value: PlaygroundSample["difficulty"]): string {
  return { beginner: "初級", intermediate: "中級", advanced: "上級" }[value]
}

function kindLabel(value: PlaygroundSample["kind"]): string {
  return { lesson: "Lesson", recipe: "Recipe", showcase: "Showcase" }[value]
}
