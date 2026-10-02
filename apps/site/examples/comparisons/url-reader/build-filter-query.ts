export {}
const search = new URLSearchParams()
search.append("q", "tea & cake")
const selected = new URLSearchParams(search)
selected.append("tag", "books")
selected.append("tag", "music")
console.log(search.toString())
console.log(selected.toString())
