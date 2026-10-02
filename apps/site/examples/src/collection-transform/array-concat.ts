const display = (value: unknown): void => console.log(JSON.stringify(value))
const batches = [["Read"], [], ["Review", "Ship"]]
display(batches.flat())

export {}
