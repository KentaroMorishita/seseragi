const lessonTitle = document.querySelector("#tour-lesson-title")

if (lessonTitle instanceof HTMLElement) {
  let previousTitle = lessonTitle.textContent

  const observer = new MutationObserver(() => {
    const nextTitle = lessonTitle.textContent
    if (nextTitle === previousTitle) return
    previousTitle = nextTitle

    requestAnimationFrame(() => {
      window.scrollTo({ top: 0, left: 0, behavior: "auto" })
    })
  })

  observer.observe(lessonTitle, { childList: true })
}
