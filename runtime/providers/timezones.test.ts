import { describe, expect, test } from "bun:test"
import {
  createTimeZone,
  formatOffsetDateTime,
  parseLocalDateTime,
  parseOffsetDateTime,
  resolveLocal,
} from "../ts/src/time"
import { createTimeZonesProvider } from "./timezones"

describe("pinned timezone database", () => {
  test("reports canonical aliases and the exact 2025b release", async () => {
    const provider = createTimeZonesProvider()
    expect(await provider.databaseVersion?.(undefined)).toEqual({
      kind: "success",
      value: "2025b",
    })
    expect(await provider.loadTimeZone?.("US/Eastern")).toEqual({
      kind: "success",
      value: { id: "America/New_York", version: "2025b" },
    })
  })

  test("returns a typed version mismatch without consulting host tzdb", async () => {
    const provider = createTimeZonesProvider({
      version: () => "2025a",
      canonicalize: (id) => id,
    })
    expect(await provider.loadTimeZone?.("Europe/Berlin")).toEqual({
      kind: "failure",
      failure: {
        tag: "TimeZoneDatabaseVersionMismatch",
        value: { required: "2025b", actual: "2025a" },
      },
    })
  })

  test("keeps DST gaps and overlaps explicit", () => {
    const berlin = createTimeZone("Europe/Berlin", "2025b")
    const gap = parseLocalDateTime("2024-03-31T02:30:00")
    const overlap = parseLocalDateTime("2024-10-27T02:30:00")
    expect(gap.tag).toBe("Right")
    expect(overlap.tag).toBe("Right")
    if (gap.tag === "Right")
      expect(resolveLocal(gap.value, berlin).tag).toBe("Gap")
    if (overlap.tag === "Right") {
      expect(resolveLocal(overlap.value, berlin).tag).toBe("Ambiguous")
    }
  })

  test("round-trips expanded years, nanoseconds, offsets, and pre-epoch instants", () => {
    for (const text of [
      "1969-12-31T23:59:59.999999999Z",
      "+012345-06-07T08:09:10.120+18:00",
      "-000001-01-01T00:00:00-18:00",
    ]) {
      const parsed = parseOffsetDateTime(text)
      expect(parsed.tag).toBe("Right")
      if (parsed.tag === "Right") {
        expect(formatOffsetDateTime(parsed.value)).toBe(
          text.replace(".120", ".12")
        )
      }
    }
  })

  test("reports date-time syntax errors as whole-input UTF-8 byte offsets", () => {
    expect(parseLocalDateTime("2024-01-01T12-30:00")).toEqual({
      tag: "Left",
      value: {
        tag: "InvalidDateTimeText",
        value: { offset: 13 },
      },
    })
    expect(parseLocalDateTime("2024-01-01T12:30:00あ")).toEqual({
      tag: "Left",
      value: {
        tag: "InvalidDateTimeText",
        value: { offset: 22 },
      },
    })
  })
})
