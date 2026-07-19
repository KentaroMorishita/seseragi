export type SampleGuideValue = {
  readonly label: string
  readonly level: string
  readonly summary: string
  readonly concepts: readonly string[]
  readonly sourcePath: string
}

type SampleGuideElements = {
  readonly button: HTMLButtonElement
  readonly panel: HTMLElement
  readonly closeButton: HTMLButtonElement
  readonly category: HTMLElement
  readonly title: HTMLElement
  readonly summary: HTMLElement
  readonly concepts: HTMLUListElement
  readonly source: HTMLElement
}

export function connectSampleGuide(elements: SampleGuideElements): {
  readonly setSample: (sample: SampleGuideValue) => void
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
      elements.category.textContent = sample.level
      elements.title.textContent = sample.label
      elements.summary.textContent = sample.summary
      elements.source.textContent = sample.sourcePath
      elements.concepts.replaceChildren(
        ...sample.concepts.map((concept) => {
          const item = ownerDocument.createElement("li")
          item.textContent = concept
          return item
        })
      )
      setOpen(false)
    },
  }
}
