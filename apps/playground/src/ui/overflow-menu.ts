type OverflowMenuElements = {
  readonly button: HTMLButtonElement
  readonly menu: HTMLElement
}

export function connectOverflowMenu({ button, menu }: OverflowMenuElements): {
  readonly close: () => void
} {
  const ownerDocument = menu.ownerDocument

  const close = (restoreFocus = false): void => {
    if (menu.hidden) return
    menu.hidden = true
    button.setAttribute("aria-expanded", "false")
    if (restoreFocus) button.focus()
  }

  const open = (): void => {
    menu.hidden = false
    button.setAttribute("aria-expanded", "true")
    menu.querySelector<HTMLElement>("[role^='menuitem']")?.focus()
  }

  button.addEventListener("click", () => {
    if (menu.hidden) open()
    else close(true)
  })
  menu.addEventListener("click", (event) => {
    if ((event.target as Element).closest("[role^='menuitem']")) close()
  })
  menu.addEventListener("keydown", (event) => {
    const items = [
      ...menu.querySelectorAll<HTMLButtonElement>("[role^='menuitem']"),
    ]
    const current = items.indexOf(
      ownerDocument.activeElement as HTMLButtonElement
    )
    const next =
      event.key === "ArrowDown"
        ? (current + 1) % items.length
        : event.key === "ArrowUp"
          ? (current - 1 + items.length) % items.length
          : event.key === "Home"
            ? 0
            : event.key === "End"
              ? items.length - 1
              : undefined
    if (next === undefined || items.length === 0) return
    event.preventDefault()
    items[next]?.focus()
  })
  ownerDocument.addEventListener("pointerdown", (event) => {
    const target = event.target as Node
    if (!menu.hidden && !menu.contains(target) && !button.contains(target)) {
      close()
    }
  })
  ownerDocument.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || menu.hidden) return
    event.preventDefault()
    close(true)
  })

  return { close: () => close() }
}
