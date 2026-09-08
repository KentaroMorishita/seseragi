// The declared Array boundary supplies an intentional isolated copy.
export function copyProbe(values) {
  values[0] = 99
  return values.length
}
