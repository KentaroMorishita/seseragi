import {
  assertProviderRuntimeAbi,
  type ProviderEntry,
  type ProviderRuntimeAbi,
  providerRuntimeAbi,
} from "./provider"

export const providerPackageRuntime = Object.freeze({
  identity: "seseragi/provider-package/typescript",
  version: 1,
  abi: providerRuntimeAbi,
} as const)

export type ProviderPackageRuntime = typeof providerPackageRuntime
export type ProviderLoadMode = "eager" | "lazy"
export type ProviderRuntimeTarget = "browser" | "bun-process" | "node-process"

export type ProviderSourceLocation = Readonly<{
  path: string
  start: number
  end: number
}>

export type ProviderStackFrame =
  | Readonly<{
      kind: "seseragi"
      path: string
      start: number
      end: number
    }>
  | Readonly<{ kind: "host"; stack: string }>

export type ProviderPackageDefinition = Readonly<{
  abi: ProviderRuntimeAbi
  provider: string
  service: string
  targets: ReadonlyArray<ProviderRuntimeTarget>
  operations: ProviderEntry
  shutdown?: () => Promise<void>
}>

export type ProviderPackageEntry = ProviderEntry

export type ProviderModuleSelection = Readonly<{
  provider: string
  service: string
  target: ProviderRuntimeTarget
  module: string
  exportName: string
  loadMode: ProviderLoadMode
  importModule: () => Promise<unknown>
  source?: ProviderSourceLocation
}>

export type LoadedProviderEntry = Readonly<{
  provider: string
  service: string
  entry: ProviderPackageEntry
}>

export type ProviderPackageStage = "load" | "validate" | "shutdown"

export class ProviderPackageDefect extends Error {
  readonly provider: string
  readonly service: string
  readonly module: string
  readonly stage: ProviderPackageStage
  readonly frames: ReadonlyArray<ProviderStackFrame>
  readonly notes: ReadonlyArray<ProviderPackageDefect>
  override readonly cause: unknown

  constructor(
    selection: Pick<
      ProviderModuleSelection,
      "module" | "provider" | "service" | "source"
    >,
    stage: ProviderPackageStage,
    cause: unknown,
    notes: ReadonlyArray<ProviderPackageDefect> = []
  ) {
    super(errorMessage(cause))
    this.name = "ProviderPackageDefect"
    this.provider = selection.provider
    this.service = selection.service
    this.module = selection.module
    this.stage = stage
    this.frames = providerStack(selection.source, cause)
    this.notes = Object.freeze([...notes])
    this.cause = cause
  }
}

const entryBrand = Symbol.for("seseragi.provider-package.entry.v1")
const runtimeStateKey = Symbol.for("seseragi.provider-package.runtime.v1")

type EntryMetadata = Readonly<{
  provider: string
  service: string
  targets: ReadonlyArray<ProviderRuntimeTarget>
  shutdown?: () => Promise<void>
}>

type RuntimeState = {
  identity: string
  entries: Map<string, Readonly<{ entry: ProviderPackageEntry; owner: symbol }>>
}

type GlobalWithProviderState = typeof globalThis & {
  [runtimeStateKey]?: RuntimeState
}

type BrandedEntry = ProviderPackageEntry & {
  readonly [entryBrand]: EntryMetadata
}

export function defineProviderPackage(
  definition: ProviderPackageDefinition
): ProviderPackageEntry {
  assertProviderRuntimeAbi(definition.abi)
  canonicalIdentity(definition.provider, "provider identity")
  canonicalIdentity(definition.service, "service identity")
  const targets = uniqueTargets(definition.targets)
  const operations = snapshotOperations(definition.operations)
  if (
    definition.shutdown !== undefined &&
    typeof definition.shutdown !== "function"
  ) {
    throw new TypeError("provider shutdown must be a function")
  }
  Object.defineProperty(operations, entryBrand, {
    enumerable: false,
    value: Object.freeze({
      provider: definition.provider,
      service: definition.service,
      targets,
      ...(definition.shutdown === undefined
        ? {}
        : { shutdown: definition.shutdown }),
    }),
  })
  return Object.freeze(operations)
}

export class ProviderPackageLoader {
  readonly #selections: ReadonlyMap<string, ProviderModuleSelection>
  readonly #loads = new Map<string, Promise<LoadedProviderEntry>>()
  readonly #loaded: Array<LoadedProviderEntry> = []
  readonly #owner = Symbol("seseragi.provider-package.loader")
  #accepting = true
  #start: Promise<void> | undefined
  #shutdown: Promise<void> | undefined

  constructor(
    target: ProviderRuntimeTarget,
    selections: Iterable<ProviderModuleSelection>
  ) {
    validateRuntimeTarget(target)
    const indexed = new Map<string, ProviderModuleSelection>()
    for (const selection of selections) {
      validateSelection(selection, target)
      if (indexed.has(selection.provider)) {
        throw new TypeError(
          `provider package selection is duplicated: ${selection.provider}`
        )
      }
      indexed.set(selection.provider, Object.freeze({ ...selection }))
    }
    this.#selections = indexed
  }

  start(): Promise<void> {
    if (!this.#accepting) {
      return Promise.reject(
        new TypeError("provider package loader is shutting down")
      )
    }
    this.#start ??= this.#startEager()
    return this.#start
  }

  async load(provider: string): Promise<LoadedProviderEntry> {
    if (!this.#accepting) {
      throw new TypeError("provider package loader is shutting down")
    }
    const selection = this.#selections.get(provider)
    if (selection === undefined) {
      throw new TypeError(`provider package is not selected: ${provider}`)
    }
    let loading = this.#loads.get(provider)
    if (loading === undefined) {
      loading = this.#loadSelection(selection)
      this.#loads.set(provider, loading)
    }
    return loading
  }

  shutdown(): Promise<void> {
    this.#accepting = false
    this.#shutdown ??= this.#shutdownLoaded()
    return this.#shutdown
  }

  async #startEager(): Promise<void> {
    for (const selection of this.#selections.values()) {
      if (selection.loadMode === "eager") await this.load(selection.provider)
    }
  }

  async #loadSelection(
    selection: ProviderModuleSelection
  ): Promise<LoadedProviderEntry> {
    let module: unknown
    try {
      const returned = selection.importModule()
      if (!(returned instanceof Promise)) {
        throw new TypeError("provider module loader must return a Promise")
      }
      module = await returned
    } catch (cause) {
      throw new ProviderPackageDefect(selection, "load", cause)
    }

    try {
      const entry = moduleExport(module, selection.exportName)
      const metadata = entryMetadata(entry)
      if (
        metadata.provider !== selection.provider ||
        metadata.service !== selection.service ||
        !metadata.targets.includes(selection.target)
      ) {
        throw new TypeError(
          "provider entry identity, service, or target does not match selection"
        )
      }
      const state = runtimeState()
      const existing = state.entries.get(selection.provider)
      if (existing !== undefined) {
        throw new TypeError(
          `provider singleton is already owned: ${selection.provider}`
        )
      }
      state.entries.set(
        selection.provider,
        Object.freeze({ entry, owner: this.#owner })
      )
      const loaded = Object.freeze({
        provider: selection.provider,
        service: selection.service,
        entry,
      })
      this.#loaded.push(loaded)
      return loaded
    } catch (cause) {
      throw new ProviderPackageDefect(selection, "validate", cause)
    }
  }

  async #shutdownLoaded(): Promise<void> {
    const pending = [...this.#loads.values()]
    await Promise.allSettled(pending)
    const defects: ProviderPackageDefect[] = []
    for (const loaded of [...this.#loaded].reverse()) {
      const selection = this.#selections.get(loaded.provider)
      if (selection === undefined) continue
      const metadata = entryMetadata(loaded.entry)
      if (metadata.shutdown !== undefined) {
        try {
          const returned = metadata.shutdown()
          if (!(returned instanceof Promise)) {
            throw new TypeError("provider shutdown must return a Promise")
          }
          await returned
        } catch (cause) {
          defects.push(new ProviderPackageDefect(selection, "shutdown", cause))
        }
      }
      const state = runtimeState()
      const registered = state.entries.get(loaded.provider)
      if (
        registered?.owner === this.#owner &&
        registered.entry === loaded.entry
      ) {
        state.entries.delete(loaded.provider)
      }
    }
    if (defects.length > 0) {
      const primary = defects[0]
      if (primary === undefined) return
      throw new ProviderPackageDefect(
        {
          provider: primary.provider,
          service: primary.service,
          module: primary.module,
        },
        "shutdown",
        primary.cause,
        defects.slice(1)
      )
    }
  }
}

function snapshotOperations(entry: ProviderEntry): BrandedEntry {
  if (!isPlainRecord(entry)) {
    throw new TypeError("provider operations must be a plain object")
  }
  const operations = Object.create(null) as BrandedEntry
  for (const key of Reflect.ownKeys(entry)) {
    if (typeof key !== "string") {
      throw new TypeError("provider operations must not contain symbol members")
    }
    const descriptor = Object.getOwnPropertyDescriptor(entry, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      typeof descriptor.value !== "function"
    ) {
      throw new TypeError(`provider operation must be a data function: ${key}`)
    }
    Object.defineProperty(operations, key, {
      enumerable: true,
      value: descriptor.value,
    })
  }
  return operations
}

function validateSelection(
  selection: ProviderModuleSelection,
  runtimeTarget: ProviderRuntimeTarget
): void {
  canonicalIdentity(selection.provider, "provider identity")
  canonicalIdentity(selection.service, "service identity")
  if (selection.target !== runtimeTarget) {
    throw new TypeError(
      `provider target ${selection.target} cannot run on ${runtimeTarget}`
    )
  }
  if (
    selection.module.length === 0 ||
    selection.exportName.length === 0 ||
    typeof selection.importModule !== "function"
  ) {
    throw new TypeError("provider module selection is incomplete")
  }
  if (selection.source !== undefined) validateSource(selection.source)
}

function moduleExport(
  module: unknown,
  exportName: string
): ProviderPackageEntry {
  if (
    (typeof module !== "object" && typeof module !== "function") ||
    module === null
  ) {
    throw new TypeError("provider module must be an ESM namespace object")
  }
  const descriptor = Object.getOwnPropertyDescriptor(module, exportName)
  if (descriptor === undefined || !("value" in descriptor)) {
    throw new TypeError(`provider module export is invalid: ${exportName}`)
  }
  return descriptor.value as ProviderPackageEntry
}

function entryMetadata(entry: ProviderPackageEntry): EntryMetadata {
  if (!isPlainRecord(entry)) {
    throw new TypeError("provider entry must be a plain object")
  }
  const descriptor = Object.getOwnPropertyDescriptor(entry, entryBrand)
  if (descriptor === undefined || !("value" in descriptor)) {
    throw new TypeError(
      "provider entry does not use the official package brand"
    )
  }
  return descriptor.value as EntryMetadata
}

function runtimeState(): RuntimeState {
  const host = globalThis as GlobalWithProviderState
  const existing = host[runtimeStateKey]
  if (existing !== undefined) {
    if (existing.identity !== providerPackageRuntime.identity) {
      throw new TypeError(
        "provider package runtime singleton identity mismatch"
      )
    }
    return existing
  }
  const state: RuntimeState = {
    identity: providerPackageRuntime.identity,
    entries: new Map(),
  }
  Object.defineProperty(host, runtimeStateKey, { value: state })
  return state
}

function providerStack(
  source: ProviderSourceLocation | undefined,
  cause: unknown
): ReadonlyArray<ProviderStackFrame> {
  const frames: ProviderStackFrame[] = []
  if (source !== undefined) {
    frames.push(Object.freeze({ kind: "seseragi", ...source }))
  }
  if (cause instanceof Error && typeof cause.stack === "string") {
    frames.push(Object.freeze({ kind: "host", stack: cause.stack }))
  }
  return Object.freeze(frames)
}

function validateSource(source: ProviderSourceLocation): void {
  if (
    source.path.length === 0 ||
    !Number.isSafeInteger(source.start) ||
    !Number.isSafeInteger(source.end) ||
    source.start < 0 ||
    source.end < source.start
  ) {
    throw new TypeError("provider source location is invalid")
  }
}

function uniqueTargets(
  targets: ReadonlyArray<ProviderRuntimeTarget>
): ReadonlyArray<ProviderRuntimeTarget> {
  if (targets.length === 0 || new Set(targets).size !== targets.length) {
    throw new TypeError("provider targets must be non-empty and unique")
  }
  for (const target of targets) validateRuntimeTarget(target)
  return Object.freeze([...targets])
}

function validateRuntimeTarget(target: ProviderRuntimeTarget): void {
  if (!(["browser", "bun-process", "node-process"] as const).includes(target)) {
    throw new TypeError(`provider runtime target is invalid: ${String(target)}`)
  }
}

function canonicalIdentity(value: string, label: string): void {
  if (value.length === 0 || value.trim() !== value || !value.includes("/")) {
    throw new TypeError(`${label} is not canonical`)
  }
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "provider package failed"
}

function isPlainRecord(value: unknown): value is Record<PropertyKey, unknown> {
  if (typeof value !== "object" || value === null) return false
  const prototype = Object.getPrototypeOf(value)
  return prototype === Object.prototype || prototype === null
}
