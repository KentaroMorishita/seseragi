// Search（⌘K）の画面。入力欄、範囲のチップ、結果の一覧、API の詳しい説明、キーの案内を並べる。
// 一覧の作り方と並べ方は search-index.ts に置き、ここではキーボードとマウスの操作だけを扱う。
// 入力欄は combobox、結果は listbox とし、選んでいる結果は aria-activedescendant で伝える。
import type { AnalysisReferenceItem } from "../compiler/types"
import {
  apiEntries,
  normalizeSearchText,
  type SearchEntry,
  type SearchFilter,
  type SearchScope,
  searchFilters,
  searchSections,
} from "./search-index"

export type SearchCommand = Readonly<{
  id: string
  title: string
  detail: string
  keywords?: string
  // 今の状態を示す小さな印（"Current" など）。なければ出さない
  tag?: () => string | undefined
  visible?: () => boolean
  run: () => undefined
}>

export type SearchPaletteOptions = Readonly<{
  dialog: HTMLDialogElement
  input: HTMLInputElement
  scopes: HTMLElement
  results: HTMLElement
  detail: HTMLElement
  examples: readonly SearchEntry[]
  onExample: (id: string) => boolean | undefined
  loadLessons: () => Promise<readonly SearchEntry[]>
  onLesson: (id: string) => void
  commands: readonly SearchCommand[]
  renderApiDetail: (item: AnalysisReferenceItem) => HTMLElement
}>

export type SearchPalette = Readonly<{
  open: (
    options?: Readonly<{ filter?: SearchFilter; opener?: HTMLElement }>
  ) => void
  close: () => void
  setApiCatalog: (items: readonly AnalysisReferenceItem[]) => void
  setCurrentExample: (id: string | undefined) => void
}>

const icons: Readonly<Record<SearchScope, string>> = {
  examples: '<path d="M8 2 13.5 5v6L8 14 2.5 11V5zM2.5 5 8 8l5.5-3M8 8v6" />',
  lessons:
    '<path d="M2.5 3.5h4A1.5 1.5 0 0 1 8 5v8a1.5 1.5 0 0 0-1.5-1.5h-4zM13.5 3.5h-4A1.5 1.5 0 0 0 8 5v8a1.5 1.5 0 0 1 1.5-1.5h4z" />',
  api: '<path d="M10 2.5c-1.5 0-2 .8-2.3 2.2L6.4 11c-.3 1.6-.9 2.5-2.4 2.5M5 6.5h5" />',
  settings:
    '<path d="M2.5 4.5h7M12.5 4.5h1M2.5 11.5h1M6.5 11.5h7" /><circle cx="11" cy="4.5" r="1.5" /><circle cx="5" cy="11.5" r="1.5" />',
}

export function connectSearchPalette(
  options: SearchPaletteOptions
): SearchPalette {
  const { dialog, input, scopes, results, detail } = options
  const ownerDocument = dialog.ownerDocument
  let filter: SearchFilter = "all"
  let lessons: readonly SearchEntry[] = []
  let lessonsRequested = false
  let apiItems: readonly AnalysisReferenceItem[] = []
  let api: readonly SearchEntry[] = []
  let currentExample: string | undefined
  let visible: readonly SearchEntry[] = []
  let active = 0
  let opener: HTMLElement | undefined

  const commandEntries = (): readonly SearchEntry[] =>
    options.commands
      .filter((command) => command.visible?.() ?? true)
      .map((command) => {
        const tag = command.tag?.()
        return {
          key: `settings:${command.id}`,
          scope: "settings" as const,
          section: "Settings",
          title: command.title,
          detail: command.detail,
          ...(tag === undefined ? {} : { tag }),
          searchText: normalizeSearchText(
            [command.title, command.detail, command.keywords ?? ""].join(" ")
          ),
        }
      })

  const filterButtons = searchFilters.map(({ id, label }) => {
    const button = ownerDocument.createElement("button")
    button.type = "button"
    button.className = "pg-search-scope"
    button.dataset.filter = id
    button.textContent = label
    button.addEventListener("click", () => {
      setFilter(id)
      input.focus()
    })
    return button
  })
  scopes.replaceChildren(...filterButtons)

  const setFilter = (next: SearchFilter): void => {
    filter = next
    active = 0
    for (const button of filterButtons) {
      button.setAttribute(
        "aria-pressed",
        String(button.dataset.filter === next)
      )
    }
    render()
  }

  const optionId = (index: number): string => `search-option-${index}`

  const render = (): void => {
    if (!dialog.open) return
    const sections = searchSections(
      [...options.examples, ...lessons, ...api, ...commandEntries()],
      input.value,
      filter
    )
    visible = sections.flatMap(({ entries }) => entries)
    if (active >= visible.length) active = Math.max(0, visible.length - 1)
    let index = 0
    const groups = sections.map((section) => {
      const group = ownerDocument.createElement("div")
      group.className = "pg-search-group"
      group.setAttribute("role", "group")
      const heading = ownerDocument.createElement("h3")
      heading.id = `search-group-${section.id}`
      heading.textContent = section.title
      group.setAttribute("aria-labelledby", heading.id)
      group.append(heading)
      for (const entry of section.entries) {
        group.append(createOption(entry, index))
        index += 1
      }
      return group
    })
    if (groups.length === 0) {
      const empty = ownerDocument.createElement("p")
      empty.className = "pg-search-empty"
      empty.textContent =
        filter === "lessons" && !lessonsLoaded()
          ? "レッスンを読み込んでいます。"
          : "見つかりませんでした。言葉を変えるか、範囲を All に戻してください。"
      results.replaceChildren(empty)
    } else {
      results.replaceChildren(...groups)
    }
    markActive()
  }

  const lessonsLoaded = (): boolean => lessons.length > 0

  const createOption = (entry: SearchEntry, index: number): HTMLElement => {
    const option = ownerDocument.createElement("div")
    option.id = optionId(index)
    option.className = "pg-search-result"
    option.setAttribute("role", "option")
    option.dataset.scope = entry.scope
    const [, id] = splitKey(entry.key)
    if (entry.scope === "examples") option.dataset.sampleId = id
    if (entry.scope === "lessons") option.dataset.lessonId = id
    if (entry.scope === "settings") option.dataset.command = id
    if (entry.scope === "examples" && id === currentExample) {
      option.setAttribute("aria-current", "true")
    }

    const icon = ownerDocument.createElement("span")
    icon.className = "pg-search-icon"
    icon.innerHTML = `<svg class="ui-icon" viewBox="0 0 16 16" aria-hidden="true">${icons[entry.scope]}</svg>`
    const text = ownerDocument.createElement("span")
    text.className = "pg-search-text"
    const title = ownerDocument.createElement("strong")
    title.textContent = entry.title
    const sub = ownerDocument.createElement("small")
    if (entry.signature === undefined) {
      sub.textContent = entry.detail
    } else {
      sub.className = "pg-search-signature"
      sub.textContent = entry.signature
    }
    text.append(title, sub)
    option.append(icon, text)
    if (entry.tag !== undefined) {
      const tag = ownerDocument.createElement("span")
      tag.className = "ui-tag"
      tag.textContent = entry.tag
      option.append(tag)
    }
    option.addEventListener("pointermove", () => {
      if (active === index) return
      active = index
      markActive()
    })
    option.addEventListener("click", () => {
      active = index
      markActive()
      activate()
    })
    return option
  }

  const markActive = (): void => {
    const options = results.querySelectorAll<HTMLElement>(".pg-search-result")
    for (const [index, option] of options.entries()) {
      const selected = index === active
      option.setAttribute("aria-selected", String(selected))
      if (selected) option.scrollIntoView({ block: "nearest" })
    }
    if (visible.length === 0) input.removeAttribute("aria-activedescendant")
    else input.setAttribute("aria-activedescendant", optionId(active))
    renderDetail()
  }

  // API の結果を選んでいるときだけ、その説明（シグネチャ、説明、制約）を下に出す。
  const renderDetail = (): void => {
    const entry = visible[active]
    const item =
      entry?.scope === "api" ? apiItems[apiIndex(entry.key)] : undefined
    if (item === undefined) {
      detail.hidden = true
      detail.replaceChildren()
      return
    }
    detail.hidden = false
    detail.replaceChildren(options.renderApiDetail(item))
  }

  const activate = (): void => {
    const entry = visible[active]
    if (entry === undefined) return
    const [, id] = splitKey(entry.key)
    if (entry.scope === "examples") {
      if (options.onExample(id) === false) return
      close()
      return
    }
    if (entry.scope === "lessons") {
      close()
      options.onLesson(id)
      return
    }
    if (entry.scope === "settings") {
      const command = options.commands.find((candidate) => candidate.id === id)
      if (command === undefined) return
      // 先に閉じてから実行する。設定のダイアログを開く操作が、閉じたあとの
      // フォーカスの戻り先に邪魔されないようにする。
      close()
      command.run()
    }
    // API は説明を読むためのもの。選ぶと説明が下に出ている。
  }

  input.addEventListener("input", () => {
    active = 0
    render()
  })
  input.addEventListener("keydown", (event) => {
    if (event.isComposing) return
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault()
      if (visible.length === 0) return
      const step = event.key === "ArrowDown" ? 1 : -1
      active = (active + step + visible.length) % visible.length
      markActive()
      return
    }
    if (event.key === "Enter") {
      event.preventDefault()
      activate()
    }
  })
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) close()
  })
  dialog.addEventListener("close", () => {
    results.replaceChildren()
    detail.replaceChildren()
    detail.hidden = true
    visible = []
    opener?.setAttribute("aria-expanded", "false")
    opener?.focus()
    opener = undefined
  })

  const ensureLessons = (): void => {
    if (lessonsRequested) return
    lessonsRequested = true
    void options
      .loadLessons()
      .then((loaded) => {
        lessons = loaded
        render()
      })
      .catch(() => {
        lessonsRequested = false
      })
  }

  const open: SearchPalette["open"] = (openOptions = {}) => {
    opener = openOptions.opener
    opener?.setAttribute("aria-expanded", "true")
    if (!dialog.open) dialog.showModal()
    setFilter(openOptions.filter ?? "all")
    ensureLessons()
    // 前に打った言葉は残し、全体を選んでおく。続けて打てば置き換わる。
    input.focus()
    input.select()
  }

  const close = (): void => {
    if (dialog.open) dialog.close()
  }

  return {
    open,
    close,
    setApiCatalog: (items) => {
      apiItems = items
      api = apiEntries(items)
      // 解析は Search を閉じているあいだも届く。閉じているときは描かない。
      if (dialog.open) render()
    },
    setCurrentExample: (id) => {
      currentExample = id
    },
  }
}

function splitKey(key: string): readonly [string, string] {
  const separator = key.indexOf(":")
  return [key.slice(0, separator), key.slice(separator + 1)]
}

function apiIndex(key: string): number {
  return Number.parseInt(splitKey(key)[1] ?? "", 10)
}
