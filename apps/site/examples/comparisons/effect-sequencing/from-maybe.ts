function display(value: number | undefined): void {
  console.log(
    value === undefined ? "Rejected: count is missing" : `Count: ${value}`
  )
}
display(2)
display(0)
display(undefined)
export {}
