export {}
const url = new URL(
  "HTTPS://EXAMPLE.TEST:443/a/../search?q=tea%20cake~#results"
)
console.log(url.href)
url.search = url.searchParams.toString()
console.log(url.href)
