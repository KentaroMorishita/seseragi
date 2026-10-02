export {}
function sectionText(url: URL): string {
  return url.hash === ""
    ? "No section"
    : `Section: ${decodeURIComponent(url.hash.slice(1))}`
}
const original = new URL("https://example.test/search?q=tea")
const changed = new URL(original)
changed.hash = encodeURIComponent("results 2")
console.log(changed.href)
console.log(sectionText(changed))
console.log(sectionText(original))
