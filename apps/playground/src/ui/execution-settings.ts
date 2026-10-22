/** Network permission is temporary and cleared when the program changes. */
export function connectExecutionSettings(
  menu: HTMLElement,
  onChange: () => void
) {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("role", "menuitemcheckbox")
  button.setAttribute(
    "aria-label",
    "Allow preview HTTP, WebSocket and external images"
  )
  button.title =
    "このコードの外部通信・外部画像を許可します。入力や保存データが送信される可能性があります。コードを変更すると解除されます。"
  let allowed = false
  const render = () => {
    button.setAttribute("aria-checked", String(allowed))
    button.textContent = `${allowed ? "☑" : "☐"} Network（外部通信・画像）`
  }
  button.addEventListener("click", () => {
    allowed = !allowed
    render()
    onChange()
  })
  render()
  menu.append(button)
  return {
    allowed: () => allowed,
    reset: () => {
      allowed = false
      render()
    },
  }
}
