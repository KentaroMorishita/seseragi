function renderTasks(tasks: readonly string[]): string {
  return JSON.stringify(tasks)
}
const tasks = ["Read", "Review", "Ship"]
console.log(renderTasks(tasks))
console.log(JSON.stringify(tasks))

export {}
