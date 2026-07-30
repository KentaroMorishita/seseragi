type ResizeHandle = Pick<
  HTMLElement,
  | "dataset"
  | "hasPointerCapture"
  | "releasePointerCapture"
  | "setPointerCapture"
>

let activeResize:
  | Readonly<{
      handle: ResizeHandle
      pointerId: number
    }>
  | undefined

export function beginExclusiveResize(
  handle: ResizeHandle,
  pointerId: number
): boolean {
  if (activeResize !== undefined) return false
  handle.setPointerCapture(pointerId)
  activeResize = { handle, pointerId }
  handle.dataset.dragging = "true"
  return true
}

export function ownsExclusiveResize(
  handle: ResizeHandle,
  pointerId: number
): boolean {
  return (
    activeResize?.handle === handle &&
    activeResize.pointerId === pointerId &&
    handle.hasPointerCapture(pointerId)
  )
}

export function finishExclusiveResize(
  handle: ResizeHandle,
  pointerId: number
): boolean {
  if (activeResize?.handle !== handle || activeResize.pointerId !== pointerId) {
    return false
  }
  activeResize = undefined
  if (handle.hasPointerCapture(pointerId)) {
    handle.releasePointerCapture(pointerId)
  }
  delete handle.dataset.dragging
  return true
}
