// API の説明のカード。Search（⌘K）で API の結果を選んだときに、下に出す。
import type { AnalysisReferenceItem } from "../compiler/types"
import { highlightSeseragi } from "../editor/seseragi-language"

export function createReferenceCard(
  ownerDocument: Document,
  item: AnalysisReferenceItem
): HTMLElement {
  const article = ownerDocument.createElement("article")
  article.className = "reference-card"
  const header = ownerDocument.createElement("header")
  const identity = ownerDocument.createElement("div")
  const name = ownerDocument.createElement("strong")
  name.textContent = item.name
  const meta = ownerDocument.createElement("span")
  meta.textContent = `${item.category} · ${item.kind}`
  identity.append(name, meta)
  const module = ownerDocument.createElement("code")
  module.textContent = item.module
  header.append(identity, module)
  article.append(header)
  if (item.signature !== undefined) {
    const signature = ownerDocument.createElement("pre")
    signature.className = "seseragi-highlight"
    signature.append(
      ...highlightSeseragi(item.signature).map(({ text, classes }) => {
        const part = ownerDocument.createElement("span")
        part.className = classes
        part.textContent = text
        return part
      })
    )
    article.append(signature)
  }
  const description = ownerDocument.createElement("p")
  description.textContent = item.description
  article.append(description)
  if (item.constraints.length > 0 && !item.signature?.includes(" where ")) {
    const constraints = ownerDocument.createElement("small")
    constraints.textContent = `where ${item.constraints.join(", ")}`
    article.append(constraints)
  }
  return article
}
