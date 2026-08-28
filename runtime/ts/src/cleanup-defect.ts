export function defectWithSuppressed(defects: ReadonlyArray<unknown>): unknown {
  const first = defects[0]
  if (defects.length === 1) return first
  if (
    (typeof first === "object" && first !== null) ||
    typeof first === "function"
  ) {
    const previous = Object.getOwnPropertyDescriptor(first, "suppressed")?.value
    const suppressed = [
      ...(Array.isArray(previous) ? previous : []),
      ...defects.slice(1),
    ]
    try {
      Object.defineProperty(first, "suppressed", {
        configurable: true,
        enumerable: false,
        value: Object.freeze(suppressed),
      })
      return first
    } catch {
      // Frozen or otherwise non-extensible host errors still retain the
      // primary defect as AggregateError.cause.
    }
  }
  return new AggregateError(defects.slice(1), String(first), { cause: first })
}
