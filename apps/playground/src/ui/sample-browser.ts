import type {
  LearningPathDefinition,
  SampleCapability,
  SampleKind,
} from "../sample-catalog"
import type { PlaygroundSample } from "../samples"

type SampleBrowserElements = {
  readonly button: HTMLButtonElement
  readonly dialog: HTMLDialogElement
  readonly closeButton: HTMLButtonElement
  readonly learnTab: HTMLButtonElement
  readonly discoverTab: HTMLButtonElement
  readonly learnPanel: HTMLElement
  readonly discoverPanel: HTMLElement
  readonly learningPaths: HTMLElement
  readonly search: HTMLInputElement
  readonly kindFilter: HTMLSelectElement
  readonly topicFilter: HTMLSelectElement
  readonly capabilityFilter: HTMLSelectElement
  readonly featuredFilter: HTMLInputElement
  readonly newFilter: HTMLInputElement
  readonly resultCount: HTMLElement
  readonly results: HTMLElement
  readonly currentContext: HTMLElement
  readonly currentTitle: HTMLElement
}

export function connectSampleBrowser(
  elements: SampleBrowserElements,
  samples: readonly PlaygroundSample[],
  paths: readonly LearningPathDefinition[],
  onSelect: (sample: PlaygroundSample) => void
): { readonly setCurrent: (sample: PlaygroundSample) => void } {
  const ownerDocument = elements.dialog.ownerDocument
  const byId = new Map(samples.map((sample) => [sample.id, sample]))
  let currentSample = samples[0]

  for (const path of paths) {
    const section = ownerDocument.createElement("section")
    section.className = "sample-path"
    const heading = ownerDocument.createElement("div")
    heading.className = "sample-path-heading"
    const title = ownerDocument.createElement("h3")
    title.textContent = path.title
    const summary = ownerDocument.createElement("p")
    summary.textContent = path.summary
    heading.append(title, summary)
    const list = ownerDocument.createElement("div")
    list.className = "sample-card-grid"
    path.samples.forEach((sampleId, index) => {
      const sample = byId.get(sampleId)
      if (!sample) return
      const prerequisites = sample.prerequisites
        .map((id) => byId.get(id)?.title ?? id)
        .join("、")
      const nextId = path.samples[index + 1]
      const next = nextId
        ? (byId.get(nextId)?.title ?? nextId)
        : "このpathは完了"
      list.append(
        createSampleCard(
          sample,
          `${index + 1} / ${path.samples.length}`,
          `前提: ${prerequisites || "なし"} · 次: ${next}`,
          `${path.title} · ${index + 1}/${path.samples.length}`
        )
      )
    })
    section.append(heading, list)
    elements.learningPaths.append(section)
  }

  const topics = [...new Set(samples.flatMap((sample) => sample.topics))].sort(
    (left, right) => left.localeCompare(right)
  )
  for (const topic of topics) {
    const option = ownerDocument.createElement("option")
    option.value = topic
    option.textContent = topic
    elements.topicFilter.append(option)
  }

  const renderDiscover = (): void => {
    const query = elements.search.value.trim().toLocaleLowerCase()
    const kind = elements.kindFilter.value as SampleKind | ""
    const topic = elements.topicFilter.value
    const capability = elements.capabilityFilter.value as SampleCapability | ""
    const filtered = samples.filter((sample) => {
      const searchable = [sample.title, sample.summary, ...sample.topics]
        .join(" ")
        .toLocaleLowerCase()
      return (
        (!query || searchable.includes(query)) &&
        (!kind || sample.kind === kind) &&
        (!topic || sample.topics.includes(topic)) &&
        (!capability || sample.capabilities.includes(capability)) &&
        (!elements.featuredFilter.checked || sample.featured) &&
        (!elements.newFilter.checked || sample.isNew)
      )
    })
    elements.resultCount.textContent = `${filtered.length} samples`
    elements.results.replaceChildren(
      ...filtered.map((sample) => createSampleCard(sample))
    )
  }

  function createSampleCard(
    sample: PlaygroundSample,
    pathProgress?: string,
    route?: string,
    context = "Discover"
  ): HTMLButtonElement {
    const card = ownerDocument.createElement("button")
    card.type = "button"
    card.className = "sample-card"
    card.dataset.sampleId = sample.id
    card.setAttribute("aria-label", sample.title)

    const meta = ownerDocument.createElement("span")
    meta.className = "sample-card-meta"
    meta.textContent = [
      pathProgress,
      difficultyLabel(sample.difficulty),
      kindLabel(sample.kind),
    ]
      .filter(Boolean)
      .join(" · ")
    const name = ownerDocument.createElement("strong")
    name.textContent = sample.title
    const summary = ownerDocument.createElement("span")
    summary.className = "sample-card-summary"
    summary.textContent = sample.summary
    const topics = ownerDocument.createElement("span")
    topics.className = "sample-card-topics"
    topics.textContent = sample.topics.join(" · ")
    const routeText = ownerDocument.createElement("span")
    routeText.className = "sample-card-route"
    routeText.textContent = route ?? ""
    const badges = ownerDocument.createElement("span")
    badges.className = "sample-card-badges"
    if (sample.featured) badges.append(createBadge("FEATURED"))
    if (sample.isNew) badges.append(createBadge("NEW"))

    card.append(meta, name, summary, topics)
    if (route) card.append(routeText)
    card.append(badges)
    card.addEventListener("click", () => {
      onSelect(sample)
      setCurrentSample(sample, context)
      elements.dialog.close()
    })
    if (currentSample?.id === sample.id)
      card.setAttribute("aria-current", "true")
    return card
  }

  function createBadge(label: string): HTMLElement {
    const badge = ownerDocument.createElement("span")
    badge.className = "sample-card-badge"
    badge.textContent = label
    return badge
  }

  const setCurrentSample = (
    sample: PlaygroundSample,
    context?: string
  ): void => {
    currentSample = sample
    const firstPath = paths.find((path) => path.samples.includes(sample.id))
    const index = firstPath?.samples.indexOf(sample.id) ?? -1
    const defaultContext = firstPath
      ? `${firstPath.title} · ${index + 1}/${firstPath.samples.length}`
      : `${difficultyLabel(sample.difficulty)} · ${kindLabel(sample.kind)}`
    elements.currentContext.textContent = context ?? defaultContext
    elements.currentTitle.textContent = sample.title
    for (const card of elements.dialog.querySelectorAll<HTMLButtonElement>(
      ".sample-card"
    )) {
      if (card.dataset.sampleId === sample.id) {
        card.setAttribute("aria-current", "true")
      } else {
        card.removeAttribute("aria-current")
      }
    }
  }

  const setMode = (mode: "learn" | "discover"): void => {
    const learn = mode === "learn"
    elements.learnTab.setAttribute("aria-selected", String(learn))
    elements.discoverTab.setAttribute("aria-selected", String(!learn))
    elements.learnPanel.hidden = !learn
    elements.discoverPanel.hidden = learn
    if (!learn) renderDiscover()
  }
  elements.learnTab.addEventListener("click", () => setMode("learn"))
  elements.discoverTab.addEventListener("click", () => setMode("discover"))
  for (const control of [
    elements.search,
    elements.kindFilter,
    elements.topicFilter,
    elements.capabilityFilter,
    elements.featuredFilter,
    elements.newFilter,
  ]) {
    control.addEventListener("input", renderDiscover)
    control.addEventListener("change", renderDiscover)
  }

  const setExpanded = (expanded: boolean): void => {
    elements.button.setAttribute("aria-expanded", String(expanded))
  }
  elements.button.addEventListener("click", () => {
    elements.dialog.showModal()
    setExpanded(true)
    elements.dialog
      .querySelector<HTMLButtonElement>('.sample-card[aria-current="true"]')
      ?.focus()
  })
  elements.closeButton.addEventListener("click", () => elements.dialog.close())
  elements.dialog.addEventListener("click", (event) => {
    if (event.target === elements.dialog) elements.dialog.close()
  })
  elements.dialog.addEventListener("close", () => {
    setExpanded(false)
    elements.button.focus()
  })
  setMode("learn")

  return {
    setCurrent: setCurrentSample,
  }
}

function difficultyLabel(value: PlaygroundSample["difficulty"]): string {
  return { beginner: "初級", intermediate: "中級", advanced: "上級" }[value]
}

function kindLabel(value: PlaygroundSample["kind"]): string {
  return { lesson: "Lesson", recipe: "Recipe", showcase: "Showcase" }[value]
}
