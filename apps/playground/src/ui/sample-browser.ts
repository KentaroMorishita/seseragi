import type { PlaygroundSample } from "../samples"
import { sampleLevels } from "../sample-catalog"

type SampleBrowserElements = {
  readonly button: HTMLButtonElement
  readonly dialog: HTMLDialogElement
  readonly closeButton: HTMLButtonElement
  readonly groups: HTMLElement
  readonly currentLevel: HTMLElement
  readonly currentTitle: HTMLElement
}

export function connectSampleBrowser(
  elements: SampleBrowserElements,
  samples: readonly PlaygroundSample[],
  onSelect: (sample: PlaygroundSample) => void
): {
  readonly setCurrent: (sample: PlaygroundSample) => void
} {
  const ownerDocument = elements.dialog.ownerDocument
  const cards = new Map<string, HTMLButtonElement>()

  for (const level of sampleLevels) {
    const levelSamples = samples.filter((sample) => sample.level === level)
    if (levelSamples.length === 0) continue

    const section = ownerDocument.createElement("section")
    section.className = "sample-level"
    const heading = ownerDocument.createElement("div")
    heading.className = "sample-level-heading"

    const title = ownerDocument.createElement("h3")
    title.textContent = level
    const description = ownerDocument.createElement("p")
    description.textContent = levelDescription(level)
    heading.append(title, description)

    const list = ownerDocument.createElement("div")
    list.className = "sample-card-grid"
    for (const sample of levelSamples) {
      const card = ownerDocument.createElement("button")
      card.type = "button"
      card.className = "sample-card"
      card.dataset.sampleId = sample.id
      card.setAttribute(
        "aria-label",
        `${sequenceLabel(sample)} ${sample.label}`
      )

      const sequence = ownerDocument.createElement("span")
      sequence.className = "sample-card-sequence"
      sequence.textContent = sequenceLabel(sample)
      const name = ownerDocument.createElement("strong")
      name.textContent = sample.label
      const summary = ownerDocument.createElement("span")
      summary.className = "sample-card-summary"
      summary.textContent = sample.summary
      const concepts = ownerDocument.createElement("span")
      concepts.className = "sample-card-concepts"
      concepts.textContent = sample.concepts.join(" · ")

      card.append(sequence, name, summary, concepts)
      card.addEventListener("click", () => {
        onSelect(sample)
        elements.dialog.close()
      })
      cards.set(sample.id, card)
      list.append(card)
    }

    section.append(heading, list)
    elements.groups.append(section)
  }

  const setExpanded = (expanded: boolean): void => {
    elements.button.setAttribute("aria-expanded", String(expanded))
  }

  elements.button.addEventListener("click", () => {
    elements.dialog.showModal()
    setExpanded(true)
    const current = elements.groups.querySelector<HTMLButtonElement>(
      '.sample-card[aria-current="true"]'
    )
    current?.focus()
  })
  elements.closeButton.addEventListener("click", () => elements.dialog.close())
  elements.dialog.addEventListener("click", (event) => {
    if (event.target === elements.dialog) elements.dialog.close()
  })
  elements.dialog.addEventListener("close", () => {
    setExpanded(false)
    elements.button.focus()
  })

  return {
    setCurrent: (sample) => {
      elements.currentLevel.textContent = `${sample.level} ${sequenceLabel(sample)}`
      elements.currentTitle.textContent = sample.label
      for (const [id, card] of cards) {
        if (id === sample.id) card.setAttribute("aria-current", "true")
        else card.removeAttribute("aria-current")
      }
    },
  }
}

function sequenceLabel(sample: PlaygroundSample): string {
  return String(sample.sequence).padStart(2, "0")
}

function levelDescription(level: (typeof sampleLevels)[number]): string {
  switch (level) {
    case "初級":
      return "値と関数から、実行できる小さなprogramを作る"
    case "中級":
      return "型とEffectで、domainの境界を設計する"
    case "上級":
      return "抽象化とinstanceを使い、振る舞いを合成する"
    case "実践":
      return "状態、HTML、DOMを一つのappへ統合する"
  }
}
