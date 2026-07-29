import {
  closeSearchPanel,
  findNext,
  findPrevious,
  getSearchQuery,
  openSearchPanel,
  replaceAll,
  replaceNext,
  SearchQuery,
  setSearchQuery,
} from "@codemirror/search"
import type { EditorState } from "@codemirror/state"
import {
  type Command,
  type EditorView,
  type KeyBinding,
  type Panel,
  runScopeHandlers,
  type ViewUpdate,
} from "@codemirror/view"

export type SearchMatchStatus = {
  readonly current: number
  readonly total: number
  readonly valid: boolean
}

export function searchMatchStatus(
  state: EditorState,
  query: SearchQuery = getSearchQuery(state)
): SearchMatchStatus {
  if (!query.valid) {
    return { current: 0, total: 0, valid: query.search.length === 0 }
  }

  const selection = state.selection.main
  let exactIndex = -1
  let followingIndex = -1
  let total = 0
  const cursor = query.getCursor(state)

  for (let match = cursor.next(); !match.done; match = cursor.next()) {
    const index = total
    total += 1
    if (
      match.value.from === selection.from &&
      match.value.to === selection.to
    ) {
      exactIndex = index
    }
    if (followingIndex === -1 && match.value.from >= selection.head) {
      followingIndex = index
    }
  }

  if (total === 0) return { current: 0, total, valid: true }
  const current = exactIndex >= 0 ? exactIndex : Math.max(0, followingIndex)
  return { current: current + 1, total, valid: true }
}

const panels = new WeakMap<EditorView, CompactSearchPanel>()

export const openReplaceSearchPanel: Command = (view) => {
  openSearchPanel(view)
  panels.get(view)?.setReplaceExpanded(true, true)
  return true
}

export const compactSearchPanelKeymap: readonly KeyBinding[] = [
  {
    key: "Mod-h",
    run: openReplaceSearchPanel,
    scope: "editor search-panel",
    preventDefault: true,
  },
]

function actionButton(
  ownerDocument: Document,
  label: string,
  symbol: string,
  action: () => void
): HTMLButtonElement {
  const button = ownerDocument.createElement("button")
  button.type = "button"
  button.className = "ssrg-search-action"
  button.setAttribute("aria-label", label)
  button.title = label
  button.textContent = symbol
  button.addEventListener("click", action)
  return button
}

function optionButton(
  ownerDocument: Document,
  label: string,
  symbol: string,
  action: () => void
): HTMLButtonElement {
  const button = actionButton(ownerDocument, label, symbol, action)
  button.className = "ssrg-search-option"
  button.setAttribute("aria-pressed", "false")
  return button
}

class CompactSearchPanel implements Panel {
  readonly dom: HTMLFormElement
  readonly top = true
  private query: SearchQuery
  private readonly expandButton: HTMLButtonElement
  private readonly searchField: HTMLInputElement
  private readonly replaceField: HTMLInputElement
  private readonly caseButton: HTMLButtonElement
  private readonly wordButton: HTMLButtonElement
  private readonly regexpButton: HTMLButtonElement
  private readonly count: HTMLOutputElement
  private readonly previousButton: HTMLButtonElement
  private readonly nextButton: HTMLButtonElement
  private readonly replaceButton: HTMLButtonElement
  private readonly replaceAllButton: HTMLButtonElement
  private readonly replaceRow: HTMLDivElement
  private replaceExpanded = false

  constructor(private readonly view: EditorView) {
    const ownerDocument = view.dom.ownerDocument
    this.query = getSearchQuery(view.state)
    this.dom = ownerDocument.createElement("form")
    this.dom.className = "cm-search ssrg-search-panel"
    this.dom.setAttribute("role", "search")
    this.dom.setAttribute("aria-label", "Find and replace")
    this.dom.addEventListener("submit", (event) => event.preventDefault())
    this.dom.addEventListener("keydown", (event) => this.keydown(event))

    const searchRow = ownerDocument.createElement("div")
    searchRow.className = "ssrg-search-row"

    this.expandButton = actionButton(ownerDocument, "Show replace", "›", () =>
      this.setReplaceExpanded(!this.replaceExpanded, true)
    )
    this.expandButton.classList.add("ssrg-search-expand")

    const field = ownerDocument.createElement("div")
    field.className = "ssrg-search-field"
    this.searchField = ownerDocument.createElement("input")
    this.searchField.className = "cm-textfield ssrg-search-input"
    this.searchField.name = "search"
    this.searchField.type = "text"
    this.searchField.placeholder = "Find"
    this.searchField.autocomplete = "off"
    this.searchField.spellcheck = false
    this.searchField.setAttribute("aria-label", "Find")
    this.searchField.setAttribute("main-field", "true")
    this.searchField.addEventListener("input", () => this.commit())

    const options = ownerDocument.createElement("div")
    options.className = "ssrg-search-options"
    this.caseButton = optionButton(ownerDocument, "Match case", "Aa", () =>
      this.commit({ caseSensitive: !this.query.caseSensitive })
    )
    this.wordButton = optionButton(
      ownerDocument,
      "Match whole word",
      "ab",
      () => this.commit({ wholeWord: !this.query.wholeWord })
    )
    this.regexpButton = optionButton(
      ownerDocument,
      "Use regular expression",
      ".*",
      () => this.commit({ regexp: !this.query.regexp })
    )
    options.append(this.caseButton, this.wordButton, this.regexpButton)
    field.append(this.searchField, options)

    this.count = ownerDocument.createElement("output")
    this.count.className = "ssrg-search-count"
    this.count.setAttribute("aria-live", "polite")

    this.previousButton = actionButton(
      ownerDocument,
      "Previous match",
      "↑",
      () => findPrevious(view)
    )
    this.nextButton = actionButton(ownerDocument, "Next match", "↓", () =>
      findNext(view)
    )
    const closeButton = actionButton(
      ownerDocument,
      "Close find and replace",
      "×",
      () => closeSearchPanel(view)
    )
    searchRow.append(
      this.expandButton,
      field,
      this.count,
      this.previousButton,
      this.nextButton,
      closeButton
    )

    this.replaceRow = ownerDocument.createElement("div")
    this.replaceRow.className = "ssrg-search-row ssrg-search-replace-row"
    this.replaceRow.hidden = true
    const replaceSpacer = ownerDocument.createElement("span")
    replaceSpacer.className = "ssrg-search-replace-spacer"
    this.replaceField = ownerDocument.createElement("input")
    this.replaceField.className = "cm-textfield ssrg-search-input"
    this.replaceField.name = "replace"
    this.replaceField.type = "text"
    this.replaceField.placeholder = "Replace"
    this.replaceField.autocomplete = "off"
    this.replaceField.spellcheck = false
    this.replaceField.setAttribute("aria-label", "Replace")
    this.replaceField.addEventListener("input", () => this.commit())
    this.replaceButton = actionButton(
      ownerDocument,
      "Replace current match",
      "Replace",
      () => replaceNext(view)
    )
    this.replaceButton.classList.add("ssrg-search-replace-action")
    this.replaceAllButton = actionButton(
      ownerDocument,
      "Replace all matches",
      "All",
      () => replaceAll(view)
    )
    this.replaceAllButton.classList.add("ssrg-search-replace-action")
    this.replaceRow.append(
      replaceSpacer,
      this.replaceField,
      this.replaceButton,
      this.replaceAllButton
    )
    this.dom.append(searchRow, this.replaceRow)
    this.setQuery(this.query)
    this.renderStatus()
  }

  mount(): void {
    this.searchField.focus()
    this.searchField.select()
  }

  update(update: ViewUpdate): void {
    const query = getSearchQuery(update.state)
    if (!query.eq(this.query)) this.setQuery(query)
    this.renderStatus(update.state)
  }

  destroy(): void {
    if (panels.get(this.view) === this) panels.delete(this.view)
  }

  setReplaceExpanded(expanded: boolean, focus: boolean): void {
    if (this.view.state.readOnly) return
    this.replaceExpanded = expanded
    this.replaceRow.hidden = !expanded
    this.expandButton.textContent = expanded ? "⌄" : "›"
    this.expandButton.setAttribute("aria-expanded", String(expanded))
    this.expandButton.setAttribute(
      "aria-label",
      expanded ? "Hide replace" : "Show replace"
    )
    this.expandButton.title = expanded ? "Hide replace" : "Show replace"
    if (expanded && focus) {
      this.replaceField.focus()
      this.replaceField.select()
    }
  }

  private commit(
    override: Partial<
      Pick<SearchQuery, "caseSensitive" | "regexp" | "wholeWord">
    > = {}
  ): void {
    const query = new SearchQuery({
      search: this.searchField.value,
      replace: this.replaceField.value,
      caseSensitive: override.caseSensitive ?? this.query.caseSensitive,
      literal: this.query.literal,
      regexp: override.regexp ?? this.query.regexp,
      wholeWord: override.wholeWord ?? this.query.wholeWord,
    })
    if (!query.eq(this.query)) {
      this.query = query
      this.view.dispatch({ effects: setSearchQuery.of(query) })
    }
    this.setQuery(query)
    this.renderStatus()
  }

  private setQuery(query: SearchQuery): void {
    this.query = query
    this.searchField.value = query.search
    this.replaceField.value = query.replace
    this.caseButton.setAttribute("aria-pressed", String(query.caseSensitive))
    this.wordButton.setAttribute("aria-pressed", String(query.wholeWord))
    this.regexpButton.setAttribute("aria-pressed", String(query.regexp))
  }

  private renderStatus(state: EditorState = this.view.state): void {
    const status = searchMatchStatus(state, this.query)
    this.count.textContent = status.valid
      ? `${status.current} / ${status.total}`
      : "Invalid"
    this.count.setAttribute(
      "aria-label",
      status.valid
        ? status.total === 0
          ? "No matches"
          : `Match ${status.current} of ${status.total}`
        : "Invalid regular expression"
    )
    const disabled = !status.valid || status.total === 0
    this.previousButton.disabled = disabled
    this.nextButton.disabled = disabled
    this.replaceButton.disabled = disabled
    this.replaceAllButton.disabled = disabled
  }

  private keydown(event: KeyboardEvent): void {
    if (runScopeHandlers(this.view, event, "search-panel")) {
      event.preventDefault()
      return
    }
    if (event.key !== "Enter") return
    event.preventDefault()
    if (event.target === this.replaceField) {
      replaceNext(this.view)
      return
    }
    ;(event.shiftKey ? findPrevious : findNext)(this.view)
  }
}

export function createCompactSearchPanel(view: EditorView): Panel {
  const panel = new CompactSearchPanel(view)
  panels.set(view, panel)
  return panel
}
