import { renderWorkspaceDiagnosticCards } from "../diagnostics/diagnostic-cards"
import type {
  BindingInspection,
  InspectedBinding,
} from "../workspace/binding-inspection"
import { currentBindingInspection } from "../workspace/binding-inspection"
import type { WorkspaceState } from "../workspace/model"
import { describeSemanticNode } from "../workspace/semantic-inspection"

export type InteropView = Readonly<{
  id: string
  label: string
  render: (
    container: HTMLElement,
    inspection: BindingInspection,
    binding: InspectedBinding | undefined
  ) => void
}>

/** Extra views (such as semantic IR) share selection, revision and focus handling. */
export function connectInteropInspector(options: {
  button: HTMLButtonElement
  dialog: HTMLDialogElement
  navigate: (path: string, range: { start: number; end: number }) => void
  views?: readonly InteropView[]
}) {
  const { button, dialog } = options
  const doc = dialog.ownerDocument
  let inspection: BindingInspection | undefined
  let selectedEntry = ""
  let selectedView = "bindings"
  let selectedSource = ""
  let selectedType = ""
  let invalidated = false
  const element = <T extends keyof HTMLElementTagNameMap>(
    tag: T,
    text?: string
  ) => {
    const node = doc.createElement(tag)
    if (text !== undefined) node.textContent = text
    return node
  }
  const navigate = (path: string, range: { start: number; end: number }) => {
    dialog.close()
    options.navigate(path, range)
  }
  const codePanel = (title: string, path: string, source: string) => {
    const section = element("section")
    section.className = "interop-code-panel"
    section.append(element("h3", title), element("p", path))
    const pre = element("pre")
    pre.tabIndex = 0
    pre.setAttribute("aria-label", title)
    pre.append(element("code", source))
    section.append(pre)
    return section
  }
  const views: readonly InteropView[] = [
    {
      id: "bindings",
      label: "Bindings",
      render: (container, snapshot, binding) => {
        const preferred =
          binding?.declaration ??
          snapshot.diagnostics.find((d) => d.path.endsWith(".d.ts"))?.path ??
          snapshot.inputs.find((file) => file.path.endsWith(".d.ts"))?.path
        if (!snapshot.inputs.some((file) => file.path === selectedSource))
          selectedSource = preferred ?? snapshot.inputs[0]?.path ?? ""
        const label = element("label", "Source input ")
        const select = element("select")
        select.setAttribute("aria-label", "Source input")
        for (const file of snapshot.inputs) {
          const option = element("option", file.path)
          option.value = file.path
          select.append(option)
        }
        select.value = selectedSource
        select.addEventListener("change", () => {
          selectedSource = select.value
          render()
          content
            .querySelector<HTMLSelectElement>('[aria-label="Source input"]')
            ?.focus()
        })
        label.append(select)
        const open = element("button", "Open source in editor")
        open.type = "button"
        open.disabled = !selectedSource
        open.addEventListener("click", () =>
          navigate(selectedSource, { start: 0, end: 0 })
        )
        const controls = element("div")
        controls.className = "interop-source-controls"
        controls.append(label, open)
        const grid = element("div")
        grid.className = "interop-comparison"
        grid.append(
          codePanel(
            "Source · read only",
            selectedSource,
            snapshot.inputs.find((file) => file.path === selectedSource)
              ?.source ?? ""
          )
        )
        grid.append(
          codePanel(
            "Generated binding · read only",
            binding?.path ?? "No generated binding",
            binding?.source ??
              "No binding was generated. Check Diagnostics for conversion errors."
          )
        )
        container.append(controls, grid)
      },
    },
    {
      id: "report",
      label: "Report",
      render: (container, _snapshot, binding) => {
        if (!binding) {
          container.append(
            element(
              "p",
              "No conversion report is available. Check Diagnostics."
            )
          )
          return
        }
        container.append(element("h3", `Conversion report · ${binding.id}`))
        for (const key of [
          "added",
          "changed",
          "removed",
          "unsupported",
        ] as const) {
          const values = binding.report[key]
          const section = element("section")
          section.className = "interop-report-group"
          section.append(
            element(
              "h4",
              `${key[0]?.toUpperCase()}${key.slice(1)} (${values.length})`
            )
          )
          if (values.length) {
            const list = element("ul")
            for (const value of values) list.append(element("li", value))
            section.append(list)
          } else section.append(element("p", "None"))
          container.append(section)
        }
      },
    },
    {
      id: "semantic",
      label: "Semantic IR",
      render: (container, snapshot) => {
        const semantic = snapshot.semantics.find(
          (entry) => entry.id === selectedEntry
        )
        if (!semantic) {
          container.append(
            element("p", "No semantic graph is available for this conversion.")
          )
          return
        }
        container.append(element("h3", `Semantic graph · ${semantic.id}`))
        const select = element("select")
        select.setAttribute("aria-label", "Semantic type")
        for (const node of semantic.nodes) {
          const option = element(
            "option",
            `${node.id} · ${String(node.node.kind)}`
          )
          option.value = node.id
          select.append(option)
        }
        if (!semantic.nodes.some((node) => node.id === selectedType))
          selectedType = semantic.nodes[0]?.id ?? ""
        select.value = selectedType
        select.addEventListener("change", () => {
          selectedType = select.value
          render()
          content
            .querySelector<HTMLSelectElement>('[aria-label="Semantic type"]')
            ?.focus()
        })
        const label = element("label", "Type relation ")
        label.append(select)
        container.append(label)
        const node = semantic.nodes.find((node) => node.id === selectedType)
        if (!node) return
        const description = describeSemanticNode(node.node)
        container.append(element("p", description.meaning))
        for (const relation of description.relations) {
          const link = element(
            "button",
            `${relation.label} → type:${relation.target}`
          )
          link.type = "button"
          link.addEventListener("click", () => {
            selectedType = relation.target
            render()
            content
              .querySelector<HTMLSelectElement>('[aria-label="Semantic type"]')
              ?.focus()
          })
          container.append(link)
        }
        container.append(
          codePanel(
            "Type meaning · read only",
            `type:${node.id}`,
            JSON.stringify(node.node, null, 2)
          )
        )
        for (const decision of node.decisions)
          container.append(
            element(
              "p",
              `${decision.classification} · ${decision.reason} (${decision.rule})`
            )
          )
        container.append(element("h4", "Source provenance"))
        for (const origin of node.origins) {
          const open = element(
            "button",
            `${origin.source}:${origin.start}–${origin.end} · ${origin.rule}`
          )
          open.type = "button"
          open.addEventListener("click", () => navigate(origin.source, origin))
          container.append(open)
          const related = snapshot.diagnostics.filter(
            (diagnostic) =>
              diagnostic.path === origin.source &&
              diagnostic.diagnostic.primary.start <= origin.end &&
              diagnostic.diagnostic.primary.end >= origin.start
          )
          if (related.length) {
            const diagnostics = element("div")
            renderWorkspaceDiagnosticCards(diagnostics, related, navigate)
            container.append(diagnostics)
          }
        }
        if (!node.origins.length)
          container.append(
            element("p", "No source span is attached to this type.")
          )
        container.append(
          codePanel(
            "Symbols and artifact identity · read only",
            "schema 1",
            JSON.stringify(
              { symbols: semantic.symbols, identity: semantic.identity },
              null,
              2
            )
          )
        )
      },
    },
    {
      id: "diagnostics",
      label: "Diagnostics",
      render: (container, snapshot) => {
        if (!snapshot.diagnostics.length)
          container.append(element("p", "No conversion diagnostics."))
        else
          renderWorkspaceDiagnosticCards(
            container,
            snapshot.diagnostics,
            navigate
          )
      },
    },
    ...(options.views ?? []),
  ]
  if (new Set(views.map((view) => view.id)).size !== views.length)
    throw new Error("Duplicate Interop view identity")
  const header = element("header")
  header.className = "sample-browser-dialog-header"
  const heading = element("div")
  const title = element("h2", "Interop Inspector")
  title.id = "interop-inspector-title"
  heading.append(
    title,
    element(
      "p",
      "Compare declaration inputs with generated bindings. Generated code is read-only."
    )
  )
  const close = element("button", "×")
  close.type = "button"
  close.className = "sample-browser-close"
  close.setAttribute("aria-label", "Close Interop Inspector")
  close.addEventListener("click", () => dialog.close())
  header.append(heading, close)
  const content = element("div")
  content.className = "interop-content"
  dialog.replaceChildren(header, content)
  function render() {
    content.replaceChildren()
    if (!inspection) {
      content.append(
        element(
          "p",
          invalidated
            ? "Inputs changed. Convert bindings again to inspect the current result."
            : "Convert bindings to compare declaration inputs, generated code, reports and diagnostics."
        )
      )
      return
    }
    const snapshot = inspection
    const entryIds = [
      ...new Set([
        ...snapshot.bindings.map((binding) => binding.id),
        ...snapshot.semantics.map((entry) => entry.id),
      ]),
    ]
    if (!entryIds.includes(selectedEntry)) selectedEntry = entryIds[0] ?? ""
    const binding = snapshot.bindings.find(
      (binding) => binding.id === selectedEntry
    )
    const status = element(
      "p",
      snapshot.status === "success"
        ? "Current conversion result"
        : "Conversion contains errors · generated code is not published"
    )
    status.className = "interop-result-status"
    status.setAttribute("role", "status")
    content.append(status)
    if (entryIds.length) {
      const label = element("label", "Binding entry ")
      const select = element("select")
      select.setAttribute("aria-label", "Binding entry")
      for (const id of entryIds) {
        const option = element("option", id)
        option.value = id
        select.append(option)
      }
      select.value = selectedEntry
      select.addEventListener("change", () => {
        selectedEntry = select.value
        selectedSource = ""
        selectedType = ""
        render()
        content
          .querySelector<HTMLSelectElement>('[aria-label="Binding entry"]')
          ?.focus()
      })
      label.append(select)
      content.append(label)
    }
    const tabs = element("div")
    tabs.className = "interop-tabs"
    tabs.setAttribute("role", "tablist")
    tabs.setAttribute("aria-label", "Interop views")
    const panel = element("section")
    panel.className = "interop-view"
    panel.id = "interop-view-panel"
    panel.setAttribute("role", "tabpanel")
    const active = views.find((view) => view.id === selectedView) ?? views[0]
    for (const [index, view] of views.entries()) {
      const tab = element(
        "button",
        view.id === "diagnostics"
          ? `${view.label} (${snapshot.diagnostics.length})`
          : view.label
      )
      tab.type = "button"
      tab.id = `interop-tab-${view.id}`
      tab.setAttribute("role", "tab")
      tab.setAttribute("aria-selected", String(view === active))
      tab.setAttribute("aria-controls", panel.id)
      tab.tabIndex = view === active ? 0 : -1
      tab.addEventListener("click", () => {
        selectedView = view.id
        render()
        doc.getElementById(tab.id)?.focus()
      })
      tab.addEventListener("keydown", (event) => {
        const target =
          event.key === "ArrowRight"
            ? (index + 1) % views.length
            : event.key === "ArrowLeft"
              ? (index + views.length - 1) % views.length
              : event.key === "Home"
                ? 0
                : event.key === "End"
                  ? views.length - 1
                  : undefined
        if (target === undefined) return
        event.preventDefault()
        selectedView = views[target]?.id ?? "bindings"
        render()
        doc.getElementById(`interop-tab-${selectedView}`)?.focus()
      })
      tabs.append(tab)
    }
    if (active) {
      panel.setAttribute("aria-labelledby", `interop-tab-${active.id}`)
      active.render(panel, snapshot, binding)
    }
    content.append(tabs, panel)
  }
  button.addEventListener("click", () => {
    render()
    dialog.showModal()
    button.setAttribute("aria-expanded", "true")
  })
  dialog.addEventListener("close", () =>
    button.setAttribute("aria-expanded", "false")
  )
  render()
  return {
    set(next: BindingInspection | undefined) {
      inspection = next
      invalidated = false
      selectedSource = ""
      render()
    },
    sync(state: WorkspaceState) {
      const current = currentBindingInspection(inspection, state)
      if (inspection !== undefined && current === undefined) {
        inspection = undefined
        invalidated = true
        render()
      }
    },
  }
}
