import type { AnalysisReferenceItem } from "../compiler/types"

type ReferenceBrowserElements = {
  readonly button: HTMLButtonElement
  readonly dialog: HTMLDialogElement
  readonly closeButton: HTMLButtonElement
  readonly search: HTMLInputElement
  readonly category: HTMLSelectElement
  readonly count: HTMLElement
  readonly results: HTMLElement
}

export function connectReferenceBrowser(elements: ReferenceBrowserElements): {
  readonly setCatalog: (items: readonly AnalysisReferenceItem[]) => void
} {
  let catalog: readonly AnalysisReferenceItem[] = []

  const render = (): void => {
    const query = elements.search.value.trim().toLocaleLowerCase()
    const category = elements.category.value
    const filtered = catalog.filter((item) => {
      const searchable = [
        item.name,
        item.module,
        item.signature ?? "",
        item.description,
        ...item.constraints,
      ]
        .join(" ")
        .toLocaleLowerCase()
      return (
        (!category || item.category === category) &&
        (!query || searchable.includes(query))
      )
    })
    elements.count.textContent = `${filtered.length} symbols`
    elements.results.replaceChildren(...filtered.map(createReferenceCard))
  }

  const createReferenceCard = (item: AnalysisReferenceItem): HTMLElement => {
    const article = elements.dialog.ownerDocument.createElement("article")
    article.className = "reference-card"
    const header = elements.dialog.ownerDocument.createElement("header")
    const identity = elements.dialog.ownerDocument.createElement("div")
    const name = elements.dialog.ownerDocument.createElement("strong")
    name.textContent = item.name
    const meta = elements.dialog.ownerDocument.createElement("span")
    meta.textContent = `${item.category} · ${item.kind}`
    identity.append(name, meta)
    const module = elements.dialog.ownerDocument.createElement("code")
    module.textContent = item.module
    header.append(identity, module)
    article.append(header)
    if (item.signature !== undefined) {
      const signature = elements.dialog.ownerDocument.createElement("pre")
      signature.textContent = item.signature
      article.append(signature)
    }
    const description = elements.dialog.ownerDocument.createElement("p")
    description.textContent = item.description
    article.append(description)
    if (item.constraints.length > 0) {
      const constraints = elements.dialog.ownerDocument.createElement("small")
      constraints.textContent = `where ${item.constraints.join(", ")}`
      article.append(constraints)
    }
    return article
  }

  const setCatalog = (items: readonly AnalysisReferenceItem[]): void => {
    catalog = items
    const selected = elements.category.value
    const categories = [...new Set(items.map((item) => item.category))].sort(
      (left, right) => left.localeCompare(right)
    )
    elements.category.replaceChildren(
      option("", "すべて"),
      ...categories.map((category) => option(category, category))
    )
    if (categories.includes(selected)) elements.category.value = selected
    render()
  }

  const option = (value: string, label: string): HTMLOptionElement => {
    const option = elements.dialog.ownerDocument.createElement("option")
    option.value = value
    option.textContent = label
    return option
  }

  elements.search.addEventListener("input", render)
  elements.category.addEventListener("change", render)
  elements.button.addEventListener("click", () => {
    elements.dialog.showModal()
    elements.button.setAttribute("aria-expanded", "true")
    elements.search.focus()
  })
  elements.closeButton.addEventListener("click", () => elements.dialog.close())
  elements.dialog.addEventListener("click", (event) => {
    if (event.target === elements.dialog) elements.dialog.close()
  })
  elements.dialog.addEventListener("close", () => {
    elements.button.setAttribute("aria-expanded", "false")
    elements.button.focus()
  })

  return { setCatalog }
}
