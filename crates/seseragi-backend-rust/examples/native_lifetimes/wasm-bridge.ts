// Research scalar ABI. Seseragi Int stays a safe integer at both boundaries.
export class WasmDefect extends Error {
  override name = "WasmDefect"
}

export async function loadCompute(bytes: BufferSource) {
  const module = await WebAssembly.compile(bytes)
  if (WebAssembly.Module.imports(module).length !== 0) {
    throw new TypeError("The scalar research module must have no host imports")
  }
  const instance = await WebAssembly.instantiate(module)
  const compute = instance.exports.compute
  if (typeof compute !== "function")
    throw new TypeError("Missing compute export")
  return scalarCodec(compute)
}

function scalarCodec(compute: CallableFunction | undefined) {
  return (input: number): number => {
    if (!Number.isSafeInteger(input))
      throw new TypeError("Expected a Seseragi Int")
    if (!compute) throw new WasmDefect("WASM instance stopped after a defect")
    let value: unknown
    try {
      value = compute(BigInt(input))
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) {
        // An aborting WASM panic does not unwind Rust captures. Retire the
        // instance and release this codec's owning function reference.
        compute = undefined
        throw new WasmDefect("Seseragi computation defect", { cause: error })
      }
      throw error
    }
    if (typeof value !== "bigint") throw new TypeError("Invalid Int result ABI")
    const result = Number(value)
    if (!Number.isSafeInteger(result) || BigInt(result) !== value) {
      throw new TypeError("Result is outside the Seseragi Int range")
    }
    return result
  }
}
