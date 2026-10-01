const tags: Set<string> = new Set(["work", "home", "work"])
const updated = new Set(tags).add("travel")
console.log(JSON.stringify([...tags]))
console.log(JSON.stringify([...updated]))
const reordered = new Set(["home", "work"])
console.log(
  tags.size === reordered.size && [...tags].every((tag) => reordered.has(tag))
)
const empty: Set<string> = new Set()
console.log(empty.size)

export {}
