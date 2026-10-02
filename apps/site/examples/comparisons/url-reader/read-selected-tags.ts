export {}
const query = new URLSearchParams("tag=books&q=tea+cake&tag=music&empty=")
console.log(query.getAll("tag").join(", "))
console.log(`Missing count: ${query.getAll("missing").length}`)
console.log(`Empty count: ${query.getAll("empty").length}`)
console.log([...query].map(([name, value]) => `${name}=${value}`).join(" | "))
