export {}
function report(source: string): string {
  const value: unknown = JSON.parse(source)
  if (!Array.isArray(value)) return "Counts rejected"
  const counts: number[] = []
  for (const count of value) {
    if (typeof count !== "number" || !Number.isSafeInteger(count))
      return "Counts rejected"
    counts.push(count)
  }
  return JSON.stringify(counts)
}
for (const source of ["[2, 0, 5]", "[]", "[2, false, 5]"])
  console.log(report(source))
