// Both arguments are already safe integers; packages is nonzero here.
function perPackage(total: number, packages: number): number {
  return total / packages
}

console.log(`converted: ${5}`)
console.log(`per package: ${perPackage(5, 2)}`)
console.log(`maximum: ${Number.MAX_SAFE_INTEGER}`)

export {}
