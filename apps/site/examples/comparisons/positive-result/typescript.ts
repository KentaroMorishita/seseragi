function positive(value: number): number {
  if (value <= 0) throw new Error("must be positive")
  return value
}

function describe(value: number): string {
  try {
    return `ok: ${positive(value)}`
  } catch (error) {
    if (error instanceof Error) return `error: ${error.message}`
    throw error
  }
}

console.log(describe(2))
console.log(describe(0))
