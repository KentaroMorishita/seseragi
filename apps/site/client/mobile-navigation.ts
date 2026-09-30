// The Seseragi component owns all navigation markup and translated copy.
// This adapter only supplies native browser modality and scroll restoration.
const navigation = document.querySelector(".mobile-docs-navigation")
const trigger = navigation?.querySelector("summary")
const drawer = navigation?.querySelector("dialog")
const closeButton = drawer?.querySelector(".mobile-navigation-close")

if (
  navigation instanceof HTMLDetailsElement &&
  trigger instanceof HTMLElement &&
  drawer instanceof HTMLDialogElement &&
  closeButton instanceof HTMLButtonElement &&
  typeof drawer.showModal === "function"
) {
  const mobile = matchMedia("(max-width: 760px)")
  let scrollPosition = 0
  let previousTop = ""
  navigation.dataset.enhanced = "true"
  trigger.setAttribute("aria-controls", drawer.id)
  trigger.setAttribute("aria-haspopup", "dialog")
  trigger.setAttribute("aria-expanded", "false")

  const restore = () => {
    if (!navigation.open) return
    navigation.open = false
    trigger.setAttribute("aria-expanded", "false")
    document.documentElement.classList.remove("navigation-is-open")
    document.body.style.top = previousTop
    window.scrollTo({ top: scrollPosition, behavior: "instant" })
    if (mobile.matches) trigger.focus({ preventScroll: true })
  }
  const close = () => {
    if (drawer.open) drawer.close()
    restore()
  }

  trigger.addEventListener("click", (event) => {
    event.preventDefault()
    if (!mobile.matches || drawer.open) return
    scrollPosition = window.scrollY
    previousTop = document.body.style.top
    navigation.open = true
    drawer.showModal()
    trigger.setAttribute("aria-expanded", "true")
    document.body.style.top = `-${scrollPosition}px`
    document.documentElement.classList.add("navigation-is-open")
    closeButton.focus({ preventScroll: true })
    drawer.querySelector(".sidebar-link.current")?.scrollIntoView({
      block: "nearest",
    })
  })

  drawer.addEventListener("close", () => {
    // A queued close event from a previous opening must not unlock a new one.
    if (!drawer.open) restore()
  })
  drawer.addEventListener("cancel", (event) => {
    event.preventDefault()
    close()
  })

  closeButton.addEventListener("click", close)
  const isBackdrop = (event: MouseEvent) => {
    const bounds = drawer.getBoundingClientRect()
    // Backdrop clicks are retargeted to the dialog; blank space inside is not.
    return (
      event.target === drawer &&
      (event.clientX < bounds.left ||
        event.clientX > bounds.right ||
        event.clientY < bounds.top ||
        event.clientY > bounds.bottom)
    )
  }
  let backdropPointerDown = false
  drawer.addEventListener("pointerdown", (event) => {
    backdropPointerDown = isBackdrop(event)
  })
  drawer.addEventListener("click", (event) => {
    if (backdropPointerDown && isBackdrop(event)) close()
    backdropPointerDown = false
    if (event.target instanceof Element && event.target.closest("a[href]")) {
      close()
    }
  })
  mobile.addEventListener("change", () => {
    if (!mobile.matches) close()
  })
  window.addEventListener("pagehide", close)
}
