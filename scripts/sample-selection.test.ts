import { describe, expect, test } from "bun:test"
import {
  readSampleRunPlan,
  sampleCheckerFiles,
  selectChangedSamples,
} from "./sample-selection"

const ids = ["fizzbuzz", "hello-world", "web-starter"]

describe("selectChangedSamples", () => {
  test("picks the samples whose directory changed", () => {
    expect(
      selectChangedSamples(
        [
          "examples/samples/hello-world/main.ssrg",
          "examples/samples/hello-world/stdout.txt",
          "examples/samples/web-starter/src/main.ssrg",
          "crates/seseragi-cli/src/main.rs",
        ],
        ids
      )
    ).toEqual(["hello-world", "web-starter"])
  })

  test("ignores files at the top of examples/samples and removed samples", () => {
    expect(
      selectChangedSamples(
        [
          "examples/samples/discover-groups.json",
          "examples/samples/README.md",
          "examples/samples/removed-sample/main.ssrg",
        ],
        ids
      )
    ).toEqual([])
  })

  test("accepts Windows separators", () => {
    expect(
      selectChangedSamples(["examples\\samples\\fizzbuzz\\main.ssrg"], ids)
    ).toEqual(["fizzbuzz"])
  })

  test("runs every sample when the checker itself changed", () => {
    for (const file of sampleCheckerFiles) {
      expect(selectChangedSamples([file], ids)).toEqual(ids)
    }
  })
})

describe("readSampleRunPlan", () => {
  test("reads only what running a sample needs", () => {
    expect(
      readSampleRunPlan(
        {
          id: "hello-world",
          title: "ignored here",
          files: {
            source: "main.ssrg",
            guide: "guide.md",
            stdin: "stdin.txt",
            expectedOutput: "stdout.txt",
          },
        },
        "hello-world"
      )
    ).toEqual({
      id: "hello-world",
      interactive: false,
      source: "main.ssrg",
      stdin: "stdin.txt",
      expectedOutput: "stdout.txt",
    })
  })

  test("leaves the expected output optional", () => {
    expect(
      readSampleRunPlan({ files: { source: "main.ssrg" } }, "hello-world")
    ).toEqual({ id: "hello-world", interactive: false, source: "main.ssrg" })
  })

  test("keeps the package manifest and the interactive flag", () => {
    expect(
      readSampleRunPlan(
        { interactive: true, files: { manifest: "seseragi.toml" } },
        "web-starter"
      )
    ).toEqual({
      id: "web-starter",
      interactive: true,
      manifest: "seseragi.toml",
    })
  })

  test("refuses a sample.json that names nothing to run", () => {
    expect(() =>
      readSampleRunPlan({ files: { guide: "guide.md" } }, "hello-world")
    ).toThrow("neither files.source nor files.manifest")
    expect(() => readSampleRunPlan({}, "hello-world")).toThrow(
      "files must be an object"
    )
    expect(() => readSampleRunPlan([], "hello-world")).toThrow(
      "must be an object"
    )
  })

  test("refuses malformed fields", () => {
    expect(() =>
      readSampleRunPlan(
        { interactive: "yes", files: { source: "main.ssrg" } },
        "hello-world"
      )
    ).toThrow("interactive must be a boolean")
    expect(() =>
      readSampleRunPlan({ files: { source: "" } }, "hello-world")
    ).toThrow("must be a file name")
    expect(() =>
      readSampleRunPlan({ files: { source: "../other/main.ssrg" } }, "x")
    ).toThrow("inside the sample directory")
    expect(() =>
      readSampleRunPlan({ files: { source: "/tmp/main.ssrg" } }, "x")
    ).toThrow("inside the sample directory")
  })
})
