import type { InteropView } from "./interop-inspector"

/** Counts, evidence and source jumps use the same payload emitted by the CLI. */
export function createCompatibilityView(
  navigate: (path: string, range: { start: number; end: number }) => void
): InteropView {
  return {
    id: "compatibility",
    label: "Compatibility",
    render(container, inspection, binding) {
      const doc = container.ownerDocument
      const element = <T extends keyof HTMLElementTagNameMap>(
        tag: T,
        text?: string
      ) => {
        const node = doc.createElement(tag)
        if (text !== undefined) node.textContent = text
        return node
      }
      const reports = (inspection.compatibility ?? []).filter(
        (entry) => !binding || entry.id === binding.id
      )
      if (!reports.length) {
        container.append(
          element(
            "p",
            "No semantic compatibility report is available. Convert declarations to inspect their meaning and projection evidence."
          )
        )
        return
      }
      for (const report of reports) {
        container.append(element("h3", `Semantic compatibility · ${report.id}`))
        container.append(
          element(
            "p",
            "Public symbols are counted once. Runtime ABI uncertainty is shown separately from type interpretation."
          )
        )
        for (const count of report.counts) {
          const details = element("details")
          const summary = element(
            "summary",
            `${count.classification}: ${count.count}`
          )
          details.append(summary)
          for (const identity of count.symbols) {
            const symbol = report.symbols[identity]
            if (!symbol) continue
            const section = element("section")
            section.append(element("h4", symbol.publicName))
            section.append(
              element(
                "p",
                `Type ${symbol.typeRef} · meaning: ${symbol.semanticClassification} · projection: ${symbol.projectionClassification ?? "no projection evidence"}`
              )
            )
            if (symbol.runtimeAbiUnresolved)
              section.append(element("p", "Runtime ABI unresolved"))
            const reasons = element("ul")
            for (const reason of symbol.reasons) {
              reasons.append(
                element(
                  "li",
                  `${reason.classification} · ${reason.rule}: ${reason.reason}${reason.policy ? ` · policy ${reason.policy.name}@${reason.policy.version}` : ""}`
                )
              )
            }
            section.append(reasons)
            for (const origin of symbol.origins) {
              const button = element(
                "button",
                `${origin.source}:${origin.start}–${origin.end} · ${origin.rule}`
              )
              button.type = "button"
              button.addEventListener("click", () =>
                navigate(origin.source, {
                  start: origin.start,
                  end: origin.end,
                })
              )
              section.append(button)
            }
            details.append(section)
          }
          container.append(details)
        }
      }
    },
  }
}
