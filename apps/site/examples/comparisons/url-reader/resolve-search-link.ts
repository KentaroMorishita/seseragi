export {}
const reference = "../search?tag=books"
console.log(new URL(reference, "https://example.test/catalog/item").href)
console.log(new URL(reference, "https://example.test/catalog/item/").href)
