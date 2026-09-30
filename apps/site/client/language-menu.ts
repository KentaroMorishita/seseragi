// Routes, language labels and selection remain owned by the Seseragi component.
// Native details/links work without JS; this adds dismissal and focus behavior.
for (const menu of document.querySelectorAll<HTMLDetailsElement>(
  ".language-menu"
)) {
  const trigger = menu.querySelector<HTMLElement>(".language-trigger")
  if (!trigger) continue
  trigger.setAttribute("aria-expanded", String(menu.open))
  menu.addEventListener("toggle", () => {
    trigger.setAttribute("aria-expanded", String(menu.open))
  })
  document.addEventListener("pointerdown", (event) => {
    if (event.target instanceof Node && !menu.contains(event.target))
      menu.open = false
  })
  menu.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || !menu.open) return
    event.preventDefault()
    menu.open = false
    trigger.focus()
  })
  menu.addEventListener("focusout", (event) => {
    // activeElement can temporarily be body between focusout and focusin.
    // Use the destination to avoid closing while Tab enters a language link.
    if (event.relatedTarget instanceof Node) {
      if (!menu.contains(event.relatedTarget)) menu.open = false
      return
    }
    requestAnimationFrame(() => {
      if (!menu.contains(document.activeElement)) menu.open = false
    })
  })
  window.addEventListener("pagehide", () => {
    menu.open = false
  })
}
