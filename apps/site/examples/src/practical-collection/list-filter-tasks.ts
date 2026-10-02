const tasks = [
  { title: "Read", urgent: false },
  { title: "Review", urgent: true },
  { title: "Ship", urgent: true },
]

console.log(
  JSON.stringify(tasks.filter((task) => task.urgent).map((task) => task.title))
)
console.log(JSON.stringify(tasks.map((task) => task.title)))
const none: typeof tasks = []
console.log(
  JSON.stringify(none.filter((task) => task.urgent).map((task) => task.title))
)
export {}
