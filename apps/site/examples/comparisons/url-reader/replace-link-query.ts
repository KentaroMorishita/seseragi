export {}
const original = new URL("https://example.test/search?tag=books&page=3#results")
const oldQuery = new URLSearchParams(original.search)
const replacement = new URLSearchParams({ q: "tea" })
const changed = new URL(original)
changed.search = replacement.toString()
const cleared = new URL(original)
cleared.search = ""
console.log(oldQuery.toString())
console.log(changed.href)
console.log(cleared.href)
console.log(original.href)
