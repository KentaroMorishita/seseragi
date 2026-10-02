const tasks = [
  { title: "Read", urgent: false },
  { title: "Review", urgent: true },
  { title: "Ship", urgent: true },
]

const describe = (task: { title: string } | undefined) =>
  task === undefined ? "not found" : task.title
console.log(describe(tasks.find((task) => task.urgent)))
console.log(describe(tasks.find((task) => task.title === "Missing")))
const none: typeof tasks = []
console.log(describe(none.find((task) => task.urgent)))
console.log(JSON.stringify(tasks.map((task) => task.title)))
export {}
