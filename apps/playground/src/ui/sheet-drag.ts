// 狭い画面の出力のシートを、指で引き上げ・引き下げできるようにする。
// - 持ち手と出力の見出し（ボタンなどの操作部品の上は除く）を掴むと、シートが指に合わせて伸び縮みする。
// - 指を離すと、動かした向きと距離で、顔を出した状態（"code"）か開いた状態（"io"）に収まる。
// - ほとんど動かさずに離したときは、タップとして扱う（持ち手のタップは開閉を切り替える）。
import type { MobilePanel } from "./mobile-panels"

/** これより小さい動きは、ドラッグではなくタップとして扱う */
export const sheetDragSlop = 6
/** これ以上動かしたら、その向きの状態に収める */
export const sheetSnapDistance = 40

export type SheetSnapInput = Readonly<{
  /** 掴んでから上に動かした量（px）。下に動かすと負 */
  deltaY: number
  /** 指を離したときのシートの高さ（px） */
  height: number
  /** 顔を出した状態の高さ（px） */
  peekHeight: number
  /** 開いた状態の高さ（px） */
  openHeight: number
}>

export function sheetSnapTarget(input: SheetSnapInput): MobilePanel {
  if (input.deltaY >= sheetSnapDistance) return "io"
  if (input.deltaY <= -sheetSnapDistance) return "code"
  const middle = (input.peekHeight + input.openHeight) / 2
  return input.height >= middle ? "io" : "code"
}

export function clampSheetHeight(
  height: number,
  peekHeight: number,
  maxHeight: number
): number {
  return Math.min(Math.max(height, peekHeight), maxHeight)
}

const interactive = "button, a, input, select, textarea, [role='tab']"

export function connectSheetDrag(
  options: Readonly<{
    sheet: HTMLElement
    handle: HTMLElement
    grips: readonly HTMLElement[]
    panels: Readonly<{
      show: (panel: MobilePanel) => void
      current: () => MobilePanel
    }>
    isSheet: () => boolean
    peekHeight: () => number
    openHeight: () => number
  }>
): void {
  let drag:
    | {
        pointerId: number
        target: HTMLElement
        startY: number
        startHeight: number
        moved: boolean
      }
    | undefined
  let suppressClick = false

  const begin = (event: PointerEvent): void => {
    if (!options.isSheet() || !event.isPrimary || event.button > 0) return
    const target = event.currentTarget as HTMLElement
    const origin = event.target instanceof Element ? event.target : null
    if (target !== options.handle && origin?.closest(interactive)) return
    drag = {
      pointerId: event.pointerId,
      target,
      startY: event.clientY,
      startHeight: options.sheet.getBoundingClientRect().height,
      moved: false,
    }
    target.setPointerCapture?.(event.pointerId)
  }

  const move = (event: PointerEvent): void => {
    if (drag === undefined || event.pointerId !== drag.pointerId) return
    const deltaY = drag.startY - event.clientY
    if (!drag.moved && Math.abs(deltaY) < sheetDragSlop) return
    drag.moved = true
    event.preventDefault()
    options.sheet.dataset.dragging = "true"
    const height = clampSheetHeight(
      drag.startHeight + deltaY,
      options.peekHeight(),
      Math.max(options.openHeight(), window.innerHeight * 0.9)
    )
    options.sheet.style.height = `${height}px`
  }

  const end = (event: PointerEvent): void => {
    if (drag === undefined || event.pointerId !== drag.pointerId) return
    const current = drag
    drag = undefined
    current.target.releasePointerCapture?.(event.pointerId)
    if (!current.moved) return
    const height = options.sheet.getBoundingClientRect().height
    delete options.sheet.dataset.dragging
    options.sheet.style.removeProperty("height")
    suppressClick = true
    window.setTimeout(() => {
      suppressClick = false
    }, 0)
    if (event.type === "pointercancel") return
    options.panels.show(
      sheetSnapTarget({
        deltaY: current.startY - event.clientY,
        height,
        peekHeight: options.peekHeight(),
        openHeight: options.openHeight(),
      })
    )
  }

  for (const grip of [options.handle, ...options.grips]) {
    grip.addEventListener("pointerdown", begin)
    grip.addEventListener("pointermove", move)
    grip.addEventListener("pointerup", end)
    grip.addEventListener("pointercancel", end)
  }
  // ドラッグを終えた直後のクリックは、持ち手のタップ（開閉）として扱わない。
  // 祖先のシートで先に捕まえ、持ち手の切り替えまで届かないようにする。
  options.sheet.addEventListener(
    "click",
    (event) => {
      if (!suppressClick) return
      if (!(event.target instanceof Node)) return
      if (!options.handle.contains(event.target)) return
      suppressClick = false
      event.stopPropagation()
      event.preventDefault()
    },
    true
  )
}
