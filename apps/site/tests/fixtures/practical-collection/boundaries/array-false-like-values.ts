const present = (value: number | boolean | string | undefined) =>
  value === undefined ? "missing" : "present:" + JSON.stringify(value)
console.log(present([0][0]))
console.log(present([false][0]))
console.log(present([""][0]))
console.log(present([0][0]))
console.log(present([false][0]))
console.log(present([""][0]))
console.log(present([1, 0, 0].find((value) => value === 0)))
console.log(present([true, false, false].find((value) => value === false)))
console.log(present(["x", "", ""].find((value) => value === "")))
console.log(JSON.stringify([1, 0, 0].filter((value) => value === 0)))
console.log(
  JSON.stringify([true, false, false].filter((value) => value === false))
)
console.log(JSON.stringify(["x", "", ""].filter((value) => value === "")))
export {}
