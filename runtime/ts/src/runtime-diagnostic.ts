export type RuntimeSource = Readonly<{
  readonly file: string
  readonly line: number
  readonly column: number
  readonly uri: string
  readonly start: number
  readonly end: number
}>

const defectSources = new WeakMap<object, RuntimeSource>()

export function runtimeDefectSource(value: unknown): RuntimeSource | undefined {
  return isObject(value) ? defectSources.get(value) : undefined
}

export function locateRuntimeDefect(
  value: unknown,
  source: RuntimeSource | undefined
): unknown {
  if (source === undefined) return value
  const error = isObject(value)
    ? value
    : new Error(runtimeDefectMessage(value), { cause: value })
  if (!defectSources.has(error)) defectSources.set(error, source)
  return error
}

export function runtimeDefectMessage(value: unknown): string {
  if (value instanceof Error) return value.message
  if (typeof value === "string" && value.length > 0) return value
  try {
    return String(value)
  } catch {
    return "runtime defect"
  }
}

function isObject(value: unknown): value is object {
  return (
    (typeof value === "object" && value !== null) || typeof value === "function"
  )
}
