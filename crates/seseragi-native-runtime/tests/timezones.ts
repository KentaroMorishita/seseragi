import { readFileSync, writeFileSync } from "node:fs"
import {
  createInstant,
  instantNanoseconds,
} from "../../../runtime/ts/src/clock-value"
import * as time from "../../../runtime/ts/src/time"

const zero = createInstant(0n)
const offset = (value: time.UtcOffset) =>
  time.formatOffsetDateTime(time.atOffset(value, zero))
const zoned = (value: time.ZonedDateTime) =>
  [
    instantNanoseconds(time.zonedInstant(value)),
    time.formatLocalDateTime(time.zonedLocalDateTime(value)),
    offset(time.zonedOffset(value)),
    time.timeZoneId(time.zonedTimeZone(value)),
    time.timeZoneVersion(time.zonedTimeZone(value)),
  ].join("|")

const lines = readFileSync(process.argv[2]!, "utf8").trimEnd().split("\n")
const results = lines.map((line) => {
  const [mode, id, value] = line.split("\t")
  const zone = time.createTimeZone(id!, "2025b")
  if (mode === "instant")
    return zoned(time.atTimeZone(createInstant(BigInt(value!)), zone))
  const parsed = time.parseLocalDateTime(value!)
  if (parsed.tag === "Left") throw new Error(JSON.stringify(parsed))
  const resolved = time.resolveLocal(parsed.value, zone)
  switch (resolved.tag) {
    case "Unique":
      return `U|${zoned(resolved.value)}`
    case "Ambiguous":
      return `A|${zoned(resolved.value.earlier)}|${zoned(resolved.value.later)}`
    case "Gap":
      return `G|${instantNanoseconds(resolved.value.transition)}|${offset(resolved.value.offsetBefore)}|${offset(resolved.value.offsetAfter)}`
  }
})
writeFileSync(process.argv[3]!, `${results.join("\n")}\n`)
