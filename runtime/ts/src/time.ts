import {
  resolveTimeZoneLocal,
  timeZoneOffsetAtEpochSecond,
  type RuleLocalResolution,
} from "./timezone-rules.js"
import { createInstant, type Instant, instantNanoseconds } from "./clock-value"
import type { Effect, EffectContext } from "./effect"
import { serviceEffect, type serviceFailure, serviceSuccess } from "./service"
import { type Either, Left, type Left as LeftValue, Right } from "./sum"

const NANOS_PER_SECOND = 1_000_000_000n
const SECONDS_PER_DAY = 86_400n
const NANOS_PER_DAY = SECONDS_PER_DAY * NANOS_PER_SECOND
const LAST_BUNDLED_YEAR = 2499
const FIRST_BUNDLED_YEAR = 1800

export type DateTimeError =
  | Readonly<{
      tag: "InvalidDate"
      value: Readonly<{ year: number; month: number; day: number }>
    }>
  | Readonly<{
      tag: "InvalidTime"
      value: Readonly<{
        hour: number
        minute: number
        second: number
        nanosecond: number
      }>
    }>
  | Readonly<{ tag: "InvalidUtcOffsetSeconds"; value: number }>
  | Readonly<{
      tag: "InvalidDateTimeText"
      value: Readonly<{ offset: number }>
    }>

export type TimeZoneError =
  | Readonly<{ tag: "UnknownTimeZone"; value: string }>
  | Readonly<{ tag: "TimeZoneDatabaseUnavailable"; value: string }>
  | Readonly<{
      tag: "TimeZoneDatabaseVersionMismatch"
      value: Readonly<{ required: string; actual: string }>
    }>

type DateFields = Readonly<{ year: number; month: number; day: number }>
type TimeFields = Readonly<{
  hour: number
  minute: number
  second: number
  nanosecond: number
}>

declare const localDateBrand: unique symbol
declare const localTimeBrand: unique symbol
declare const localDateTimeBrand: unique symbol
declare const utcOffsetBrand: unique symbol
declare const offsetDateTimeBrand: unique symbol
declare const timeZoneBrand: unique symbol
declare const zonedDateTimeBrand: unique symbol

export type LocalDate = Readonly<{ readonly [localDateBrand]: true }>
export type LocalTime = Readonly<{ readonly [localTimeBrand]: true }>
export type LocalDateTime = Readonly<{ readonly [localDateTimeBrand]: true }>
export type UtcOffset = Readonly<{ readonly [utcOffsetBrand]: true }>
export type OffsetDateTime = Readonly<{
  readonly [offsetDateTimeBrand]: true
}>
export type TimeZone = Readonly<{ readonly [timeZoneBrand]: true }>
export type ZonedDateTime = Readonly<{
  readonly [zonedDateTimeBrand]: true
}>

export type LocalResolution =
  | Readonly<{ tag: "Unique"; value: ZonedDateTime }>
  | Readonly<{
      tag: "Ambiguous"
      value: Readonly<{ earlier: ZonedDateTime; later: ZonedDateTime }>
    }>
  | Readonly<{
      tag: "Gap"
      value: Readonly<{
        transition: Instant
        offsetBefore: UtcOffset
        offsetAfter: UtcOffset
      }>
    }>

type OffsetDateTimeFields = Readonly<{ instant: Instant; offset: UtcOffset }>
type TimeZoneFields = Readonly<{ id: string; version: string }>
type ZonedDateTimeFields = Readonly<{
  instant: Instant
  local: LocalDateTime
  offset: UtcOffset
  zone: TimeZone
}>

const dates = new WeakMap<object, DateFields>()
const times = new WeakMap<object, TimeFields>()
const localDateTimes = new WeakMap<
  object,
  Readonly<{ date: LocalDate; time: LocalTime }>
>()
const offsets = new WeakMap<object, number>()
const offsetDateTimes = new WeakMap<object, OffsetDateTimeFields>()
const timeZones = new WeakMap<object, TimeZoneFields>()
const zonedDateTimes = new WeakMap<object, ZonedDateTimeFields>()

export const InvalidDate = (value: DateFields): DateTimeError => ({
  tag: "InvalidDate",
  value,
})

export const InvalidTime = (value: TimeFields): DateTimeError => ({
  tag: "InvalidTime",
  value,
})

export const InvalidUtcOffsetSeconds = (value: number): DateTimeError => ({
  tag: "InvalidUtcOffsetSeconds",
  value,
})

export const InvalidDateTimeText = (value: {
  readonly offset: number
}): DateTimeError => ({ tag: "InvalidDateTimeText", value })

export const UnknownTimeZone = (value: string): TimeZoneError => ({
  tag: "UnknownTimeZone",
  value,
})

export const TimeZoneDatabaseUnavailable = (value: string): TimeZoneError => ({
  tag: "TimeZoneDatabaseUnavailable",
  value,
})

export const TimeZoneDatabaseVersionMismatch = (value: {
  readonly required: string
  readonly actual: string
}): TimeZoneError => ({ tag: "TimeZoneDatabaseVersionMismatch", value })

export const Unique = (value: ZonedDateTime): LocalResolution => ({
  tag: "Unique",
  value,
})

export const Ambiguous = (value: {
  readonly earlier: ZonedDateTime
  readonly later: ZonedDateTime
}): LocalResolution => ({ tag: "Ambiguous", value })

export const Gap = (value: {
  readonly transition: Instant
  readonly offsetBefore: UtcOffset
  readonly offsetAfter: UtcOffset
}): LocalResolution => ({ tag: "Gap", value })

export function localDate(
  year: number,
  month: number,
  day: number
): Either<DateTimeError, LocalDate> {
  if (!validDate(year, month, day)) {
    return Left(InvalidDate({ year, month, day }))
  }
  return Right(makeDate({ year, month, day }))
}

export function localTime(
  hour: number,
  minute: number,
  second: number,
  nanosecond: number
): Either<DateTimeError, LocalTime> {
  const value = { hour, minute, second, nanosecond }
  if (
    ![hour, minute, second, nanosecond].every(Number.isSafeInteger) ||
    hour < 0 ||
    hour > 23 ||
    minute < 0 ||
    minute > 59 ||
    second < 0 ||
    second > 59 ||
    nanosecond < 0 ||
    nanosecond > 999_999_999
  ) {
    return Left(InvalidTime(value))
  }
  return Right(makeTime(value))
}

export function localDateTime(date: LocalDate, time: LocalTime): LocalDateTime {
  dateFields(date)
  timeFields(time)
  const value = Object.freeze({}) as LocalDateTime
  localDateTimes.set(value, Object.freeze({ date, time }))
  return value
}

export function utcOffset(seconds: number): Either<DateTimeError, UtcOffset> {
  if (!Number.isSafeInteger(seconds) || Math.abs(seconds) > 18 * 60 * 60) {
    return Left(InvalidUtcOffsetSeconds(seconds))
  }
  return Right(makeOffset(seconds))
}

export function parseLocalDate(text: string): Either<DateTimeError, LocalDate> {
  const match = /^(\d{4}|[+-]\d{6,})-(\d{2})-(\d{2})$/.exec(text)
  if (match === null) return textFailure(text, localDateShapeOffset(text))
  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  return localDate(year, month, day)
}

export function parseLocalTime(text: string): Either<DateTimeError, LocalTime> {
  const match = /^(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,9}))?$/.exec(text)
  if (match === null) return textFailure(text, localTimeShapeOffset(text))
  return localTime(
    Number(match[1]),
    Number(match[2]),
    Number(match[3]),
    Number((match[4] ?? "").padEnd(9, "0"))
  )
}

export function parseLocalDateTime(
  text: string
): Either<DateTimeError, LocalDateTime> {
  const separator = text.indexOf("T")
  if (separator <= 0 || separator !== text.lastIndexOf("T")) {
    return textFailure(text, separator < 0 ? text.length : separator)
  }
  const date = parseLocalDate(text.slice(0, separator))
  if (date.tag === "Left") return date
  const time = parseLocalTime(text.slice(separator + 1))
  if (time.tag === "Left") {
    return shiftTextFailure(time, text.slice(0, separator + 1))
  }
  return Right(localDateTime(date.value, time.value))
}

export function parseOffsetDateTime(
  text: string
): Either<DateTimeError, OffsetDateTime> {
  const offsetMatch = /(Z|[+-]\d{2}:\d{2})$/.exec(text)
  if (offsetMatch === null) return textFailure(text, text.length)
  const localText = text.slice(0, offsetMatch.index)
  const parsedLocal = parseLocalDateTime(localText)
  if (parsedLocal.tag === "Left") return parsedLocal
  let offsetSeconds = 0
  if (offsetMatch[1] !== "Z") {
    const sign = offsetMatch[1]?.startsWith("-") ? -1 : 1
    const hour = Number(offsetMatch[1]?.slice(1, 3))
    const minute = Number(offsetMatch[1]?.slice(4, 6))
    if (minute > 59) return textFailure(text, offsetMatch.index + 4)
    offsetSeconds = sign * (hour * 3600 + minute * 60)
  }
  const parsedOffset = utcOffset(offsetSeconds)
  if (parsedOffset.tag === "Left") return parsedOffset
  const instant = createInstant(
    localEpochNanoseconds(parsedLocal.value) -
      BigInt(offsetSeconds) * NANOS_PER_SECOND
  )
  return Right(makeOffsetDateTime(instant, parsedOffset.value))
}

export function formatLocalDate(value: LocalDate): string {
  const { year, month, day } = dateFields(value)
  return `${formatYear(year)}-${two(month)}-${two(day)}`
}

export function formatLocalTime(value: LocalTime): string {
  const { hour, minute, second, nanosecond } = timeFields(value)
  const fraction =
    nanosecond === 0
      ? ""
      : `.${String(nanosecond).padStart(9, "0").replace(/0+$/, "")}`
  return `${two(hour)}:${two(minute)}:${two(second)}${fraction}`
}

export function formatLocalDateTime(value: LocalDateTime): string {
  const fields = localFields(value)
  return `${formatLocalDate(fields.date)}T${formatLocalTime(fields.time)}`
}

export function formatOffsetDateTime(value: OffsetDateTime): string {
  const fields = offsetDateTimeFields(value)
  return `${formatLocalDateTime(offsetLocalDateTime(value))}${formatOffset(
    offsetSeconds(fields.offset)
  )}`
}

export function atOffset(offset: UtcOffset, instant: Instant): OffsetDateTime {
  offsetSeconds(offset)
  instantNanoseconds(instant)
  return makeOffsetDateTime(instant, offset)
}

export function offsetInstant(value: OffsetDateTime): Instant {
  return offsetDateTimeFields(value).instant
}

export function offsetLocalDateTime(value: OffsetDateTime): LocalDateTime {
  const fields = offsetDateTimeFields(value)
  return localFromEpochNanoseconds(
    instantNanoseconds(fields.instant) +
      BigInt(offsetSeconds(fields.offset)) * NANOS_PER_SECOND
  )
}

export type TimeZones = Readonly<{
  databaseVersion: (context: EffectContext) => Promise<string>
  loadTimeZone: (
    id: string,
    context: EffectContext
  ) => Promise<
    | ReturnType<typeof serviceSuccess<TimeZone>>
    | ReturnType<typeof serviceFailure<TimeZoneError>>
  >
}>

export type TimeZonesEnvironment = Readonly<{ timeZones: TimeZones }>

export function databaseVersion(): Effect<TimeZonesEnvironment, never, string> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.timeZones.databaseVersion(context))
  )
}

export function loadTimeZone(
  id: string
): Effect<TimeZonesEnvironment, TimeZoneError, TimeZone> {
  return serviceEffect((environment, context) =>
    environment.timeZones.loadTimeZone(id, context)
  )
}

export function createTimeZone(id: string, version: string): TimeZone {
  if (id.length === 0 || version.length === 0) {
    throw new TypeError("TimeZone snapshot requires an ID and tzdb version")
  }
  const value = Object.freeze({}) as TimeZone
  timeZones.set(value, Object.freeze({ id, version }))
  return value
}

export function timeZoneProviderValue(value: TimeZone): Readonly<{
  id: string
  version: string
}> {
  return timeZoneFields(value)
}

export function timeZoneId(zone: TimeZone): string {
  return timeZoneFields(zone).id
}

export function timeZoneVersion(zone: TimeZone): string {
  return timeZoneFields(zone).version
}

export function atTimeZone(instant: Instant, zone: TimeZone): ZonedDateTime {
  const nanoseconds = instantNanoseconds(instant)
  const offset = makeOffset(zoneOffsetAtInstant(zone, nanoseconds))
  const local = localFromEpochNanoseconds(
    nanoseconds + BigInt(offsetSeconds(offset)) * NANOS_PER_SECOND
  )
  return makeZonedDateTime(instant, local, offset, zone)
}

export function resolveLocal(
  local: LocalDateTime,
  zone: TimeZone
): LocalResolution {
  const localValue = localValueFields(local)
  const mappedYear = mappedRuleYear(localValue.year)
  if (localValue.year < FIRST_BUNDLED_YEAR) {
    const offset = makeOffset(initialZoneOffset(zone))
    return Unique(zonedFromLocal(local, zone, offset))
  }
  const resolution = resolveTimeZoneLocal(timeZoneId(zone), {
    ...localValue,
    year: mappedYear,
  })
  if (resolution.kind === "unique") {
    return Unique(
      zonedFromLocal(local, zone, makeOffset(resolution.offsets[0]))
    )
  }
  if (resolution.kind === "ambiguous") {
    const values = resolution.offsets
      .map((entry) => zonedFromLocal(local, zone, makeOffset(entry)))
      .sort((left, right) =>
        instantNanoseconds(zonedInstant(left)) <
        instantNanoseconds(zonedInstant(right))
          ? -1
          : 1
      )
    return Ambiguous({
      earlier: values[0] as ZonedDateTime,
      later: values[1] as ZonedDateTime,
    })
  }
  return gapResolution(localValue.year, resolution)
}

export function zonedInstant(value: ZonedDateTime): Instant {
  return zonedFields(value).instant
}

export function zonedLocalDateTime(value: ZonedDateTime): LocalDateTime {
  return zonedFields(value).local
}

export function zonedOffset(value: ZonedDateTime): UtcOffset {
  return zonedFields(value).offset
}

export function zonedTimeZone(value: ZonedDateTime): TimeZone {
  return zonedFields(value).zone
}

function gapResolution(
  originalYear: number,
  transition: Extract<RuleLocalResolution, { kind: "gap" }>
): LocalResolution {
  const transitionLocal = localDateTimeFromFields({
    year: originalYear,
    month: transition.transitionLocal.month,
    day: transition.transitionLocal.day,
    hour: transition.transitionLocal.hour,
    minute: transition.transitionLocal.minute,
    second: transition.transitionLocal.second,
    nanosecond: transition.transitionLocal.nanosecond,
  })
  return Gap({
    transition: createInstant(
      localEpochNanoseconds(transitionLocal) -
        BigInt(transition.offsetBefore) * NANOS_PER_SECOND
    ),
    offsetBefore: makeOffset(transition.offsetBefore),
    offsetAfter: makeOffset(transition.offsetAfter),
  })
}

function zoneOffsetAtInstant(zone: TimeZone, nanoseconds: bigint): number {
  const utc = localFromEpochNanoseconds(nanoseconds)
  const fields = localValueFields(utc)
  if (fields.year < FIRST_BUNDLED_YEAR) return initialZoneOffset(zone)
  const mappedYear = mappedRuleYear(fields.year)
  const mappedLocal = localDateTimeFromFields({ ...fields, year: mappedYear })
  const mappedSeconds = localEpochNanoseconds(mappedLocal) / NANOS_PER_SECOND
  return timeZoneOffsetAtEpochSecond(timeZoneId(zone), Number(mappedSeconds))
}

function initialZoneOffset(zone: TimeZone): number {
  return timeZoneOffsetAtEpochSecond(
    timeZoneId(zone),
    Number(
      localEpochNanoseconds(
        localDateTimeFromFields({
          year: FIRST_BUNDLED_YEAR,
          month: 1,
          day: 1,
          hour: 0,
          minute: 0,
          second: 0,
          nanosecond: 0,
        })
      ) / NANOS_PER_SECOND
    )
  )
}

function mappedRuleYear(year: number): number {
  if (year <= LAST_BUNDLED_YEAR) return year
  return 2100 + modulo(year - 2100, 400)
}

function zonedFromLocal(
  local: LocalDateTime,
  zone: TimeZone,
  offset: UtcOffset
): ZonedDateTime {
  const instant = createInstant(
    localEpochNanoseconds(local) -
      BigInt(offsetSeconds(offset)) * NANOS_PER_SECOND
  )
  return makeZonedDateTime(instant, local, offset, zone)
}

function localDateTimeFromFields(
  fields: DateFields & TimeFields
): LocalDateTime {
  return localDateTime(
    makeDate({ year: fields.year, month: fields.month, day: fields.day }),
    makeTime({
      hour: fields.hour,
      minute: fields.minute,
      second: fields.second,
      nanosecond: fields.nanosecond,
    })
  )
}

function localValueFields(value: LocalDateTime): DateFields & TimeFields {
  const fields = localFields(value)
  return { ...dateFields(fields.date), ...timeFields(fields.time) }
}

function localEpochNanoseconds(value: LocalDateTime): bigint {
  const fields = localValueFields(value)
  const day = daysFromCivil(fields.year, fields.month, fields.day)
  const seconds = BigInt(
    fields.hour * 3600 + fields.minute * 60 + fields.second
  )
  return (
    day * NANOS_PER_DAY + seconds * NANOS_PER_SECOND + BigInt(fields.nanosecond)
  )
}

function localFromEpochNanoseconds(nanoseconds: bigint): LocalDateTime {
  const day = floorDiv(nanoseconds, NANOS_PER_DAY)
  const withinDay = nanoseconds - day * NANOS_PER_DAY
  const civil = civilFromDays(day)
  const secondOfDay = withinDay / NANOS_PER_SECOND
  const nanosecond = Number(withinDay % NANOS_PER_SECOND)
  return localDateTimeFromFields({
    ...civil,
    hour: Number(secondOfDay / 3600n),
    minute: Number((secondOfDay % 3600n) / 60n),
    second: Number(secondOfDay % 60n),
    nanosecond,
  })
}

function daysFromCivil(year: number, month: number, day: number): bigint {
  let adjustedYear = BigInt(year)
  if (month <= 2) adjustedYear -= 1n
  const era = floorDiv(adjustedYear, 400n)
  const yearOfEra = adjustedYear - era * 400n
  const adjustedMonth = BigInt(month + (month > 2 ? -3 : 9))
  const dayOfYear = (153n * adjustedMonth + 2n) / 5n + BigInt(day - 1)
  const dayOfEra =
    yearOfEra * 365n + yearOfEra / 4n - yearOfEra / 100n + dayOfYear
  return era * 146097n + dayOfEra - 719468n
}

function civilFromDays(days: bigint): DateFields {
  const shifted = days + 719468n
  const era = floorDiv(shifted, 146097n)
  const dayOfEra = shifted - era * 146097n
  const yearOfEra =
    (dayOfEra - dayOfEra / 1460n + dayOfEra / 36524n - dayOfEra / 146096n) /
    365n
  let year = yearOfEra + era * 400n
  const dayOfYear =
    dayOfEra - (365n * yearOfEra + yearOfEra / 4n - yearOfEra / 100n)
  const monthPrime = (5n * dayOfYear + 2n) / 153n
  const day = dayOfYear - (153n * monthPrime + 2n) / 5n + 1n
  const month = monthPrime + (monthPrime < 10n ? 3n : -9n)
  if (month <= 2n) year += 1n
  const numericYear = Number(year)
  if (!Number.isSafeInteger(numericYear)) {
    throw new RangeError("Instant calendar year is outside the Int range")
  }
  return { year: numericYear, month: Number(month), day: Number(day) }
}

function floorDiv(value: bigint, divisor: bigint): bigint {
  const quotient = value / divisor
  const remainder = value % divisor
  return remainder < 0n ? quotient - 1n : quotient
}

function modulo(value: number, divisor: number): number {
  return ((value % divisor) + divisor) % divisor
}

function validDate(year: number, month: number, day: number): boolean {
  if (![year, month, day].every(Number.isSafeInteger)) return false
  if (month < 1 || month > 12 || day < 1) return false
  const lengths = [
    31,
    leapYear(year) ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ]
  return day <= (lengths[month - 1] ?? 0)
}

function leapYear(year: number): boolean {
  return (
    modulo(year, 4) === 0 &&
    (modulo(year, 100) !== 0 || modulo(year, 400) === 0)
  )
}

function makeDate(fields: DateFields): LocalDate {
  const value = Object.freeze({}) as LocalDate
  dates.set(value, Object.freeze(fields))
  return value
}

function makeTime(fields: TimeFields): LocalTime {
  const value = Object.freeze({}) as LocalTime
  times.set(value, Object.freeze(fields))
  return value
}

function makeOffset(seconds: number): UtcOffset {
  const value = Object.freeze({}) as UtcOffset
  offsets.set(value, seconds)
  return value
}

function makeOffsetDateTime(
  instant: Instant,
  offset: UtcOffset
): OffsetDateTime {
  const value = Object.freeze({}) as OffsetDateTime
  offsetDateTimes.set(value, Object.freeze({ instant, offset }))
  return value
}

function makeZonedDateTime(
  instant: Instant,
  local: LocalDateTime,
  offset: UtcOffset,
  zone: TimeZone
): ZonedDateTime {
  const value = Object.freeze({}) as ZonedDateTime
  zonedDateTimes.set(value, Object.freeze({ instant, local, offset, zone }))
  return value
}

function dateFields(value: LocalDate): DateFields {
  const fields = dates.get(value)
  if (fields === undefined) throw new TypeError("invalid LocalDate value")
  return fields
}

function timeFields(value: LocalTime): TimeFields {
  const fields = times.get(value)
  if (fields === undefined) throw new TypeError("invalid LocalTime value")
  return fields
}

function localFields(value: LocalDateTime) {
  const fields = localDateTimes.get(value)
  if (fields === undefined) throw new TypeError("invalid LocalDateTime value")
  return fields
}

function offsetSeconds(value: UtcOffset): number {
  const seconds = offsets.get(value)
  if (seconds === undefined) throw new TypeError("invalid UtcOffset value")
  return seconds
}

function offsetDateTimeFields(value: OffsetDateTime): OffsetDateTimeFields {
  const fields = offsetDateTimes.get(value)
  if (fields === undefined) throw new TypeError("invalid OffsetDateTime value")
  return fields
}

function timeZoneFields(value: TimeZone): TimeZoneFields {
  const fields = timeZones.get(value)
  if (fields === undefined) throw new TypeError("invalid TimeZone value")
  return fields
}

function zonedFields(value: ZonedDateTime): ZonedDateTimeFields {
  const fields = zonedDateTimes.get(value)
  if (fields === undefined) throw new TypeError("invalid ZonedDateTime value")
  return fields
}

function formatYear(year: number): string {
  if (year >= 0 && year <= 9999) return String(year).padStart(4, "0")
  const sign = year < 0 ? "-" : "+"
  return `${sign}${String(Math.abs(year)).padStart(6, "0")}`
}

function formatOffset(seconds: number): string {
  if (seconds === 0) return "Z"
  const sign = seconds < 0 ? "-" : "+"
  const absolute = Math.abs(seconds)
  return `${sign}${two(Math.floor(absolute / 3600))}:${two(
    Math.floor((absolute % 3600) / 60)
  )}`
}

function two(value: number): string {
  return String(value).padStart(2, "0")
}

function textFailure<Success>(
  text: string,
  offset: number
): Either<DateTimeError, Success> {
  const prefix = text.slice(0, Math.max(0, Math.min(offset, text.length)))
  return Left(
    InvalidDateTimeText({ offset: new TextEncoder().encode(prefix).length })
  )
}

function shiftTextFailure(
  result: LeftValue<DateTimeError>,
  prefix: string
): LeftValue<DateTimeError> {
  if (result.tag === "Left" && result.value.tag === "InvalidDateTimeText") {
    return Left(
      InvalidDateTimeText({
        offset:
          new TextEncoder().encode(prefix).length + result.value.value.offset,
      })
    )
  }
  return result
}

function localDateShapeOffset(text: string): number {
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index] as string
    if ((index === 4 || index === 7) && character !== "-") return index
  }
  return text.length
}

function localTimeShapeOffset(text: string): number {
  for (const index of [2, 5]) {
    if (text[index] !== ":") return Math.min(index, text.length)
  }
  return text.length
}
