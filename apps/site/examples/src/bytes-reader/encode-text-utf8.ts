export {}
for (const label of ["A", "café", "🌊", ""]) {
  const content = new TextEncoder().encode(label)
  console.log(`${content.toHex()}:${content.length}`)
}
