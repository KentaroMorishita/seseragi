export {}
const url = new URL("https://example.test/search?q=tea#results")
url.hash = ""
console.log(url.href)
url.hash = ""
console.log(url.href)
