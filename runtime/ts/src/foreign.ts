import {
  awaitEffectValue,
  type Effect,
  type EffectContext,
  fail,
  isEffectCancellation,
  mapError,
} from "./effect"

export type RuntimeDiagnosticFrame = Readonly<{
  readonly language: "typescript" | "seseragi" | "interop"
  readonly function: string
  readonly uri: string | null
  readonly range: Readonly<{ readonly start: number; readonly end: number }> | null
  readonly generated: boolean
}>

export type JsErrorPhase =
  | "ModuleLoad"
  | "BindingLookup"
  | "SynchronousThrow"
  | "PromiseRejection"

export type JsError = Readonly<{
  readonly phase: JsErrorPhase
  readonly message: string
  readonly cause: unknown
  readonly hostStack?: string
  readonly observedStack?: string
  readonly observedFrame?: RuntimeDiagnosticFrame
  readonly adapterFrame: Readonly<{
    readonly language: "interop"
    readonly function: string
    readonly generated: true
  }>
}>

export type JsUnknown = unknown
export type JsNullOr<Value> = Value | null
export type JsNullable<Value> = Value | null | undefined
export type JsUndefinedOr<Value> = Value | undefined
export type JsPromise<Value> = PromiseLike<Value>
export type JsObject = object
export type JsNumber = number
export type JsString = string
export type JsNull = null
export type JsUndefined = undefined
export type JsMutableArray<Value> = Array<Value>
export type JsCallback<Arguments, Result> = (
  arguments_: Arguments
) => Result

export type ForeignCodec =
  | "js-unknown"
  | "opaque"
  | "unsupported"
  | "unit"
  | "bool"
  | "char"
  | "string"
  | "int"
  | "float"
  | "bigint"
  | "bytes"
  | "js-object"
  | "js-number"
  | "js-string"
  | "js-null"
  | "js-undefined"
  | Readonly<{ readonly array: ForeignCodec }>
  | Readonly<{ readonly mutableArray: ForeignCodec }>
  | Readonly<{ readonly nullOr: ForeignCodec }>
  | Readonly<{ readonly nullable: ForeignCodec }>
  | Readonly<{ readonly undefinedOr: ForeignCodec }>
  | Readonly<{ readonly promise: ForeignCodec }>
  | Readonly<{ readonly rawCallback: true }>
  | Readonly<{ readonly tuple: ReadonlyArray<ForeignCodec> }>
  | Readonly<{
      readonly callback: Readonly<{
        readonly parameters: ReadonlyArray<ForeignCodec>
        readonly result: ForeignCodec
      }>
    }>
  | Readonly<{
      readonly record: Readonly<
        Record<
          string,
          ForeignCodec | Readonly<{ readonly optional: ForeignCodec }>
        >
      >
    }>

export type ForeignTaskModule = Readonly<{
  readonly load: () => Promise<Record<string, unknown>>
  readonly moduleUrl?: string
}>

export type ForeignPath = string | ReadonlyArray<string>
export type ForeignCallKind = "function" | "constructor" | "method" | "property"

const foreignModuleLoads = new Map<
  string,
  Promise<Record<string, unknown>>
>()

export function createForeignTaskModule(
  load: () => Promise<unknown>,
  moduleUrl?: string
): ForeignTaskModule {
  let memoized: Promise<Record<string, unknown>> | undefined
  return Object.freeze({
    ...(moduleUrl === undefined ? {} : { moduleUrl }),
    load: () => {
      memoized ??=
        moduleUrl === undefined
          ? loadForeignModule(load)
          : memoizedForeignModuleLoad(moduleUrl, load)
      return memoized
    },
  })
}

function loadForeignModule(
  load: () => Promise<unknown>
): Promise<Record<string, unknown>> {
  return Promise.resolve()
    .then(load)
    .then((module) => requireNamespace(module))
}

function memoizedForeignModuleLoad(
  exactIdentity: string,
  load: () => Promise<unknown>
): Promise<Record<string, unknown>> {
  const existing = foreignModuleLoads.get(exactIdentity)
  if (existing !== undefined) return existing
  const pending = loadForeignModule(load)
  foreignModuleLoads.set(exactIdentity, pending)
  return pending
}

export function annotateForeignTask<Environment, Success>(
  effect: Effect<Environment, JsError, Success>,
  frame: RuntimeDiagnosticFrame
): Effect<Environment, JsError, Success> {
  return mapError(
    (error: JsError) => Object.freeze({ ...error, observedFrame: frame }),
    effect
  )
}

export function invokeForeignPure<Result>(
  namespace: unknown,
  member: ForeignPath,
  callKind: ForeignCallKind,
  arguments_: ReadonlyArray<unknown>,
  parameterCodecs: ReadonlyArray<ForeignCodec>,
  returnCodec: ForeignCodec
): Result {
  const label = pathLabel(member)
  const encoded = encodeArguments(arguments_, parameterCodecs)
  let result: unknown
  try {
    const invocation = prepareInvocation(namespace, member, callKind, encoded)
    result = runInvocation(invocation)
  } catch (cause) {
    throw new TypeError(`foreign pure binding ${label} threw`, { cause })
  }
  if (isPromiseLike(result)) {
    throw new TypeError(`foreign pure binding ${label} returned PromiseLike`)
  }
  return decodeValue(result, returnCodec, `return value of ${label}`) as Result
}

export function readForeignPureValue<Value>(
  namespace: unknown,
  member: ForeignPath,
  codec: ForeignCodec
): Value {
  return decodeValue(
    lookup(namespace, member),
    codec,
    `value ${pathLabel(member)}`
  ) as Value
}

export function invokeForeignTask<Success>(
  module: ForeignTaskModule,
  member: ForeignPath,
  callKind: ForeignCallKind,
  arguments_: ReadonlyArray<unknown>,
  parameterCodecs: ReadonlyArray<ForeignCodec>,
  returnCodec: ForeignCodec
): Effect<unknown, JsError, Success> {
  const label = pathLabel(member)
  const observedStack = new Error(`foreign task ${label} observed`).stack
  return async (environment, context) => {
    let namespace: Record<string, unknown>
    try {
      namespace = await awaitTask(module.load(), context)
    } catch (cause) {
      if (isEffectCancellation(cause)) throw cause
      return fail(jsError("ModuleLoad", cause, observedStack))(
        environment,
        context
      )
    }

    let encoded: ReadonlyArray<unknown>
    try {
      encoded = encodeArguments(arguments_, parameterCodecs)
    } catch (cause) {
      return fail(jsError("SynchronousThrow", cause, observedStack))(
        environment,
        context
      )
    }

    let invocation: ForeignInvocation
    try {
      invocation = prepareInvocation(namespace, member, callKind, encoded)
    } catch (cause) {
      return fail(jsError("BindingLookup", cause, observedStack))(
        environment,
        context
      )
    }
    let result: unknown
    try {
      result = runInvocation(invocation)
    } catch (cause) {
      return fail(jsError("SynchronousThrow", cause, observedStack))(
        environment,
        context
      )
    }
    let value = result
    if (isPromiseLike(result)) {
      try {
        value = await awaitTask(Promise.resolve(result), context)
      } catch (cause) {
        if (isEffectCancellation(cause)) throw cause
        return fail(jsError("PromiseRejection", cause, observedStack))(
          environment,
          context
        )
      }
    }
    try {
      return decodeValue(value, returnCodec, `return value of ${label}`) as Success
    } catch (cause) {
      return fail(jsError("SynchronousThrow", cause, observedStack))(
        environment,
        context
      )
    }
  }
}

function awaitTask<Value>(
  value: Promise<Value>,
  context: EffectContext | undefined
): Promise<Value> {
  return context === undefined ? value : awaitEffectValue(value, context)
}

function jsError(
  phase: JsErrorPhase,
  cause: unknown,
  observedStack: string | undefined
): JsError {
  return Object.freeze({
    phase,
    message: cause instanceof Error ? cause.message : String(cause),
    cause,
    ...(cause instanceof Error && cause.stack !== undefined
      ? { hostStack: cause.stack }
      : {}),
    ...(observedStack === undefined ? {} : { observedStack }),
    adapterFrame: Object.freeze({
      language: "interop",
      function: labelFromObservedStack(observedStack),
      generated: true,
    }),
  })
}

function labelFromObservedStack(stack: string | undefined): string {
  const match = stack?.match(/foreign task ([^ ]+) observed/)
  return match?.[1] ?? "<foreign>"
}

export function renderJsErrorDiagnostic(
  value: unknown,
  readSource: (url: string) => string | undefined
): string | undefined {
  if (!isJsError(value)) return undefined
  const thrown = hostDiagnosticFrame(value, readSource)
  const observed = [
    ...(value.observedFrame === undefined ? [] : [value.observedFrame]),
    {
      language: "interop" as const,
      function: value.adapterFrame.function,
      uri: null,
      range: null,
      generated: true,
    },
  ]
  return JSON.stringify({
    schema: 1,
    kind: "TypedFailure",
    phase: value.phase,
    message: value.message,
    groups: [
      { role: "Thrown", frames: thrown === undefined ? [] : [thrown] },
      { role: "Observed", frames: observed },
    ],
  })
}

export function renderTypedFailureDiagnostic(message: string): string {
  return JSON.stringify({
    schema: 1,
    kind: "TypedFailure",
    phase: null,
    message,
    groups: [],
  })
}

export function renderRuntimeDefectDiagnostic(
  value: unknown,
  readSource: (url: string) => string | undefined
): string {
  const frame =
    value instanceof Error
      ? stackDiagnosticFrame(value.stack, readSource)
      : undefined
  return JSON.stringify({
    schema: 1,
    kind: "Defect",
    phase: null,
    message: diagnosticMessage(value),
    groups: [
      { role: "Thrown", frames: frame === undefined ? [] : [frame] },
    ],
  })
}

export function renderCancellationDiagnostic(): string {
  return JSON.stringify({
    schema: 1,
    kind: "Cancellation",
    phase: null,
    message: "execution cancelled",
    groups: [],
  })
}

function diagnosticMessage(value: unknown): string {
  if (value instanceof Error && value.message.length > 0) return value.message
  if (typeof value === "string" && value.length > 0) return value
  try {
    return String(value)
  } catch {
    return "runtime defect"
  }
}

function isJsError(value: unknown): value is JsError {
  return (
    typeof value === "object" &&
    value !== null &&
    "phase" in value &&
    typeof value.phase === "string" &&
    "message" in value &&
    typeof value.message === "string" &&
    "adapterFrame" in value
  )
}

function hostDiagnosticFrame(
  error: JsError,
  readSource: (url: string) => string | undefined
): RuntimeDiagnosticFrame | undefined {
  return stackDiagnosticFrame(error.hostStack, readSource)
}

function stackDiagnosticFrame(
  stack: string | undefined,
  readSource: (url: string) => string | undefined
): RuntimeDiagnosticFrame | undefined {
  const match = stack?.match(
    /\n\s*at\s+([^\s(]+)\s+\((.+):(\d+):(\d+)\)/
  )
  if (match == null) return undefined
  const [, functionName, url, lineText] = match
  if (url === undefined || lineText === undefined) return undefined
  const source = readSource(url)
  if (source === undefined) return undefined
  const line = Number(lineText)
  const lines = source.split("\n")
  const statement = lines[line - 1]
  if (statement === undefined) return undefined
  const leading = statement.length - statement.trimStart().length
  const trimmed = statement.trim().replace(/;$/, "")
  const prefix = lines.slice(0, line - 1).join("\n")
  const lineStart = new TextEncoder().encode(
    line <= 1 ? "" : `${prefix}\n`
  ).length
  const start =
    lineStart + new TextEncoder().encode(statement.slice(0, leading)).length
  const end = start + new TextEncoder().encode(trimmed).length
  return {
    language: "typescript",
    function: functionName ?? "<anonymous>",
    uri: packageUri(url),
    range: { start, end },
    generated: false,
  }
}

function packageUri(url: string): string | null {
  const marker = "/dist/packages/"
  const pathname = url.startsWith("file:") ? new URL(url).pathname : url
  const index = pathname.indexOf(marker)
  return index === -1
    ? null
    : `package://${decodeURIComponent(pathname.slice(index + marker.length))}`
}

function requireNamespace(value: unknown): Record<string, unknown> {
  if (
    (typeof value !== "object" && typeof value !== "function") ||
    value === null
  ) {
    throw new TypeError("foreign module did not evaluate to a namespace")
  }
  return value as Record<string, unknown>
}

function lookup(namespace: unknown, member: ForeignPath): unknown {
  const path = typeof member === "string" ? [member] : member
  let current: unknown = namespace
  for (const segment of path) {
    const object = requireNamespace(current)
    if (!Object.hasOwn(object, segment)) {
      throw new TypeError(
        `foreign binding ${pathLabel(path)} is missing at ${segment}`
      )
    }
    current = object[segment]
  }
  return current
}

function pathLabel(path: ForeignPath): string {
  return (typeof path === "string" ? [path] : path).join(".")
}

type ForeignInvocation =
  | Readonly<{
      readonly kind: "call"
      readonly target: (...arguments_: unknown[]) => unknown
      readonly receiver: unknown
      readonly arguments: ReadonlyArray<unknown>
    }>
  | Readonly<{
      readonly kind: "construct"
      readonly target: new (...arguments_: unknown[]) => unknown
      readonly arguments: ReadonlyArray<unknown>
    }>
  | Readonly<{ readonly kind: "value"; readonly value: unknown }>

function prepareInvocation(
  namespace: unknown,
  path: ForeignPath,
  callKind: ForeignCallKind,
  arguments_: ReadonlyArray<unknown>
): ForeignInvocation {
  const label = pathLabel(path)
  if (callKind === "function" || callKind === "constructor") {
    const target = lookup(namespace, path)
    if (typeof target !== "function") {
      throw new TypeError(
        `foreign ${callKind} binding ${label} is not callable`
      )
    }
    return callKind === "constructor"
      ? {
          kind: "construct",
          target: target as new (...arguments_: unknown[]) => unknown,
          arguments: arguments_,
        }
      : {
          kind: "call",
          target: target as (...arguments_: unknown[]) => unknown,
          receiver: undefined,
          arguments: arguments_,
        }
  }
  if (arguments_.length === 0) {
    throw new TypeError(
      `foreign ${callKind} binding ${label} requires a receiver`
    )
  }
  const [receiver, ...rest] = arguments_
  if (
    (typeof receiver !== "object" && typeof receiver !== "function") ||
    receiver === null
  ) {
    throw new TypeError(
      `foreign ${callKind} receiver ${label} is not an object`
    )
  }
  const key = typeof path === "string" ? path : path.at(-1)
  if (key === undefined) throw new TypeError("foreign member path is empty")
  const target = Reflect.get(receiver, key)
  if (callKind === "property") {
    if (rest.length !== 0) {
      throw new TypeError(
        `foreign property binding ${label} has extra arguments`
      )
    }
    return { kind: "value", value: target }
  }
  if (typeof target !== "function") {
    throw new TypeError(`foreign method binding ${label} is not callable`)
  }
  return { kind: "call", target, receiver, arguments: rest }
}

function runInvocation(invocation: ForeignInvocation): unknown {
  if (invocation.kind === "value") return invocation.value
  if (invocation.kind === "construct") {
    return Reflect.construct(invocation.target, invocation.arguments)
  }
  return Reflect.apply(
    invocation.target,
    invocation.receiver,
    invocation.arguments
  )
}

function encodeArguments(
  values: ReadonlyArray<unknown>,
  codecs: ReadonlyArray<ForeignCodec>
): ReadonlyArray<unknown> {
  if (values.length !== codecs.length) {
    throw new TypeError("foreign binding arity mismatch")
  }
  return values.map((value, index) =>
    encodeValue(value, codecs[index] ?? "unsupported", `argument ${index + 1}`)
  )
}

function encodeValue(
  value: unknown,
  codec: ForeignCodec,
  path: string
): unknown {
  return convertValue(value, codec, path, "encode")
}

function decodeValue(
  value: unknown,
  codec: ForeignCodec,
  path: string
): unknown {
  return convertValue(value, codec, path, "decode")
}

function convertValue(
  value: unknown,
  codec: ForeignCodec,
  path: string,
  direction: "encode" | "decode"
): unknown {
  if (codec === "js-unknown") return value
  if (codec === "unsupported") {
    throw new TypeError(`${path} uses an unsupported foreign boundary type`)
  }
  if (codec === "opaque") {
    if (
      (typeof value !== "object" && typeof value !== "function") ||
      value === null
    ) {
      invalid(path, "opaque host object")
    }
    return value
  }
  if (codec === "unit") {
    if (value !== undefined) invalid(path, "undefined")
    return undefined
  }
  if (codec === "bool") {
    if (typeof value !== "boolean") invalid(path, "boolean")
    return value
  }
  if (codec === "string") {
    if (typeof value !== "string") invalid(path, "string")
    return value
  }
  if (codec === "char") {
    const codePoint = typeof value === "string" ? value.codePointAt(0) : undefined
    if (
      typeof value !== "string" ||
      [...value].length !== 1 ||
      codePoint === undefined ||
      (codePoint >= 0xd800 && codePoint <= 0xdfff)
    ) {
      invalid(path, "one Unicode scalar")
    }
    return value
  }
  if (codec === "int") {
    if (typeof value !== "number" || !Number.isSafeInteger(value)) {
      invalid(path, "safe integer")
    }
    return Object.is(value, -0) ? 0 : value
  }
  if (codec === "float") {
    if (typeof value !== "number" || !Number.isFinite(value)) {
      invalid(path, "finite number")
    }
    return value
  }
  if (codec === "bigint") {
    if (typeof value !== "bigint") invalid(path, "bigint")
    return value
  }
  if (codec === "bytes") {
    if (!(value instanceof Uint8Array)) invalid(path, "Uint8Array")
    return new Uint8Array(value)
  }
  if (codec === "js-object") {
    if (
      (typeof value !== "object" && typeof value !== "function") ||
      value === null
    ) {
      invalid(path, "host object")
    }
    return value
  }
  if (codec === "js-number") {
    if (typeof value !== "number") invalid(path, "number")
    return value
  }
  if (codec === "js-string") {
    if (typeof value !== "string") invalid(path, "string")
    return value
  }
  if (codec === "js-null") {
    if (value !== null) invalid(path, "null")
    return value
  }
  if (codec === "js-undefined") {
    if (value !== undefined) invalid(path, "undefined")
    return value
  }
  if ("nullOr" in codec) {
    return value === null
      ? null
      : convertValue(value, codec.nullOr, path, direction)
  }
  if ("undefinedOr" in codec) {
    return value === undefined
      ? undefined
      : convertValue(value, codec.undefinedOr, path, direction)
  }
  if ("nullable" in codec) {
    return value === null || value === undefined
      ? value
      : convertValue(value, codec.nullable, path, direction)
  }
  if ("promise" in codec) {
    if (!isPromiseLike(value)) invalid(path, "PromiseLike")
    return value
  }
  if ("rawCallback" in codec) {
    if (typeof value !== "function") invalid(path, "function")
    return value
  }
  if ("array" in codec) {
    if (!Array.isArray(value)) invalid(path, "array")
    return value.map((item, index) =>
      convertValue(item, codec.array, `${path}[${index}]`, direction)
    )
  }
  if ("mutableArray" in codec) {
    if (!Array.isArray(value)) invalid(path, "mutable array")
    value.forEach((item, index) => {
      convertValue(item, codec.mutableArray, `${path}[${index}]`, direction)
    })
    return value
  }
  if ("tuple" in codec) {
    if (!Array.isArray(value) || value.length !== codec.tuple.length) {
      invalid(path, `tuple of length ${codec.tuple.length}`)
    }
    return codec.tuple.map((itemCodec, index) =>
      convertValue(value[index], itemCodec, `${path}[${index}]`, direction)
    )
  }
  if ("callback" in codec) {
    if (typeof value !== "function") invalid(path, "function")
    if (direction === "encode") {
      return (...arguments_: ReadonlyArray<unknown>) => {
        const decoded = encodeArguments(arguments_, codec.callback.parameters)
        let result: unknown = value
        for (const argument of decoded) {
          if (typeof result !== "function") invalid(path, "curried function")
          result = Reflect.apply(result, undefined, [argument])
        }
        if (isPromiseLike(result)) {
          throw new TypeError(`${path} returned PromiseLike`)
        }
        return decodeValue(result, codec.callback.result, `${path} result`)
      }
    }
    const host = value
    const apply = (arguments_: ReadonlyArray<unknown>): unknown =>
      arguments_.length === codec.callback.parameters.length
        ? decodeValue(
            Reflect.apply(
              host,
              undefined,
              encodeArguments(arguments_, codec.callback.parameters)
            ),
            codec.callback.result,
            `${path} result`
          )
        : (argument: unknown) => apply([...arguments_, argument])
    return apply([])
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    invalid(path, "record")
  }
  const source = value as Record<string, unknown>
  const copy: Record<string, unknown> = {}
  for (const [key, fieldSpec] of Object.entries(codec.record)) {
    const optional =
      typeof fieldSpec === "object" &&
      fieldSpec !== null &&
      "optional" in fieldSpec
    const fieldCodec = optional ? fieldSpec.optional : fieldSpec
    if (!Object.hasOwn(source, key)) {
      if (optional) continue
      invalid(`${path}.${key}`, "present field")
    }
    copy[key] = convertValue(
      source[key],
      fieldCodec,
      `${path}.${key}`,
      direction
    )
  }
  return Object.freeze(copy)
}

function invalid(path: string, expected: string): never {
  throw new TypeError(`${path} must be ${expected}`)
}

function isPromiseLike(value: unknown): value is PromiseLike<unknown> {
  return (
    (typeof value === "object" || typeof value === "function") &&
    value !== null &&
    typeof (value as { then?: unknown }).then === "function"
  )
}
