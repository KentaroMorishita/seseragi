import type {
  DiscoverGroupDefinition,
  SampleArchitecture,
  SampleCapability,
  SampleExperience,
  SampleFocus,
  SampleKind,
} from "../sample-catalog"
import type { PlaygroundSample } from "../samples"

type SampleBrowserElements = {
  readonly button: HTMLButtonElement
  readonly dialog: HTMLDialogElement
  readonly closeButton: HTMLButtonElement
  readonly search: HTMLInputElement
  readonly kindFilter: HTMLSelectElement
  readonly topicFilter: HTMLSelectElement
  readonly capabilityFilter: HTMLSelectElement
  readonly featuredFilter: HTMLInputElement
  readonly newFilter: HTMLInputElement
  readonly resultCount: HTMLElement
  readonly results: HTMLElement
  readonly currentTitle: HTMLElement
}

export function connectSampleBrowser(
  elements: SampleBrowserElements,
  samples: readonly PlaygroundSample[],
  groups: readonly DiscoverGroupDefinition[],
  onSelect: (sample: PlaygroundSample) => boolean | undefined
): { readonly setCurrent: (sample: PlaygroundSample) => void } {
  const ownerDocument = elements.dialog.ownerDocument
  const byId = new Map(samples.map((sample) => [sample.id, sample]))
  const discoverSamples = samples.filter(({ kind }) => kind !== "lesson")
  let currentSample = samples[0]

  const topics = [
    ...new Set(discoverSamples.flatMap((sample) => sample.topics)),
  ].sort((left, right) => left.localeCompare(right))
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
    const filtered = discoverSamples.filter((sample) => {
      const searchable = [
        sample.title,
        sample.summary,
        ...sample.topics,
        ...(sample.experience === undefined
          ? []
          : [
              experienceLabel(sample.experience),
              architectureLabel(sample.architecture),
              focusLabel(sample.focus),
            ]),
      ]
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
    const filteredIds = new Set(filtered.map(({ id }) => id))
    elements.results.replaceChildren(
      ...groups.flatMap((group) => {
        const groupSamples = group.samples
          .map((id) => byId.get(id))
          .filter(
            (sample): sample is PlaygroundSample =>
              sample !== undefined && filteredIds.has(sample.id)
          )
        if (groupSamples.length === 0) return []

        const section = ownerDocument.createElement("section")
        section.className = "sample-discover-group"
        section.dataset.groupId = group.id
        const heading = ownerDocument.createElement("div")
        heading.className = "sample-discover-heading"
        const label = ownerDocument.createElement("span")
        label.className = "sample-discover-kind"
        label.textContent = kindLabel(group.kind)
        const title = ownerDocument.createElement("h3")
        title.textContent = group.title
        const summary = ownerDocument.createElement("p")
        summary.textContent = group.summary
        heading.append(label, title, summary)
        const list = ownerDocument.createElement("div")
        list.className = "sample-card-grid"
        list.append(...groupSamples.map((sample) => createSampleCard(sample)))
        section.append(heading, list)
        return [section]
      })
    )
  }

  function createSampleCard(sample: PlaygroundSample): HTMLButtonElement {
    const card = ownerDocument.createElement("button")
    card.type = "button"
    card.className = "sample-card"
    card.dataset.sampleId = sample.id
    card.setAttribute("aria-label", sample.title)

    const meta = ownerDocument.createElement("span")
    meta.className = "sample-card-meta"
    meta.textContent = catalogLabel(sample)
    const name = ownerDocument.createElement("strong")
    name.textContent = sample.title
    const summary = ownerDocument.createElement("span")
    summary.className = "sample-card-summary"
    summary.textContent = sample.summary
    const topics = ownerDocument.createElement("span")
    topics.className = "sample-card-topics"
    topics.textContent = [
      ...(sample.focus === undefined ? [] : [focusLabel(sample.focus)]),
      ...sample.topics,
    ].join(" · ")
    const prerequisite = ownerDocument.createElement("span")
    prerequisite.className = "sample-card-prerequisite"
    const prerequisiteTitles = sample.prerequisites
      .map((id) => byId.get(id))
      .filter(
        (candidate): candidate is PlaygroundSample =>
          candidate?.experience !== undefined
      )
      .map(({ title }) => title)
    prerequisite.textContent =
      prerequisiteTitles.length === 0
        ? ""
        : `先に見る: ${prerequisiteTitles.join(" / ")}`
    const comparison = ownerDocument.createElement("span")
    comparison.className = "sample-card-comparison"
    const comparisonTitle =
      sample.comparisonSample === undefined
        ? undefined
        : byId.get(sample.comparisonSample)?.title
    comparison.textContent =
      comparisonTitle === undefined ? "" : `比較: ${comparisonTitle}`
    const badges = ownerDocument.createElement("span")
    badges.className = "sample-card-badges"
    if (sample.featured) badges.append(createBadge("FEATURED"))
    if (sample.isNew) badges.append(createBadge("NEW"))

    card.append(meta, name, summary, topics)
    if (comparison.textContent !== "") card.append(comparison)
    if (prerequisite.textContent !== "") card.append(prerequisite)
    card.append(badges)
    card.addEventListener("click", () => {
      if (onSelect(sample) === false) return
      setCurrentSample(sample)
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

  const setCurrentSample = (sample: PlaygroundSample): void => {
    currentSample = sample
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
    renderDiscover()
    elements.dialog.showModal()
    setExpanded(true)
    const currentCard = elements.dialog.querySelector<HTMLButtonElement>(
      '.sample-card[aria-current="true"]'
    )
    ;(currentCard ?? elements.search).focus()
  })
  elements.closeButton.addEventListener("click", () => elements.dialog.close())
  elements.dialog.addEventListener("click", (event) => {
    if (event.target === elements.dialog) elements.dialog.close()
  })
  elements.dialog.addEventListener("close", () => {
    setExpanded(false)
    elements.button.focus()
  })
  renderDiscover()

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

export function experienceLabel(value: SampleExperience): string {
  return { minimal: "Minimal", guided: "Guided", showcase: "Showcase" }[value]
}

export function architectureLabel(
  value: SampleArchitecture | undefined
): string {
  if (value === undefined) return ""
  return {
    static: "Static HTML",
    "dom-app": "dom.app",
    "signal-run": "Signal + dom.run",
    "signal-mount": "Signal + mount",
    "multi-module": "Multi-module",
  }[value]
}

export function focusLabel(value: SampleFocus | undefined): string {
  if (value === undefined) return ""
  return {
    component: "Component",
    state: "State",
    form: "Form",
    event: "Event",
    composition: "Composition",
    project: "Project",
  }[value]
}

function catalogLabel(sample: PlaygroundSample): string {
  if (sample.experience === undefined) {
    return `${difficultyLabel(sample.difficulty)} · ${kindLabel(sample.kind)}`
  }
  return `${experienceLabel(sample.experience)} · ${architectureLabel(sample.architecture)} · ${difficultyLabel(sample.difficulty)}`
}
