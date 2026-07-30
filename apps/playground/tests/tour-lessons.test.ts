import { describe, expect, test } from "bun:test"
import {
  parseTourLessonMetadata,
  validateTourLessonFormat,
} from "../../../scripts/tour-lessons"

const tourRoot = new URL("../../../examples/tour/lessons/", import.meta.url)

describe("Tour lesson format", () => {
  test("loads every required structured section and validates source ranges", async () => {
    const descriptor = await readDescriptor("01-hello-world")
    const source = await Bun.file(
      new URL("01-hello-world/main.ssrg", tourRoot)
    ).text()
    const metadata = parseTourLessonMetadata(descriptor, "01-hello-world")

    expect(metadata.formatVersion).toBe(2)
    expect(metadata.files.guide).toBeUndefined()
    expect(metadata.files.exercise).toBe("exercise.ssrg")
    expect(metadata.files.exerciseExpectedOutput).toBe("exercise.stdout.txt")
    expect(metadata.files.diagnosticExample).toBe("diagnostic.ssrg")
    expect(metadata.files.diagnosticOutput).toBe("diagnostic.txt")
    expect(metadata.format?.walkthrough.length).toBeGreaterThan(0)
    expect(metadata.format?.introduced.length).toBeGreaterThan(0)
    expect(metadata.format?.recap.length).toBeGreaterThan(0)
    if (metadata.format === undefined) {
      throw new Error("expected structured lesson format")
    }
    const format = metadata.format
    expect(() =>
      validateTourLessonFormat(metadata.id, format, source)
    ).not.toThrow()
  })

  test("rejects a missing required section or required artifact reference", async () => {
    const missingRecap = await readDescriptor("01-hello-world")
    delete record(missingRecap.sections).recap
    expect(() =>
      parseTourLessonMetadata(missingRecap, "01-hello-world")
    ).toThrow("sections.recap")

    const missingDiagnostic = await readDescriptor("01-hello-world")
    delete record(missingDiagnostic.files).diagnosticOutput
    expect(() =>
      parseTourLessonMetadata(missingDiagnostic, "01-hello-world")
    ).toThrow("needs files.diagnosticOutput")
  })

  test("keeps notes optional and rejects invalid walkthrough ranges", async () => {
    const withoutNotes = await readDescriptor("01-hello-world")
    delete record(withoutNotes.sections).notes
    const parsed = parseTourLessonMetadata(withoutNotes, "01-hello-world")
    expect(parsed.format?.notes).toBeUndefined()

    const invalidRange = await readDescriptor("01-hello-world")
    const walkthrough = record(invalidRange.sections).walkthrough
    if (!Array.isArray(walkthrough)) {
      throw new Error("expected walkthrough array")
    }
    record(record(walkthrough[0]).sourceRange).endLine = 20
    const invalid = parseTourLessonMetadata(invalidRange, "01-hello-world")
    if (invalid.format === undefined) {
      throw new Error("expected structured lesson format")
    }
    const invalidFormat = invalid.format
    expect(() =>
      validateTourLessonFormat(
        invalid.id,
        invalidFormat,
        'pub effect fn main = println "ok"\n'
      )
    ).toThrow("references line 20")
  })

  test("preserves legacy lesson descriptors until their delivery issues migrate them", async () => {
    const legacy = parseTourLessonMetadata(
      await readDescriptor("06-records-and-structs"),
      "06-records-and-structs"
    )

    expect(legacy.format).toBeUndefined()
    expect(legacy.files.guide).toBe("guide.md")
    expect(legacy.challenge?.trim()).not.toBe("")
  })
})

async function readDescriptor(id: string): Promise<Record<string, unknown>> {
  return JSON.parse(
    await Bun.file(new URL(`${id}/lesson.json`, tourRoot)).text()
  ) as Record<string, unknown>
}

function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("expected record")
  }
  return value as Record<string, unknown>
}
