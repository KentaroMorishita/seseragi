export {}
const query = new URLSearchParams("q=tea&page=3&tag=books&page=4")
query.delete("page")
console.log(query.toString())
query.delete("missing")
console.log(query.toString())
