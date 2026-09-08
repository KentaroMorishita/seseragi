import { describe, expect, test } from "bun:test"
import { readFile } from "node:fs/promises"
import path from "node:path"
import { repositoryRoot } from "./release-contract"

const workflow = (name: string) =>
  readFile(path.join(repositoryRoot, ".github", "workflows", name), "utf8")

describe("Promotion workflow split", () => {
  test("preflight runs the exact gate without publishing", async () => {
    const source = await workflow("promotion-preflight.yml")

    expect(source).toContain("github.event.comment.body == '/preflight'")
    expect(source).toContain("run: bun run check")
    expect(source).toContain("uses: actions/upload-artifact@v4")
    expect(source).not.toContain("git push origin")
    expect(source).not.toContain("is not Promotion frontier")
  })

  test("publish requires the frontier and matching preflight evidence", async () => {
    const source = await workflow("promotion-prepare.yml")
    const frontier = source.indexOf("is not Promotion frontier")
    const download = source.indexOf("uses: actions/download-artifact@v4")
    const sourceDelta = source.indexOf(
      "Verify tested source and build semantics are unchanged",
    )
    const candidateDelta = source.indexOf(
      "Verify final candidate differs only by mechanical release output",
    )
    const publish = source.indexOf("git push origin")

    expect(source).toContain("github.event.comment.body == '/promote'")
    expect(source).not.toContain("run: bun run check")
    expect(source).toContain("no successful, unexpired /preflight artifact")
    expect(frontier).toBeGreaterThan(-1)
    expect(download).toBeGreaterThan(frontier)
    expect(sourceDelta).toBeGreaterThan(download)
    expect(candidateDelta).toBeGreaterThan(sourceDelta)
    expect(publish).toBeGreaterThan(candidateDelta)
  })
})
