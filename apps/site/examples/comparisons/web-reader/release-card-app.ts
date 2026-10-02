const target = document.querySelector("#app")
if (!target) throw new Error("DOM target unavailable: #app")

let expanded = false
const section = document.createElement("section")
section.className = "release-card"
const heading = document.createElement("h2")
heading.textContent = "Release <notes>"
const details = document.createElement("p")
details.textContent = "Search & navigation improvements"
const button = document.createElement("button")
button.id = "toggle-details"
button.type = "button"

function render(): void {
  details.hidden = !expanded
  button.setAttribute("aria-expanded", String(expanded))
  button.textContent = expanded ? "Hide details" : "Show details"
}

function toggleDetails(): void {
  expanded = !expanded
  render()
}

button.addEventListener("click", toggleDetails)
section.append(heading, details, button)
render()
target.replaceChildren(section)

// The caller can use this when removing this version of the app.
export function unmount(): void {
  button.removeEventListener("click", toggleDetails)
  section.remove()
}
