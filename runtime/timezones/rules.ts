import "@js-joda/timezone"
import {
  Instant,
  LocalDateTime,
  ZoneId,
  ZoneRulesProvider,
} from "@js-joda/core"

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

type PackedTzdb = Readonly<{
  version: string
  zones: ReadonlyArray<string>
  links: ReadonlyArray<string>
}>

type TzdbProvider = typeof ZoneRulesProvider & {
  getTzdbData: () => PackedTzdb
}

const data = (ZoneRulesProvider as TzdbProvider).getTzdbData()
const canonical = new Map<string, string>()
for (const packed of data.zones) {
  const separator = packed.indexOf("|")
  const id = separator < 0 ? packed : packed.slice(0, separator)
  canonical.set(id, id)
}
for (const packed of data.links) {
  const [target, alias] = packed.split("|")
  if (target !== undefined && alias !== undefined) canonical.set(alias, target)
}

export function bundledTimeZoneDatabaseVersion(): string {
  return data.version
}

export function canonicalTimeZoneId(id: string): string | undefined {
  return canonical.get(id)
}

export function timeZoneOffsetAtEpochSecond(
  id: string,
  epochSecond: number
): number {
  return ZoneId.of(id)
    .rules()
    .offsetOfInstant(Instant.ofEpochSecond(epochSecond))
    .totalSeconds()
}

export function resolveTimeZoneLocal(
  id: string,
  value: RuleLocalDateTime
): RuleLocalResolution {
  const local = LocalDateTime.of(
    value.year,
    value.month,
    value.day,
    value.hour,
    value.minute,
    value.second,
    value.nanosecond
  )
  const rules = ZoneId.of(id).rules()
  const offsets = rules
    .validOffsets(local)
    .map((offset) => offset.totalSeconds())
  if (offsets.length === 1) {
    return { kind: "unique", offsets: [offsets[0] as number] }
  }
  if (offsets.length === 2) {
    return {
      kind: "ambiguous",
      offsets: [offsets[0] as number, offsets[1] as number],
    }
  }
  const transition = rules.transition(local)
  if (transition === null) {
    throw new TypeError("timezone rules returned no transition for a gap")
  }
  const before = transition.dateTimeBefore()
  return {
    kind: "gap",
    offsetBefore: transition.offsetBefore().totalSeconds(),
    offsetAfter: transition.offsetAfter().totalSeconds(),
    transitionLocal: {
      year: before.year(),
      month: before.monthValue(),
      day: before.dayOfMonth(),
      hour: before.hour(),
      minute: before.minute(),
      second: before.second(),
      nanosecond: before.nano(),
    },
  }
}
