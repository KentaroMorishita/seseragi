export {}
function checkLink(input: string): string {
  try {
    return new URL(input).href
  } catch {
    return "Absolute link rejected"
  }
}
console.log(checkLink("https://example.test/search"))
console.log(checkLink("/search"))
