export {}
const content = Uint8Array.of(0, 15, 255)
for (const index of [0, 2, 3, -1]) {
  const value = content[index]
  console.log(
    value === undefined ? `Index ${index}: missing` : `Index ${index}: ${value}`
  )
}
