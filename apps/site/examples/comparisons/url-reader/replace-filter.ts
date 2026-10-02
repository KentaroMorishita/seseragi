export {}
const original = new URLSearchParams("q=tea&tag=books&page=3&tag=music")
const changed = new URLSearchParams(original)
changed.set("tag", "sale")
console.log(changed.toString())
console.log(original.toString())
