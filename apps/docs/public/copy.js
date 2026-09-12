const status = document.querySelector("#docs-copy-status")
document.documentElement.classList.add("docs-enhanced")

document.addEventListener("click", async (event) => {
  const target = event.target
  if (!(target instanceof Element)) return
  const button = target.closest("[data-copy-text]")
  if (!(button instanceof HTMLButtonElement)) return

  const source = button.dataset.copyText
  if (source === undefined) return
  try {
    await navigator.clipboard.writeText(source)
    button.textContent = "コピー済み"
    if (status !== null)
      status.textContent = "コードをクリップボードへコピーしました"
  } catch {
    button.textContent = "コピー失敗"
    if (status !== null) status.textContent = "コピーできませんでした"
  }
})
