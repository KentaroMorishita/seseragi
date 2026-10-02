export {}
const input =
  "https://example.test/search?q=old&page=3&tag=books&tag=music#summary"
const next = new URL(input)
next.searchParams.set("q", "tea & cake")
next.searchParams.delete("page")
next.searchParams.append("tag", "sale")
next.hash = "results"
console.log(next.href)
