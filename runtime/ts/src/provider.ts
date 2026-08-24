import {
  EffectCancellation,
  type EffectContext,
  throwIfCancelled,
} from "./effect"

export const providerRuntimeAbi = Object.freeze({
  identity: "seseragi/provider-abi/typescript",
  backend: "typescript",
  abiMajor: 1,
} as const)

export type ProviderRuntimeAbi = typeof providerRuntimeAbi

export type ProviderPrimitiveName =
  | "bool"
  | "bytes"
  | "float"
  | "int"
  | "string"

export type ProviderLogicalType =
  | Readonly<{ kind: "unit" }>
  | Readonly<{ kind: "never" }>
  | Readonly<{ kind: "primitive"; name: ProviderPrimitiveName }>
  | Readonly<{ kind: "named"; identity: string }>
  | Readonly<{ kind: "array"; items: ProviderLogicalType }>
  | Readonly<{
      kind: "record"
      fields: ReadonlyArray<
        Readonly<{ name: string; type: ProviderLogicalType }>
      >
    }>

export type ProviderOperationContract = Readonly<{
  identity: string
  kind: "one-shot" | "resource" | "subscription"
  input: ProviderLogicalType
  success: ProviderLogicalType
  failure: ProviderLogicalType
}>

export type ProviderNamedCodec = Readonly<{
  identity: string
  encode: (value: unknown) => unknown
  decode: (value: unknown) => unknown
}>

export class ProviderCodecRegistry {
  readonly #codecs: ReadonlyMap<string, ProviderNamedCodec>

  constructor(codecs: Iterable<ProviderNamedCodec> = []) {
    const indexed = new Map<string, ProviderNamedCodec>()
    for (const codec of codecs) {
      if (codec.identity.length === 0) {
        throw new TypeError("provider named codec identity must not be empty")
      }
      if (indexed.has(codec.identity)) {
        throw new TypeError(
          `provider named codec is duplicated: ${codec.identity}`
        )
      }
      indexed.set(codec.identity, Object.freeze({ ...codec }))
    }
    this.#codecs = indexed
  }

  codec(identity: string): ProviderNamedCodec {
    const codec = this.#codecs.get(identity)
    if (codec === undefined) {
      throw new TypeError(`provider named codec is not registered: ${identity}`)
    }
    return codec
  }
}

export function assertProviderRuntimeAbi(value: unknown): ProviderRuntimeAbi {
  const record = closedRecord(value, ["abiMajor", "backend", "identity"])
  if (
    record.identity !== providerRuntimeAbi.identity ||
    record.backend !== providerRuntimeAbi.backend ||
    record.abiMajor !== providerRuntimeAbi.abiMajor
  ) {
    throw new TypeError(
      "provider runtime ABI must be seseragi/provider-abi/typescript major 1"
    )
  }
  return providerRuntimeAbi
}

export type ProviderValueOwner = Readonly<{
  provider: string
  service: string
}>

export function encodeProviderValue(
  logicalType: ProviderLogicalType,
  value: unknown,
  codecs: ProviderCodecRegistry,
  owner?: ProviderValueOwner
): unknown {
  return projectValue("encode", logicalType, value, codecs, owner)
}

export function decodeProviderValue(
  logicalType: ProviderLogicalType,
  value: unknown,
  codecs: ProviderCodecRegistry
): unknown {
  return projectValue("decode", logicalType, value, codecs)
}

export type ProviderSuccess = Readonly<{
  kind: "success"
  value: unknown
}>

export type ProviderFailure = Readonly<{
  kind: "failure"
  failure: unknown
}>

export type ProviderResult = ProviderSuccess | ProviderFailure

export type ProviderOperationMember = (
  ...arguments_: ReadonlyArray<unknown>
) => unknown

export type ProviderSubscriptionObserver = Readonly<{
  next: (value: unknown) => void
  complete: () => void
  failure: (failure: unknown) => void
  defect: (cause: unknown) => void
}>

export type ProviderSubscriptionRegistration = Readonly<{
  demand: (count: number) => void | Promise<void>
  unsubscribe: () => void | Promise<void>
}>

export type ProviderSubscriptionMember = (
  input: unknown,
  observer: ProviderSubscriptionObserver,
  attachment?: unknown
) =>
  | ProviderSubscriptionRegistration
  | Promise<ProviderSubscriptionRegistration>

export type ProviderCancellation = () => void | Promise<void>

export type ProviderEntry = Readonly<Record<string, ProviderOperationMember>>

export type ProviderDefect = Readonly<{
  kind: "defect"
  defect: ProviderBoundaryDefect
}>

export type ProviderBridgeOutcome = ProviderResult | ProviderDefect

export type ProviderBoundaryStage = "input" | "call" | "result"

export type ProviderInvocationSource = Readonly<{
  path: string
  start: number
  end: number
}>

export type ProviderBoundaryFrame =
  | Readonly<{
      kind: "seseragi"
      path: string
      start: number
      end: number
    }>
  | Readonly<{ kind: "host"; stack: string }>

export class ProviderBoundaryDefect extends Error {
  readonly provider: string
  readonly service: string
  readonly operation: string
  readonly stage: ProviderBoundaryStage
  readonly frames: ReadonlyArray<ProviderBoundaryFrame>
  override readonly cause: unknown

  constructor(
    provider: string,
    service: string,
    operation: string,
    stage: ProviderBoundaryStage,
    message: string,
    cause: unknown,
    source?: ProviderInvocationSource
  ) {
    super(message)
    this.name = "ProviderBoundaryDefect"
    this.provider = provider
    this.service = service
    this.operation = operation
    this.stage = stage
    this.frames = boundaryFrames(source, cause)
    this.cause = cause
  }
}

export type ProviderInvocation = Readonly<{
  provider: string
  service: string
  operation: ProviderOperationContract
  entry: unknown
  input: unknown
  codecs: ProviderCodecRegistry
  source?: ProviderInvocationSource
  context?: EffectContext
}>

export type ProviderSubscriptionInvocation = ProviderInvocation &
  Readonly<{ attachment?: unknown }>

export type ProviderSubscriptionSource = Readonly<{
  pull: (context: EffectContext) => Promise<IteratorResult<unknown>>
  close: () => Promise<void>
}>

const providerCancellation = Symbol.for(
  "seseragi.provider-operation.cancellation.v1"
)

/** Attaches one cooperative cancellation hook without changing ABI arguments. */
export function withProviderCancellation(
  completion: Promise<ProviderResult>,
  cancel: ProviderCancellation
): Promise<ProviderResult> {
  if (!(completion instanceof Promise)) {
    throw new TypeError("provider completion must be a Promise")
  }
  if (typeof cancel !== "function") {
    throw new TypeError("provider cancellation must be a function")
  }
  Object.defineProperty(completion, providerCancellation, {
    enumerable: false,
    value: cancel,
  })
  return completion
}

export async function invokeProviderOperation(
  invocation: ProviderInvocation
): Promise<ProviderBridgeOutcome> {
  const { provider, service, operation, codecs } = invocation
  if (invocation.context !== undefined) {
    throwIfCancelled(invocation.context)
  }
  if (operation.kind === "subscription") {
    return defect(
      provider,
      service,
      operation.identity,
      "call",
      new TypeError("provider subscription requires the stream bridge"),
      invocation.source
    )
  }
  const owner = { provider, service }
  let input: unknown
  try {
    input = encodeProviderValue(
      operation.input,
      invocation.input,
      codecs,
      owner
    )
  } catch (cause) {
    return defect(
      provider,
      service,
      operation.identity,
      "input",
      cause,
      invocation.source
    )
  }

  let completion: Promise<unknown>
  let unregisterCancellation = (): void => undefined
  try {
    const member = operationMember(invocation.entry, operation.identity)
    const returned = Reflect.apply(member, invocation.entry, [input])
    if (!(returned instanceof Promise)) {
      throw new TypeError("provider operation must return a Promise")
    }
    completion = returned
    unregisterCancellation = registerCancellation(returned, invocation.context)
  } catch (cause) {
    return defect(
      provider,
      service,
      operation.identity,
      "call",
      cause,
      invocation.source
    )
  }

  let result: unknown
  try {
    result = await completion
  } catch (cause) {
    if (invocation.context?.signal.aborted) {
      throw new EffectCancellation()
    }
    return defect(
      provider,
      service,
      operation.identity,
      "call",
      cause,
      invocation.source
    )
  } finally {
    unregisterCancellation()
  }

  try {
    return decodeResult(result, operation, codecs, owner)
  } catch (cause) {
    return defect(
      provider,
      service,
      operation.identity,
      "result",
      cause,
      invocation.source
    )
  }
}

/**
 * Opens a demand-driven Provider subscription. The observer is armed before
 * host registration starts, registration-time callbacks are linearized, and
 * every close path invokes the host unsubscribe effect at most once.
 */
export function openProviderSubscription(
  invocation: ProviderSubscriptionInvocation
): ProviderSubscriptionSource {
  const { provider, service, operation, codecs } = invocation
  if (operation.kind !== "subscription") {
    throw new TypeError(
      "provider stream bridge requires a subscription operation"
    )
  }
  if (invocation.context !== undefined) throwIfCancelled(invocation.context)

  const owner = { provider, service }
  let encodedInput: unknown
  try {
    encodedInput = encodeProviderValue(
      operation.input,
      invocation.input,
      codecs,
      owner
    )
  } catch (cause) {
    throw boundaryDefect(
      provider,
      service,
      operation.identity,
      "input",
      cause,
      invocation.source
    )
  }

  type Terminal =
    | Readonly<{ kind: "complete" }>
    | Readonly<{ kind: "failure"; error: unknown }>
    | Readonly<{ kind: "defect"; error: ProviderBoundaryDefect }>

  const queue: unknown[] = []
  let terminal: Terminal | undefined
  let outstandingDemand = 0
  let pendingPull:
    | Readonly<{
        resolve: (result: IteratorResult<unknown>) => void
        reject: (cause: unknown) => void
      }>
    | undefined
  let registration: ProviderSubscriptionRegistration | undefined
  let closing = false
  let unsubscribed = false

  const settle = (): void => {
    const waiter = pendingPull
    if (waiter === undefined) return
    if (queue.length > 0) {
      pendingPull = undefined
      waiter.resolve({ done: false, value: queue.shift() })
      return
    }
    if (terminal === undefined) return
    pendingPull = undefined
    if (terminal.kind === "complete") {
      waiter.resolve({ done: true, value: undefined })
    } else {
      waiter.reject(terminal.error)
    }
  }

  const terminate = (next: Terminal): void => {
    if (terminal !== undefined || closing) return
    terminal = next
    outstandingDemand = 0
    if (next.kind !== "complete") queue.length = 0
    settle()
  }

  const observer: ProviderSubscriptionObserver = Object.freeze({
    next(value: unknown) {
      if (terminal !== undefined || closing) return
      if (outstandingDemand <= 0) {
        terminate({
          kind: "defect",
          error: boundaryDefect(
            provider,
            service,
            operation.identity,
            "result",
            new TypeError("provider subscription emitted without demand"),
            invocation.source
          ),
        })
        return
      }
      outstandingDemand -= 1
      try {
        queue.push(decodeProviderValue(operation.success, value, codecs))
      } catch (cause) {
        terminate({
          kind: "defect",
          error: boundaryDefect(
            provider,
            service,
            operation.identity,
            "result",
            cause,
            invocation.source
          ),
        })
        return
      }
      settle()
    },
    complete() {
      terminate({ kind: "complete" })
    },
    failure(value: unknown) {
      if (terminal !== undefined || closing) return
      try {
        terminate({
          kind: "failure",
          error: decodeProviderValue(operation.failure, value, codecs),
        })
      } catch (cause) {
        terminate({
          kind: "defect",
          error: boundaryDefect(
            provider,
            service,
            operation.identity,
            "result",
            cause,
            invocation.source
          ),
        })
      }
    },
    defect(cause: unknown) {
      terminate({
        kind: "defect",
        error: boundaryDefect(
          provider,
          service,
          operation.identity,
          "call",
          cause,
          invocation.source
        ),
      })
    },
  })

  const unsubscribe = async (): Promise<void> => {
    if (unsubscribed || registration === undefined) return
    unsubscribed = true
    await registration.unsubscribe()
  }

  const registered = (async (): Promise<ProviderSubscriptionRegistration> => {
    try {
      const member = operationMember(invocation.entry, operation.identity)
      const returned = Reflect.apply(member, invocation.entry, [
        encodedInput,
        observer,
        invocation.attachment,
      ])
      const candidate = returned instanceof Promise ? await returned : returned
      registration = validateSubscriptionRegistration(candidate)
      if (closing) await unsubscribe()
      return registration
    } catch (cause) {
      const error =
        cause instanceof ProviderBoundaryDefect
          ? cause
          : boundaryDefect(
              provider,
              service,
              operation.identity,
              "call",
              cause,
              invocation.source
            )
      terminate({ kind: "defect", error })
      throw error
    }
  })()
  // A consumer may close before awaiting registration. Keep that rejected
  // registration observable through pull without creating an unhandled task.
  void registered.catch(() => undefined)

  return Object.freeze({
    async pull(context: EffectContext): Promise<IteratorResult<unknown>> {
      throwIfCancelled(context)
      if (closing) return { done: true, value: undefined }
      if (pendingPull !== undefined) {
        throw new TypeError("provider subscription allows one pending pull")
      }
      if (queue.length > 0) {
        return { done: false, value: queue.shift() }
      }
      const currentTerminal = terminal as Terminal | undefined
      if (currentTerminal !== undefined) {
        if (currentTerminal.kind === "complete") {
          return { done: true, value: undefined }
        }
        throw currentTerminal.error
      }
      const active = await registered
      if (terminal !== undefined) {
        if (terminal.kind === "complete") {
          return { done: true, value: undefined }
        }
        throw terminal.error
      }
      outstandingDemand += 1
      const result = new Promise<IteratorResult<unknown>>((resolve, reject) => {
        pendingPull = { resolve, reject }
      })
      try {
        await active.demand(1)
      } catch (cause) {
        terminate({
          kind: "defect",
          error: boundaryDefect(
            provider,
            service,
            operation.identity,
            "call",
            cause,
            invocation.source
          ),
        })
      }
      return await result
    },
    async close(): Promise<void> {
      if (closing) return
      closing = true
      outstandingDemand = 0
      queue.length = 0
      if (pendingPull !== undefined) {
        pendingPull.resolve({ done: true, value: undefined })
        pendingPull = undefined
      }
      try {
        await registered
      } catch {
        return
      }
      await unsubscribe()
    },
  })
}

function validateSubscriptionRegistration(
  value: unknown
): ProviderSubscriptionRegistration {
  const record = closedRecord(value, ["demand", "unsubscribe"])
  if (
    typeof record.demand !== "function" ||
    typeof record.unsubscribe !== "function"
  ) {
    throw new TypeError("provider subscription registration is invalid")
  }
  let closed = false
  return Object.freeze({
    demand(count: number) {
      if (closed) return
      if (!Number.isSafeInteger(count) || count <= 0) {
        throw new TypeError("provider subscription demand must be positive")
      }
      return Reflect.apply(
        record.demand as (...arguments_: ReadonlyArray<unknown>) => unknown,
        value,
        [count]
      ) as void | Promise<void>
    },
    unsubscribe() {
      if (closed) return
      closed = true
      return Reflect.apply(
        record.unsubscribe as (
          ...arguments_: ReadonlyArray<unknown>
        ) => unknown,
        value,
        []
      ) as void | Promise<void>
    },
  })
}

function boundaryDefect(
  provider: string,
  service: string,
  operation: string,
  stage: ProviderBoundaryStage,
  cause: unknown,
  source?: ProviderInvocationSource
): ProviderBoundaryDefect {
  return new ProviderBoundaryDefect(
    provider,
    service,
    operation,
    stage,
    cause instanceof Error ? cause.message : "provider boundary failed",
    cause,
    source
  )
}

function registerCancellation(
  completion: Promise<unknown>,
  context: EffectContext | undefined
): () => void {
  if (context === undefined) return () => undefined
  const descriptor = Object.getOwnPropertyDescriptor(
    completion,
    providerCancellation
  )
  if (descriptor === undefined || !("value" in descriptor)) {
    return () => undefined
  }
  if (typeof descriptor.value !== "function") {
    throw new TypeError("provider cancellation hook is invalid")
  }
  let notified = false
  return context.onCancel(() => {
    if (notified) return
    notified = true
    return Reflect.apply(
      descriptor.value,
      undefined,
      []
    ) as void | Promise<void>
  })
}

const providerHandleBrand: unique symbol = Symbol("seseragi.provider-handle")

export type ProviderHandle = Readonly<{
  readonly [providerHandleBrand]: true
}>

type ProviderHandleMetadata = Readonly<{
  token: object
  provider: string
  service: string
  handleType: string
}>

const providerHandles = new WeakMap<object, ProviderHandleMetadata>()

function projectValue(
  direction: "encode" | "decode",
  logicalType: ProviderLogicalType,
  value: unknown,
  codecs: ProviderCodecRegistry,
  owner?: ProviderValueOwner
): unknown {
  switch (logicalType.kind) {
    case "unit":
      if (value !== undefined) invalidValue("unit", value)
      return undefined
    case "never":
      return invalidValue("never", value)
    case "primitive":
      return projectPrimitive(logicalType.name, value)
    case "array":
      return projectArray(direction, logicalType.items, value, codecs, owner)
    case "record":
      return projectRecord(direction, logicalType.fields, value, codecs, owner)
    case "named": {
      if (value === undefined) invalidValue("named", value)
      if (direction === "encode" && isObject(value)) {
        const handle = providerHandles.get(value)
        if (handle !== undefined) {
          if (
            owner === undefined ||
            handle.provider !== owner.provider ||
            handle.service !== owner.service ||
            handle.handleType !== logicalType.identity
          ) {
            throw new TypeError(
              `provider handle does not belong to ${logicalType.identity}`
            )
          }
          return handle.token
        }
      }
      if (direction === "decode") {
        return codecs.codec(logicalType.identity).decode(value)
      }
      const encoded = codecs.codec(logicalType.identity).encode(value)
      if (encoded === undefined) invalidValue("named", encoded)
      return encoded
    }
  }
}

function projectPrimitive(
  name: ProviderPrimitiveName,
  value: unknown
): unknown {
  switch (name) {
    case "bool":
      if (typeof value !== "boolean") invalidValue(name, value)
      return value
    case "bytes":
      if (
        !(value instanceof Uint8Array) ||
        Object.getPrototypeOf(value) !== Uint8Array.prototype
      ) {
        invalidValue(name, value)
      }
      return new Uint8Array(value)
    case "float":
      if (typeof value !== "number") invalidValue(name, value)
      return value
    case "int":
      if (typeof value !== "number" || !Number.isSafeInteger(value)) {
        invalidValue(name, value)
      }
      return Object.is(value, -0) ? 0 : value
    case "string":
      if (typeof value !== "string") invalidValue(name, value)
      return value
  }
}

function projectArray(
  direction: "encode" | "decode",
  items: ProviderLogicalType,
  value: unknown,
  codecs: ProviderCodecRegistry,
  owner?: ProviderValueOwner
): ReadonlyArray<unknown> {
  if (
    !Array.isArray(value) ||
    Object.getPrototypeOf(value) !== Array.prototype
  ) {
    return invalidValue("array", value)
  }
  const keys = Reflect.ownKeys(value)
  if (
    keys.some(
      (key) =>
        typeof key !== "string" ||
        (key !== "length" && !isCanonicalArrayIndex(key, value.length))
    )
  ) {
    throw new TypeError("provider array must contain only indexed elements")
  }
  const projected = Array.from({ length: value.length }, (_, index) => {
    const descriptor = Object.getOwnPropertyDescriptor(value, String(index))
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError(`provider array item is missing: ${index}`)
    }
    return projectValue(direction, items, descriptor.value, codecs, owner)
  })
  return Object.freeze(projected)
}

function projectRecord(
  direction: "encode" | "decode",
  fields: ReadonlyArray<Readonly<{ name: string; type: ProviderLogicalType }>>,
  value: unknown,
  codecs: ProviderCodecRegistry,
  owner?: ProviderValueOwner
): Readonly<Record<string, unknown>> {
  const names = fields.map((field) => field.name)
  const record = closedRecord(value, names)
  const projected: Record<string, unknown> = {}
  for (const field of fields) {
    projected[field.name] = projectValue(
      direction,
      field.type,
      record[field.name],
      codecs,
      owner
    )
  }
  return Object.freeze(projected)
}

function decodeResult(
  value: unknown,
  operation: ProviderOperationContract,
  codecs: ProviderCodecRegistry,
  owner: ProviderValueOwner
): ProviderSuccess | ProviderFailure {
  const kind = dataProperty(value, "kind")
  if (kind === "success") {
    const result = closedRecord(value, ["kind", "value"])
    const success =
      operation.kind === "resource"
        ? decodeHandle(result.value, operation.success, owner)
        : decodeProviderValue(operation.success, result.value, codecs)
    return Object.freeze({ kind: "success", value: success })
  }
  if (kind === "failure") {
    const result = closedRecord(value, ["failure", "kind"])
    return Object.freeze({
      kind: "failure",
      failure: decodeProviderValue(operation.failure, result.failure, codecs),
    })
  }
  throw new TypeError("provider result kind must be success or failure")
}

function decodeHandle(
  value: unknown,
  logicalType: ProviderLogicalType,
  owner: ProviderValueOwner
): ProviderHandle {
  if (logicalType.kind !== "named") {
    throw new TypeError(
      "provider resource success must use a named handle type"
    )
  }
  if (!isObject(value)) {
    throw new TypeError("provider resource token must be an opaque object")
  }
  const handle = Object.create(null) as ProviderHandle
  Object.defineProperty(handle, providerHandleBrand, {
    enumerable: false,
    value: true,
  })
  providerHandles.set(handle, {
    token: value,
    provider: owner.provider,
    service: owner.service,
    handleType: logicalType.identity,
  })
  return Object.freeze(handle)
}

function operationMember(
  entry: unknown,
  operationIdentity: string
): (input: unknown) => unknown {
  if (!isPlainRecord(entry)) {
    throw new TypeError("provider entry export must be a plain object")
  }
  const separator = operationIdentity.lastIndexOf("#")
  const name = operationIdentity.slice(separator + 1)
  if (separator <= 0 || name.length === 0) {
    throw new TypeError("provider operation identity is not canonical")
  }
  const descriptor = Object.getOwnPropertyDescriptor(entry, name)
  if (
    descriptor === undefined ||
    !("value" in descriptor) ||
    typeof descriptor.value !== "function"
  ) {
    throw new TypeError(`provider operation member is invalid: ${name}`)
  }
  return descriptor.value as (input: unknown) => unknown
}

function closedRecord(
  value: unknown,
  expectedKeys: ReadonlyArray<string>
): Record<string, unknown> {
  if (!isPlainRecord(value)) {
    throw new TypeError("provider value must be a plain object")
  }
  const actualKeys = Reflect.ownKeys(value)
  if (
    actualKeys.length !== expectedKeys.length ||
    actualKeys.some((key) => typeof key !== "string")
  ) {
    throw new TypeError("provider record fields do not match the Contract")
  }
  const expected = new Set(expectedKeys)
  for (const key of actualKeys as string[]) {
    if (!expected.has(key)) {
      throw new TypeError(`provider record field is unknown: ${key}`)
    }
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError(
        `provider record field must be a data property: ${key}`
      )
    }
  }
  for (const key of expectedKeys) {
    if (!Object.hasOwn(value, key)) {
      throw new TypeError(`provider record field is missing: ${key}`)
    }
  }
  return value
}

function dataProperty(value: unknown, key: string): unknown {
  if (!isPlainRecord(value)) {
    throw new TypeError("provider result must be a plain object")
  }
  const descriptor = Object.getOwnPropertyDescriptor(value, key)
  if (descriptor === undefined || !("value" in descriptor)) {
    throw new TypeError(`provider result field must be a data property: ${key}`)
  }
  return descriptor.value
}

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  if (!isObject(value)) return false
  const prototype = Object.getPrototypeOf(value)
  return prototype === Object.prototype || prototype === null
}

function isObject(value: unknown): value is object {
  return typeof value === "object" && value !== null
}

function isCanonicalArrayIndex(key: string, length: number): boolean {
  const index = Number(key)
  return (
    Number.isSafeInteger(index) &&
    index >= 0 &&
    index < length &&
    `${index}` === key
  )
}

function invalidValue(kind: string, value: unknown): never {
  throw new TypeError(
    `provider ${kind} value is invalid: received ${valueDescription(value)}`
  )
}

function valueDescription(value: unknown): string {
  if (value === null) return "null"
  if (value === undefined) return "undefined"
  return typeof value
}

function defect(
  provider: string,
  service: string,
  operation: string,
  stage: ProviderBoundaryStage,
  cause: unknown,
  source?: ProviderInvocationSource
): ProviderDefect {
  const message =
    cause instanceof Error
      ? cause.message
      : "provider boundary operation failed"
  return Object.freeze({
    kind: "defect",
    defect: new ProviderBoundaryDefect(
      provider,
      service,
      operation,
      stage,
      message,
      cause,
      source
    ),
  })
}

function boundaryFrames(
  source: ProviderInvocationSource | undefined,
  cause: unknown
): ReadonlyArray<ProviderBoundaryFrame> {
  const frames: ProviderBoundaryFrame[] = []
  if (source !== undefined) {
    frames.push(Object.freeze({ kind: "seseragi", ...source }))
  }
  if (cause instanceof Error && typeof cause.stack === "string") {
    frames.push(Object.freeze({ kind: "host", stack: cause.stack }))
  }
  return Object.freeze(frames)
}
