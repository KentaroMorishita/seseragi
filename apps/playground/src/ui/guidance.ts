// 用意した例の案内（DESIGN.md「案内は初回だけ、消せて、ふだんは静か」、#1350 の全体設計 2）。
// - ひとこと（コーチマーク）: 初めて開いた最初の例だけ。Close、書き換え、Run のどれかで消え、二度と出さない
// - Try next: 用意した例を開いたときに、出力の下に並べる。Hide で一行に畳み、その選択を覚える
// - Settings の Hints をオフにすると、ひとこと、Try next、開いたときの自動実行を止める
// 文字は日本語、ボタンのラベル（Try next、Hide、Show、Close）は英語のまま。
import type { SampleTryNext } from "../sample-catalog"

type GuidanceStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">

const keys = {
  hints: "seseragi.hints.enabled",
  coach: "seseragi.hints.coach-done",
  collapsed: "seseragi.hints.try-next-collapsed",
} as const

export type GuidancePreferences = Readonly<{
  hintsEnabled: () => boolean
  setHintsEnabled: (enabled: boolean) => void
  coachDone: () => boolean
  markCoachDone: () => void
  tryNextCollapsed: () => boolean
  setTryNextCollapsed: (collapsed: boolean) => void
}>

export function createGuidancePreferences(
  storage: GuidanceStorage
): GuidancePreferences {
  const read = (key: string): string | null => {
    try {
      return storage.getItem(key)
    } catch {
      return null
    }
  }
  const write = (key: string, value: string | undefined): void => {
    try {
      if (value === undefined) storage.removeItem(key)
      else storage.setItem(key, value)
    } catch {
      // 保存できない環境でも、この画面を開いているあいだは今の選択で動く。
    }
  }
  let memory = {
    hints: read(keys.hints) !== "off",
    coach: read(keys.coach) === "done",
    collapsed: read(keys.collapsed) === "true",
  }
  return {
    hintsEnabled: () => memory.hints,
    setHintsEnabled(enabled) {
      memory = { ...memory, hints: enabled }
      write(keys.hints, enabled ? undefined : "off")
      // オンに戻したら、ひとことをもう一度出せるようにする
      if (enabled) {
        memory = { ...memory, coach: false }
        write(keys.coach, undefined)
      }
    },
    coachDone: () => memory.coach,
    markCoachDone() {
      memory = { ...memory, coach: true }
      write(keys.coach, "done")
    },
    tryNextCollapsed: () => memory.collapsed,
    setTryNextCollapsed(collapsed) {
      memory = { ...memory, collapsed }
      write(keys.collapsed, collapsed ? "true" : undefined)
    },
  }
}

/** チップの書き換えを、順にコードへ入れる。入れられないときは undefined */
export function applyTryNext(
  source: string,
  chip: SampleTryNext
): string | undefined {
  let next = source
  for (const { find, replace } of chip.edits) {
    const index = next.indexOf(find)
    if (index < 0) return undefined
    next = next.slice(0, index) + replace + next.slice(index + find.length)
  }
  return next
}

export function createCoachCard(
  ownerDocument: Document,
  hint: Readonly<{ title: string; body: string }>,
  options: Readonly<{ tourHref: string; onClose: () => void }>
): HTMLElement {
  const card = ownerDocument.createElement("div")
  card.className = "pg-coach"
  card.setAttribute("role", "note")
  const title = ownerDocument.createElement("strong")
  title.textContent = hint.title
  const body = ownerDocument.createElement("span")
  body.textContent = hint.body
  const row = ownerDocument.createElement("div")
  row.className = "pg-coach-row"
  const tour = ownerDocument.createElement("a")
  tour.href = options.tourHref
  tour.textContent = "Tour で基本から →"
  const close = ownerDocument.createElement("button")
  close.type = "button"
  close.textContent = "Close"
  close.addEventListener("click", options.onClose)
  row.append(tour, close)
  card.append(title, body, row)
  return card
}

export type TryNextView = Readonly<{
  /** 用意した例のチップを出す。undefined なら隠す（自由に書く状態） */
  show: (chips: readonly SampleTryNext[] | undefined) => void
  markUsed: (index: number) => void
}>

export function connectTryNext(
  element: HTMLElement,
  options: Readonly<{
    preferences: GuidancePreferences
    tourHref: string
    onApply: (chip: SampleTryNext, index: number) => void
  }>
): TryNextView {
  const ownerDocument = element.ownerDocument
  let chips: readonly SampleTryNext[] = []
  const used = new Set<number>()

  const render = (): void => {
    const visible = options.preferences.hintsEnabled() && chips.length > 0
    element.hidden = !visible
    if (!visible) return
    const collapsed = options.preferences.tryNextCollapsed()
    element.dataset.collapsed = String(collapsed)
    const head = ownerDocument.createElement("div")
    head.className = "pg-try-next-head"
    const title = ownerDocument.createElement("h3")
    title.textContent = "Try next"
    const count = ownerDocument.createElement("span")
    count.className = "pg-try-next-count"
    count.textContent = String(chips.length + 1)
    const toggle = ownerDocument.createElement("button")
    toggle.type = "button"
    toggle.className = "ui-button ui-button--ghost ui-button--sm"
    toggle.textContent = collapsed ? "Show" : "Hide"
    toggle.setAttribute("aria-expanded", String(!collapsed))
    toggle.addEventListener("click", () => {
      options.preferences.setTryNextCollapsed(!collapsed)
      render()
    })
    head.append(title, count, ownerDocument.createElement("span"), toggle)
    head.children[2]!.className = "ui-spacer"
    const list = ownerDocument.createElement("div")
    list.className = "pg-try-next-chips"
    list.hidden = collapsed
    list.append(
      ...chips.map((chip, index) => {
        const button = ownerDocument.createElement("button")
        button.type = "button"
        button.className = "ui-chip pg-try-chip"
        button.dataset.tryNext = String(index)
        button.textContent = chip.label
        if (used.has(index)) {
          button.disabled = true
          button.setAttribute("aria-label", `${chip.label}（入れました）`)
        }
        button.addEventListener("click", () => options.onApply(chip, index))
        return button
      })
    )
    const tour = ownerDocument.createElement("a")
    tour.className = "ui-chip ui-chip--link pg-try-chip"
    tour.href = options.tourHref
    tour.textContent = "基本から順に学ぶ →"
    list.append(tour)
    element.replaceChildren(head, list)
  }

  return {
    show(next) {
      chips = next ?? []
      used.clear()
      render()
    },
    markUsed(index) {
      used.add(index)
      render()
    },
  }
}
