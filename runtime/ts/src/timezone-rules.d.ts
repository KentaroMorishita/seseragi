export type RuleLocalDateTime = Readonly<{
  year: number
  month: number
  day: number
  hour: number
  minute: number
  second: number
  nanosecond: number
}>

export type RuleLocalResolution =
  | Readonly<{ kind: "unique"; offsets: readonly [number] }>
  | Readonly<{ kind: "ambiguous"; offsets: readonly [number, number] }>
  | Readonly<{
      kind: "gap"
      offsetBefore: number
      offsetAfter: number
      transitionLocal: RuleLocalDateTime
    }>

export function bundledTimeZoneDatabaseVersion(): string
export function canonicalTimeZoneId(id: string): string | undefined
export function timeZoneOffsetAtEpochSecond(
  id: string,
  epochSecond: number
): number
export function resolveTimeZoneLocal(
  id: string,
  value: RuleLocalDateTime
): RuleLocalResolution
