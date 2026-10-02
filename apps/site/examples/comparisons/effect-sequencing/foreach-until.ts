function preview(labels: string[]): void {
  for (const label of labels) {
    console.log(`visit: ${label}`)
    if (label === "bad") {
      console.log("Rejected: bad label")
      return
    }
    if (label === "stop") break
  }
  console.log("Completed")
}
preview(["one", "stop", "after"])
preview(["one", "two"])
preview([])
preview(["stop", "after"])
preview(["one", "bad", "after"])
export {}
