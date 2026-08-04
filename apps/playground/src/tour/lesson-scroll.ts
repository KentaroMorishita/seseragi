const lessonTitle = document.querySelector("#tour-lesson-title")

if (lessonTitle instanceof HTMLElement) {
  const observer = new MutationObserver(() => {
    requestAnimationFrame(() => {
      window.scrollTo({ top: 0, left: 0, behavior: "auto" })
    })
  })

  observer.observe(lessonTitle, { childList: true })
}
