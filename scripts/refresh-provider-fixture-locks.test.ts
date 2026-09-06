import { describe, expect, test } from "bun:test"
import path from "node:path"
import {
  providerFixtureDirectories,
  repositoryRoot,
} from "./refresh-provider-fixture-locks"

describe("provider fixture lock refresh", () => {
  test("selects current provider-backed fixtures without rewriting stale-lock evidence", async () => {
    const fixtures = (await providerFixtureDirectories()).map((directory) =>
      path.relative(repositoryRoot, directory)
    )

    expect(fixtures).toContain(
      "examples/spec/fixtures/projects/postgres-application"
    )
    expect(fixtures).toContain(
      "examples/spec/fixtures/projects/sqlite-application"
    )
    expect(fixtures).toContain(
      "examples/spec/fixtures/projects/provider-http-client-e2e"
    )
    expect(fixtures).not.toContain(
      "examples/spec/fixtures/projects/package-stale-lock"
    )
    expect(fixtures).not.toContain(
      "examples/spec/fixtures/projects/http-stream-events"
    )
  })
})
