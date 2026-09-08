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
      "Verify tested source and build semantics are unchanged"
    )
    const candidateDelta = source.indexOf(
      "Verify final candidate differs only by mechanical release output"
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

test("terminal publication rejects incomplete or reordered implementation suffixes", async () => {
  const source = await workflow("promotion-prepare.yml")
  const script = source.slice(
    source.indexOf("            const promotionLines"),
    source.indexOf("            const entryPattern")
  )
  const execute = new (Object.getPrototypeOf(async () => {}).constructor)(
    "promotion",
    "impl",
    "leafNumber",
    "queueMatches",
    "getIssue",
    "names",
    "oid",
    "core",
    script
  )
  const promotion = {
    body: [1, 2]
      .map((n) => `- [ ] #${n} — \`feat/${n}\` @ \`${String(n).repeat(40)}\``)
      .join("\n"),
  }
  for (const [body, shouldPass] of [
    ["- [x] #1\n- [x] #2", true],
    ["- [x] #1\n- [x] #2\n- [ ] #3", false],
    ["- [x] #2\n- [x] #1", false],
    ["- [x] #1\n- [ ] #2", false],
  ] as const) {
    const errors: string[] = []
    const outputs: Record<string, string> = {}
    await execute(
      promotion,
      { body },
      2,
      (text: string, number: number) =>
        [...text.matchAll(new RegExp(`^- \\[([ x])\\] #${number}$`, "gm"))].map(
          (m) => m[1] === "x"
        ),
      async () => ({ labels: ["stage: promotion-ready", "O01"] }),
      (item: { labels: string[] }) => item.labels,
      "O01",
      {
        setFailed: (message: string) => errors.push(message),
        setOutput: (key: string, value: string) => {
          outputs[key] = value
        },
      }
    )
    expect(errors.length === 0).toBe(shouldPass)
    if (shouldPass)
      expect(
        JSON.parse(outputs.covered_checkpoints).map(
          (item: { leaf: number }) => item.leaf
        )
      ).toEqual([1, 2])
  }
})
