const lessonTitle = document.querySelector("#tour-lesson-title")

function resetPageScroll(): void {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      document.scrollingElement?.scrollTo({
        top: 0,
        left: 0,
        behavior: "auto",
      })
      document.documentElement.scrollTop = 0
      document.body.scrollTop = 0
      window.scrollTo(0, 0)
    })
  })
}

if (lessonTitle instanceof HTMLElement) {
  let previousTitle = lessonTitle.textContent

  const observer = new MutationObserver(() => {
    const nextTitle = lessonTitle.textContent
    if (nextTitle === previousTitle) return
    previousTitle = nextTitle
    resetPageScroll()
  })

  observer.observe(lessonTitle, { childList: true })
}
