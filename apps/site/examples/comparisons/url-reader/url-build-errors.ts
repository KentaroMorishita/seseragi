export {}
// This example checks the same HTTP(S), login-detail and percent-escape policy.
function checkLink(input: string): string {
  const offset = input.search(/%(?![0-9a-fA-F]{2})/)
  if (offset >= 0) return `Invalid percent escape at ${offset}`
  let url: URL
  try {
    url = new URL(input)
  } catch {
    return "Invalid URL at 0"
  }
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    return `Unsupported scheme: ${url.protocol.slice(0, -1)}`
  }
  if (url.username || url.password) return "Remove login details"
  return url.href
}
console.log(checkLink("https://example.test/search"))
console.log(checkLink("/search"))
console.log(checkLink("ftp://example.test/"))
console.log(checkLink("https://user:pass@example.test/"))
console.log(checkLink("https://example.test/%ZZ"))
