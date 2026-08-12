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
  input: unknown
) => Promise<ProviderResult>

export type ProviderEntry = Readonly<Record<string, ProviderOperationMember>>

export type ProviderDefect = Readonly<{
  kind: "defect"
  defect: ProviderBoundaryDefect
}>

export type ProviderBridgeOutcome = ProviderResult | ProviderDefect

export type ProviderBoundaryStage = "input" | "call" | "result"

export class ProviderBoundaryDefect extends Error {
  readonly provider: string
  readonly service: string
  readonly operation: string
  readonly stage: ProviderBoundaryStage
  override readonly cause: unknown

  constructor(
    provider: string,
    service: string,
    operation: string,
    stage: ProviderBoundaryStage,
    message: string,
    cause: unknown
  ) {
    super(message)
    this.name = "ProviderBoundaryDefect"
    this.provider = provider
    this.service = service
    this.operation = operation
    this.stage = stage
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
}>

export async function invokeProviderOperation(
  invocation: ProviderInvocation
): Promise<ProviderBridgeOutcome> {
  const { provider, service, operation, codecs } = invocation
  if (operation.kind === "subscription") {
    return defect(
      provider,
      service,
      operation.identity,
      "call",
      new TypeError("provider subscription requires the stream bridge")
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
    return defect(provider, service, operation.identity, "input", cause)
  }

  let completion: Promise<unknown>
  try {
    const member = operationMember(invocation.entry, operation.identity)
    const returned = Reflect.apply(member, invocation.entry, [input])
    if (!(returned instanceof Promise)) {
      throw new TypeError("provider operation must return a Promise")
    }
    completion = returned
  } catch (cause) {
    return defect(provider, service, operation.identity, "call", cause)
  }

  let result: unknown
  try {
    result = await completion
  } catch (cause) {
    return defect(provider, service, operation.identity, "call", cause)
  }

  try {
    return decodeResult(result, operation, codecs, owner)
  } catch (cause) {
    return defect(provider, service, operation.identity, "result", cause)
  }
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
  cause: unknown
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
      cause
    ),
  })
}
