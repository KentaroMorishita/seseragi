function finiteLabel(value: number): string {
  return Number.isFinite(value) ? "finite" : "non-finite"
}

console.log(finiteLabel(12.5))
console.log(finiteLabel(-3))
console.log(finiteLabel(-0))
console.log(finiteLabel(1e100))
console.log(finiteLabel(Number.NaN))
console.log(finiteLabel(Number.POSITIVE_INFINITY))
console.log(finiteLabel(Number.NEGATIVE_INFINITY))

export {}
