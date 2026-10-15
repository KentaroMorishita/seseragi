// The launcher installs the checked scalar codec before importing the UI.
export function compute(input: number): number {
  const host = globalThis as typeof globalThis & {
    seseragiCompute?: (input: number) => number
  }
  if (!host.seseragiCompute) throw new Error("WASM compute is not initialized")
  return host.seseragiCompute(input)
}
