// schema and contract are generated from checked Core, not supplied by callers.
export class ProgramWasmDefect extends Error {
  name = "ProgramWasmDefect"
}
export class ProgramWasmClosed extends Error {
  name = "ProgramWasmClosed"
}

const encoder = new TextEncoder()
const negativeZero = Symbol("negative zero")
const decoder = new TextDecoder("utf-8", { fatal: true })

function dataObject(value, keys) {
  if (
    value === null ||
    typeof value !== "object" ||
    ![Object.prototype, null].includes(Object.getPrototypeOf(value))
  ) {
    throw new TypeError("Expected a plain program boundary value")
  }
  const fields = Object.getOwnPropertyDescriptors(value)
  if (
    Reflect.ownKeys(fields).length !== keys.length ||
    keys.some((key) => !fields[key] || !("value" in fields[key]))
  ) {
    throw new TypeError("Invalid program boundary fields")
  }
  return Object.fromEntries(keys.map((key) => [key, fields[key].value]))
}

function write(id, value, writer) {
  const code = schema.codecs[id]
  switch (code.kind) {
    case "unit":
      if (value !== undefined) throw new TypeError("Expected Unit (undefined)")
      break
    case "bool":
      if (typeof value !== "boolean") throw new TypeError("Expected Bool")
      writer.add(Uint8Array.of(value ? 1 : 0))
      break
    case "int": {
      if (!Number.isSafeInteger(value))
        throw new TypeError("Expected a safe Seseragi Int")
      const bytes = new Uint8Array(8)
      new DataView(bytes.buffer).setBigInt64(0, BigInt(value), true)
      writer.add(bytes)
      break
    }
    case "float": {
      if (typeof value !== "number") throw new TypeError("Expected a Float")
      const bytes = new Uint8Array(8)
      new DataView(bytes.buffer).setFloat64(0, value, true)
      writer.add(bytes)
      break
    }
    case "char": {
      const point = typeof value === "string" ? value.codePointAt(0) : undefined
      if (
        point === undefined ||
        String.fromCodePoint(point) !== value ||
        (point >= 0xd800 && point <= 0xdfff)
      ) {
        throw new TypeError("Expected a Char (one Unicode scalar value)")
      }
      writer.u32(point)
      break
    }
    case "string": {
      if (typeof value !== "string" || value.length > schema.maxBoundaryBytes) {
        throw new TypeError("Expected a bounded String")
      }
      for (let index = 0; index < value.length; index++) {
        const point = value.charCodeAt(index)
        if (point >= 0xd800 && point <= 0xdbff) {
          const next = value.charCodeAt(++index)
          if (!(next >= 0xdc00 && next <= 0xdfff))
            throw new TypeError("Unpaired String surrogate")
        } else if (point >= 0xdc00 && point <= 0xdfff) {
          throw new TypeError("Unpaired String surrogate")
        }
      }
      const bytes = encoder.encode(value)
      writer.u32(bytes.length)
      writer.add(bytes)
      break
    }
    case "tuple": {
      if (
        !Array.isArray(value) ||
        Object.getPrototypeOf(value) !== Array.prototype
      ) {
        throw new TypeError("Expected a tuple array")
      }
      const fields = Object.getOwnPropertyDescriptors(value)
      if (
        fields.length?.value !== code.elements.length ||
        Reflect.ownKeys(fields).length !== code.elements.length + 1
      ) {
        throw new TypeError("Invalid tuple arity")
      }
      code.elements.forEach((element, index) => {
        if (!fields[index] || !("value" in fields[index]))
          throw new TypeError("Invalid tuple field")
        write(element, fields[index].value, writer)
      })
      break
    }
    case "maybe": {
      const fields = Object.getOwnPropertyDescriptors(value ?? {})
      const tag = fields.tag?.value
      const data = dataObject(
        value,
        tag === "Nothing" ? ["tag"] : ["tag", "value"]
      )
      if (tag === "Nothing") writer.add(Uint8Array.of(0))
      else if (tag === "Just") {
        writer.add(Uint8Array.of(1))
        write(code.value, data.value, writer)
      } else throw new TypeError("Expected Maybe variant")
      break
    }
    case "either": {
      const data = dataObject(value, ["tag", "value"])
      if (data.tag !== "Left" && data.tag !== "Right")
        throw new TypeError("Expected Either variant")
      writer.add(Uint8Array.of(data.tag === "Left" ? 0 : 1))
      write(data.tag === "Left" ? code.left : code.right, data.value, writer)
      break
    }
    case "array": {
      if (
        !Array.isArray(value) ||
        Object.getPrototypeOf(value) !== Array.prototype
      ) {
        throw new TypeError("Expected an Array")
      }
      const fields = Object.getOwnPropertyDescriptors(value)
      const length = fields.length?.value
      if (
        !Number.isSafeInteger(length) ||
        Reflect.ownKeys(fields).length !== length + 1
      ) {
        throw new TypeError("Invalid Array elements")
      }
      if (length > schema.maxSequenceItems)
        throw new TypeError("Array exceeds the program boundary length limit")
      writer.u32(length)
      for (let index = 0; index < length; index++) {
        if (!fields[index] || !("value" in fields[index]))
          throw new TypeError("Invalid Array element")
        write(code.element, fields[index].value, writer)
      }
      break
    }
    case "list": {
      // Read the whole List before writing so a cyclic object is rejected
      // instead of looping, and each cell is checked like the TS ABI.
      const values = []
      const seen = new Set()
      let cursor = value
      for (;;) {
        const tag = Object.getOwnPropertyDescriptors(cursor ?? {}).tag?.value
        if (tag === "Empty") {
          dataObject(cursor, ["tag"])
          break
        }
        if (tag !== "Cons") throw new TypeError("Expected List variant")
        if (seen.has(cursor)) throw new TypeError("Cyclic List")
        seen.add(cursor)
        const cell = dataObject(cursor, ["tag", "head", "tail"])
        values.push(cell.head)
        if (values.length > schema.maxSequenceItems)
          throw new TypeError("List exceeds the program boundary length limit")
        cursor = cell.tail
      }
      writer.u32(values.length)
      for (const item of values) write(code.element, item, writer)
      break
    }
    case "record": {
      const data = dataObject(
        value,
        code.fields.map((field) => field.name)
      )
      for (const field of code.fields)
        write(field.codec, data[field.name], writer)
      break
    }
    case "adt": {
      const tag = Object.getOwnPropertyDescriptors(value ?? {}).tag?.value
      const index = code.variants.findIndex((variant) => variant.tag === tag)
      if (index < 0) throw new TypeError("Expected a checked variant")
      const variant = code.variants[index]
      const data = dataObject(
        value,
        variant.payload === null ? ["tag"] : ["tag", "value"]
      )
      writer.u32(index)
      if (variant.payload !== null) write(variant.payload, data.value, writer)
      break
    }
    case "parameter": {
      // WASM only holds and returns a type parameter's value, so the host
      // value itself stays here and crosses by token.
      const state = writer.state
      if (!state) throw new TypeError("A host value needs its program")
      // A Map key treats -0 as +0, so negative zero has its own key.
      const key = Object.is(value, -0) ? negativeZero : value
      let token = state.hostTokens.get(key)
      if (token === undefined) {
        if (state.nextHostValue > 0xffffffff)
          throw new TypeError("Host value token space exhausted")
        token = state.nextHostValue++
        state.hostValues.set(token, value)
        state.hostTokens.set(key, token)
      }
      writer.u32(token)
      break
    }
    default:
      throw new TypeError(
        "Host function arguments are outside this program ABI"
      )
  }
}

function writer(state) {
  const chunks = []
  let size = 0
  return {
    state,
    add(bytes) {
      size += bytes.length
      if (size > schema.maxBoundaryBytes)
        throw new TypeError("Program boundary byte limit exceeded")
      chunks.push(bytes)
    },
    u32(value) {
      const bytes = new Uint8Array(4)
      new DataView(bytes.buffer).setUint32(0, value, true)
      this.add(bytes)
    },
    bytes() {
      const bytes = new Uint8Array(size)
      let offset = 0
      for (const chunk of chunks) {
        bytes.set(chunk, offset)
        offset += chunk.length
      }
      return bytes
    },
  }
}

function reader(bytes) {
  let offset = 0
  return {
    take(length) {
      if (
        !Number.isSafeInteger(length) ||
        length < 0 ||
        offset + length > bytes.length
      ) {
        throw new TypeError("Truncated program ABI result")
      }
      const result = bytes.subarray(offset, offset + length)
      offset += length
      return result
    },
    u32() {
      const value = this.take(4)
      return new DataView(value.buffer, value.byteOffset, 4).getUint32(0, true)
    },
    end() {
      if (offset !== bytes.length)
        throw new TypeError("Trailing program ABI result")
    },
  }
}

function sequenceLength(input) {
  const count = input.u32()
  if (count > schema.maxSequenceItems)
    throw new TypeError("Invalid sequence length result")
  return count
}

// Adds a source location to an outcome without making it part of the
// outcome's enumerable shape, so outcomes still compare as before.
function located(outcome, key, value) {
  if (value !== null)
    Object.defineProperty(outcome, key, { value, enumerable: false })
  return outcome
}

// A typed failure's or Defect's source location, from `file:line:column`.
function sourceLocation(text) {
  const place = /^(.*):(\d+):(\d+)$/.exec(text)
  return place
    ? Object.freeze({
        file: place[1],
        line: Number(place[2]),
        column: Number(place[3]),
      })
    : null
}

function read(id, input, state, provenance) {
  const code = schema.codecs[id]
  switch (code.kind) {
    case "unit":
      return undefined
    case "bool": {
      const value = input.take(1)[0]
      if (value > 1) throw new TypeError("Invalid Bool result")
      return value === 1
    }
    case "int": {
      const bytes = input.take(8)
      const big = new DataView(bytes.buffer, bytes.byteOffset, 8).getBigInt64(
        0,
        true
      )
      const value = Number(big)
      if (!Number.isSafeInteger(value) || BigInt(value) !== big)
        throw new TypeError("Invalid Int result")
      return value
    }
    case "float": {
      const bytes = input.take(8)
      return new DataView(bytes.buffer, bytes.byteOffset, 8).getFloat64(0, true)
    }
    case "char": {
      const point = input.u32()
      if (point > 0x10ffff || (point >= 0xd800 && point <= 0xdfff))
        throw new TypeError("Invalid Char result")
      return String.fromCodePoint(point)
    }
    case "string":
      return decoder.decode(input.take(input.u32()))
    case "tuple":
      return Object.freeze(
        code.elements.map((element) => read(element, input, state, provenance))
      )
    case "maybe": {
      const tag = input.take(1)[0]
      if (tag === 0) return Object.freeze({ tag: "Nothing" })
      if (tag === 1)
        return Object.freeze({
          tag: "Just",
          value: read(code.value, input, state, provenance),
        })
      throw new TypeError("Invalid Maybe result")
    }
    case "either": {
      const tag = input.take(1)[0]
      if (tag > 1) throw new TypeError("Invalid Either result")
      return Object.freeze({
        tag: tag === 0 ? "Left" : "Right",
        value: read(
          tag === 0 ? code.left : code.right,
          input,
          state,
          provenance
        ),
      })
    }
    case "array": {
      const count = sequenceLength(input)
      const values = []
      for (let index = 0; index < count; index++)
        values.push(read(code.element, input, state, provenance))
      return Object.freeze(values)
    }
    case "list": {
      const count = sequenceLength(input)
      const values = []
      for (let index = 0; index < count; index++)
        values.push(read(code.element, input, state, provenance))
      let list = Object.freeze({ tag: "Empty" })
      for (let index = values.length - 1; index >= 0; index--)
        list = Object.freeze({ tag: "Cons", head: values[index], tail: list })
      return list
    }
    case "record":
      return Object.freeze(
        Object.fromEntries(
          code.fields.map((field) => [
            field.name,
            read(field.codec, input, state, provenance),
          ])
        )
      )
    case "adt": {
      const variant = code.variants[input.u32()]
      if (!variant) throw new TypeError("Invalid variant result")
      return Object.freeze(
        variant.payload === null
          ? { tag: variant.tag }
          : {
              tag: variant.tag,
              value: read(variant.payload, input, state, provenance),
            }
      )
    }
    case "parameter": {
      const token = input.u32()
      if (!state.hostValues.has(token))
        throw new TypeError("Unknown host value result")
      return state.hostValues.get(token)
    }
    case "function":
      return callback(state, input.u32(), id, provenance)
    case "effect":
      return effectPlan(state, input.u32(), id, provenance)
    default:
      throw new TypeError("Unknown checked program codec")
  }
}

function pointer(value) {
  // The raw WASM memory ABI is a u32 bit pattern, not a Seseragi Int codec.
  if (!Number.isInteger(value) || value < -0x80000000 || value > 0xffffffff) {
    throw new TypeError("Invalid WASM pointer")
  }
  return value >>> 0
}
function output(api) {
  const start = pointer(api.ssrg_output_ptr())
  const length = pointer(api.ssrg_output_len())
  if (
    length > schema.maxBoundaryBytes ||
    start + length > api.memory.buffer.byteLength
  ) {
    throw new TypeError("Invalid program result buffer")
  }
  return new Uint8Array(api.memory.buffer, start, length).slice()
}
function requireActive(state) {
  if (!state.api)
    throw new ProgramWasmClosed(
      state.retired
        ? "WASM instance retired after a trap"
        : "WASM instance disposed"
    )
  if (state.closing) throw new ProgramWasmClosed("WASM instance is closing")
  if (state.busy)
    throw new ProgramWasmClosed(
      "Reentrant program WASM calls require another ABI"
    )
}
function releaseHostRoots(state, retired) {
  for (const entry of state.callbacks.values()) entry.closed = true
  state.callbacks.clear()
  state.hostValues.clear()
  state.hostTokens.clear()
  for (const entry of state.effects.values()) entry.closed = true
  state.effects.clear()
  state.api = undefined
  state.retired ||= retired
}
function trapped(state, error, provenance) {
  releaseHostRoots(state, true)
  const defect = new ProgramWasmDefect(
    `Program WASM trapped at ${provenance.symbol}`,
    {
      cause: Object.freeze({
        name: error.name,
        message: error.message,
        stack: String(error.stack),
      }),
    }
  )
  defect.origin = provenance.origin
  return defect
}

function callback(state, token, codec, provenance) {
  if (token === 0 || state.callbacks.has(token) || state.effects.has(token))
    throw new TypeError("Invalid/reused callback token")
  const entry = { closed: false }
  const invoke = (argument) => {
    requireActive(state)
    if (entry.closed) throw new ProgramWasmClosed("WASM callback detached")
    state.busy = true
    let phase = "input"
    try {
      const encoded = writer(state)
      write(schema.codecs[codec].parameter, argument, encoded)
      const bytes = encoded.bytes()
      const api = state.api
      phase = "wasm"
      const start = pointer(api.ssrg_input(bytes.length))
      phase = "result"
      if (start === 0 || start + bytes.length > api.memory.buffer.byteLength)
        throw new TypeError("Invalid program input buffer")
      new Uint8Array(api.memory.buffer, start, bytes.length).set(bytes)
      phase = "wasm"
      const status = api.ssrg_call(token)
      phase = "result"
      const bytesOut = output(api)
      if (status !== 0) throw new TypeError(decoder.decode(bytesOut))
      const decoded = reader(bytesOut)
      const result = read(
        schema.codecs[codec].result,
        decoded,
        state,
        provenance
      )
      decoded.end()
      return result
    } catch (error) {
      if (phase === "wasm" || error instanceof WebAssembly.RuntimeError)
        throw trapped(state, error, provenance)
      // A decoder/protocol failure can leave an opaque result root. Retire
      // instead of exposing a partially decoded callable or reusing the VM.
      if (phase === "result") releaseHostRoots(state, true)
      throw error
    } finally {
      state.busy = false
    }
  }
  Object.defineProperties(invoke, {
    disposed: { get: () => entry.closed },
    dispose: {
      value: () => {
        if (entry.closed) return
        requireActive(state)
        try {
          state.api.ssrg_detach(token)
        } catch (error) {
          throw trapped(state, error, provenance)
        }
        entry.closed = true
        state.callbacks.delete(token)
      },
    },
  })
  state.callbacks.set(token, entry)
  return Object.freeze(invoke)
}

function effectPlan(state, token, codec, provenance) {
  if (token === 0 || state.callbacks.has(token) || state.effects.has(token))
    throw new TypeError("Invalid/reused Effect token")
  const entry = { closed: false }
  const plan = Object.freeze({
    get disposed() {
      return entry.closed
    },
    run() {
      requireActive(state)
      if (entry.closed) throw new ProgramWasmClosed("WASM Effect detached")
      if (state.running)
        throw new ProgramWasmClosed("Only one root Effect may run per instance")
      try {
        if (state.api.ssrg_start(token) !== 0)
          throw new TypeError(decoder.decode(output(state.api)))
      } catch (error) {
        throw trapped(state, error, provenance)
      }
      const execution = { done: false }
      state.running = execution
      const result = pumpEffect(state, execution, codec, provenance)
      execution.result = result
      return Object.freeze({
        result,
        cancel() {
          if (execution.done || state.running !== execution || !state.api)
            return
          try {
            if (state.api.ssrg_cancel() !== 0)
              throw new TypeError("Program lost its active Effect")
          } catch (error) {
            throw trapped(state, error, provenance)
          }
          execution.wake?.()
        },
      })
    },
    dispose() {
      if (entry.closed) return
      requireActive(state)
      try {
        state.api.ssrg_detach(token)
      } catch (error) {
        throw trapped(state, error, provenance)
      }
      entry.closed = true
      state.effects.delete(token)
    },
  })
  state.effects.set(token, entry)
  return plan
}

async function pumpEffect(state, execution, codec, provenance) {
  let phase = "wait"
  try {
    while (true) {
      // A task turn, rather than a microtask-only loop, lets UI/timers request
      // cancellation while the same WASM continuation retains its scopes.
      await new Promise((resolve) => setTimeout(resolve, 0))
      if (!state.api) throw new ProgramWasmClosed("WASM instance retired")
      phase = "wasm"
      const status = state.api.ssrg_poll(64)
      phase = "result"
      const bytes = output(state.api)
      if (status !== 0) throw new TypeError(decoder.decode(bytes))
      const input = reader(bytes)
      const tag = input.take(1)[0]
      let outcome
      if (tag === 0) {
        input.end()
        phase = "wait"
        continue
      }
      if (tag === 5) {
        // Every Fiber waits and nothing in this ABI can wake one, as when a
        // TS Effect awaits a Deferred nobody completes. Stop polling until
        // a cancellation can move the root Fiber again.
        input.end()
        phase = "wait"
        await new Promise((resolve) => {
          execution.wake = resolve
        })
        execution.wake = undefined
        continue
      }
      const code = schema.codecs[codec]
      if (tag === 1 || tag === 2) {
        const terminal = tag === 1 ? code.success : code.failure
        if (terminal === null)
          throw new TypeError("Program produced a value of Never")
        const location =
          tag === 2
            ? sourceLocation(decoder.decode(input.take(input.u32())))
            : null
        const value = read(terminal, input, state, provenance)
        outcome =
          tag === 1
            ? { tag: "Success", value }
            : located({ tag: "Failure", error: value }, "location", location)
      } else if (tag === 3) {
        const count = input.u32()
        if (count > bytes.length / 4)
          throw new TypeError("Invalid Defect message count")
        const messages = []
        const locations = []
        for (let index = 0; index < count; index++) {
          locations.push(
            sourceLocation(decoder.decode(input.take(input.u32())))
          )
          messages.push(decoder.decode(input.take(input.u32())))
        }
        outcome = located(
          { tag: "Defect", messages: Object.freeze(messages) },
          "locations",
          Object.freeze(locations)
        )
      } else if (tag === 4) outcome = { tag: "Cancelled" }
      else throw new TypeError("Invalid Effect outcome")
      input.end()
      return Object.freeze(outcome)
    }
  } catch (error) {
    if (phase === "wasm" || error instanceof WebAssembly.RuntimeError)
      throw trapped(state, error, provenance)
    if (phase === "result" || phase === "wait") releaseHostRoots(state, true)
    throw error
  } finally {
    execution.done = true
    if (state.running === execution) state.running = undefined
  }
}

function disposeProgram(state) {
  if (state.closeResult) return state.closeResult
  if (!state.api) return Promise.resolve()
  requireActive(state)
  state.closing = true
  state.closeResult = (async () => {
    let failure
    const running = state.running
    if (running) {
      try {
        if (state.api.ssrg_cancel() !== 0)
          throw new TypeError("Program lost its active Effect")
      } catch (error) {
        throw trapped(state, error, { symbol: `${schema.module}::<cancel>` })
      }
      running.wake?.()
      // Set below by the public execution owner. Both waiters observe the
      // committed terminal outcome only after all masked finalizers finish.
      try {
        const outcome = await running.result
        if (outcome.tag === "Defect")
          failure = new AggregateError(
            outcome.messages.map((message) => new Error(message)),
            "Program WASM Effect ended with a Defect during disposal"
          )
      } catch (error) {
        failure = error
      }
    }
    if (!state.api) {
      if (failure) throw failure
      return
    }
    try {
      const live = state.api.ssrg_dispose()
      state.managedLiveAtClose = live
      if (
        live !== 0 ||
        state.api.ssrg_callbacks() !== 0 ||
        state.api.ssrg_runs() !== 0
      )
        throw new Error("Program WASM managed roots survived normal disposal")
    } catch (error) {
      throw trapped(state, error, { symbol: `${schema.module}::<dispose>` })
    }
    releaseHostRoots(state, false)
    if (failure) throw failure
  })()
  return state.closeResult
}

function programOwner(state, exports) {
  // These closures capture only the mutable owner and JS wrappers. Do not
  // construct them in the loader frame that holds a raw instance/exports.
  return Object.freeze({
    exports: Object.freeze(exports),
    dispose: () => disposeProgram(state),
    ownership: () =>
      Object.freeze({
        alive: Boolean(state.api),
        retired: state.retired,
        callbacks: state.callbacks.size,
        plans: state.effects.size,
        running: Boolean(state.running),
        closing: state.closing,
        managedLiveAtClose: state.managedLiveAtClose,
      }),
  })
}

export async function loadProgram(bytes) {
  const module = await WebAssembly.compile(bytes)
  if (WebAssembly.Module.imports(module).length !== 0)
    throw new TypeError(
      "This providerless Effect program ABI has no host imports"
    )
  let instance
  try {
    instance = await WebAssembly.instantiate(module, {})
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) {
      throw new ProgramWasmDefect("Program WASM trapped during instantiation", {
        cause: Object.freeze({
          name: error.name,
          message: error.message,
          stack: String(error.stack),
        }),
      })
    }
    throw error
  }
  const api = instance.exports
  if (!(api.memory instanceof WebAssembly.Memory))
    throw new TypeError("Missing program memory")
  for (const name of [
    "ssrg_contract_ptr",
    "ssrg_contract_len",
    "ssrg_hash_seed",
    "ssrg_init",
    "ssrg_export",
    "ssrg_input",
    "ssrg_call",
    "ssrg_detach",
    "ssrg_output_ptr",
    "ssrg_output_len",
    "ssrg_dispose",
    "ssrg_live",
    "ssrg_callbacks",
    "ssrg_start",
    "ssrg_cancel",
    "ssrg_poll",
    "ssrg_runs",
  ]) {
    if (typeof api[name] !== "function")
      throw new TypeError(`Missing program export ${name}`)
  }
  const state = {
    api,
    busy: false,
    retired: false,
    callbacks: new Map(),
    hostValues: new Map(),
    hostTokens: new Map(),
    nextHostValue: 1,
    effects: new Map(),
    running: undefined,
    closing: false,
    closeResult: undefined,
    managedLiveAtClose: null,
  }
  let executing = false
  try {
    const start = pointer(api.ssrg_contract_ptr())
    const length = pointer(api.ssrg_contract_len())
    if (
      length > schema.maxBoundaryBytes ||
      start + length > api.memory.buffer.byteLength ||
      decoder.decode(new Uint8Array(api.memory.buffer, start, length)) !==
        contract
    ) {
      throw new TypeError(
        "Program WASM checked contract does not match this loader"
      )
    }
    executing = true
    // Map and Set hash with a secret per-instance seed, as native does
    // with process entropy; WASM has no entropy source of its own.
    const seed = globalThis.crypto.getRandomValues(new Uint32Array(2))
    api.ssrg_hash_seed(seed[0], seed[1])
    const initialized = api.ssrg_init()
    executing = false
    if (initialized !== 0) throw new TypeError(decoder.decode(output(api)))
    const exports = Object.create(null)
    for (let index = 0; index < schema.exports.length; index++) {
      const item = schema.exports[index]
      executing = true
      const selected = api.ssrg_export(index)
      executing = false
      if (selected !== 0) throw new TypeError(decoder.decode(output(api)))
      const response = reader(output(api))
      exports[item.name] = callback(state, response.u32(), item.codec, item)
      response.end()
    }
    return programOwner(state, exports)
  } catch (error) {
    if (executing || error instanceof WebAssembly.RuntimeError) {
      releaseHostRoots(state, true)
      throw new ProgramWasmDefect(
        "Program WASM trapped during initialization",
        {
          cause: Object.freeze({
            name: error.name,
            message: error.message,
            stack: String(error.stack),
          }),
        }
      )
    }
    // An initialized VM with a protocol failure is retired. No promise is made
    // about unwinding a foreign/trapping module supplied in place of its bytes.
    releaseHostRoots(state, true)
    throw error
  }
}
