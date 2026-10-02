const labels = ["", "Review"]
const describe = (label: string | undefined) =>
  label === undefined ? "missing" : "present: " + JSON.stringify(label)
for (const index of [0, 1, -1, 2]) console.log(describe(labels[index]))
console.log(describe(([] as string[])[0]))
console.log(describe(labels[0]))
console.log(describe(([] as string[])[0]))
export {}
