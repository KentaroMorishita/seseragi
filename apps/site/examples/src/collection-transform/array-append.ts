const display = (value: unknown): void => console.log(JSON.stringify(value))
const firstBatch = ["Read", "Review"]
const nextBatch = ["Ship"]
display([...firstBatch, ...nextBatch])
display(firstBatch.concat(nextBatch))
display(firstBatch)
display(nextBatch)

export {}
