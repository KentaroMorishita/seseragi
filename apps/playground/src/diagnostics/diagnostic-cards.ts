import type { Diagnostic, SourceRange } from "../compiler/types"
import {
  describeSourceLocation,
  formatSourceLocation,
  utf8RangeToSourceLocation,
} from "./source-range"

export function renderDiagnosticCards(
  container: HTMLElement,
  diagnostics: readonly Diagnostic[],
  source: string,
  onNavigate: (range: SourceRange) => void
): void {
  const document = container.ownerDocument
  container.className = "diagnostic-list"
  container.replaceChildren(
    ...diagnostics.map((diagnostic) => {
      const card = document.createElement("article")
      card.className = "diagnostic-card"

      const location = document.createElement("button")
      location.type = "button"
      location.className = "diagnostic-card-location"
      const sourceLocation = utf8RangeToSourceLocation(
        source,
        diagnostic.primary
      )
      const locationLabel = formatSourceLocation("main.ssrg", sourceLocation)
      location.title = `Go to ${describeSourceLocation(sourceLocation)}`
      location.setAttribute(
        "aria-label",
        `${diagnostic.message}. ${describeSourceLocation(sourceLocation)}`
      )
      location.dataset.byteStart = String(diagnostic.primary.start)
      location.dataset.byteEnd = String(diagnostic.primary.end)
      const code = document.createElement("span")
      code.className = "diagnostic-card-code"
      code.textContent = diagnostic.code
      const title = document.createElement("strong")
      title.textContent = diagnostic.message
      const range = document.createElement("span")
      range.className = "diagnostic-card-range"
      range.textContent = locationLabel
      location.append(code, title, range)
      location.addEventListener("click", () => onNavigate(diagnostic.primary))
      card.append(location)

      if (diagnostic.expectedType || diagnostic.actualType) {
        const types = document.createElement("dl")
        types.className = "diagnostic-card-types"
        if (diagnostic.expectedType) {
          types.append(
            term(document, "Expected"),
            detail(document, diagnostic.expectedType)
          )
        }
        if (diagnostic.actualType) {
          types.append(
            term(document, "Actual"),
            detail(document, diagnostic.actualType)
          )
        }
        card.append(types)
      }

      const labels =
        diagnostic.labels.length > 0 ? diagnostic.labels : diagnostic.related
      if (labels.length > 0) {
        const list = document.createElement("ul")
        list.className = "diagnostic-card-labels"
        for (const label of labels) {
          const item = document.createElement("li")
          item.textContent = label.message
          list.append(item)
        }
        card.append(list)
      }

      for (const help of diagnostic.helps) {
        const paragraph = document.createElement("p")
        paragraph.className = "diagnostic-card-help"
        paragraph.textContent = `Help: ${help}`
        card.append(paragraph)
      }
      for (const fix of diagnostic.fixes) {
        const paragraph = document.createElement("p")
        paragraph.className = "diagnostic-card-fix"
        paragraph.textContent = `Fix: ${fix.title}`
        card.append(paragraph)
      }
      for (const note of diagnostic.notes) {
        const paragraph = document.createElement("p")
        paragraph.className = "diagnostic-card-note"
        paragraph.textContent = `Note: ${note}`
        card.append(paragraph)
      }
      return card
    })
  )
}

function term(document: Document, value: string): HTMLElement {
  const element = document.createElement("dt")
  element.textContent = value
  return element
}

function detail(document: Document, value: string): HTMLElement {
  const element = document.createElement("dd")
  element.textContent = value
  return element
}
