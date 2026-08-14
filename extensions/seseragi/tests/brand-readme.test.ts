import { describe, expect, test } from "bun:test"
import { packagedReadme } from "../scripts/package-readme"

const extensionRoot = new URL("..", import.meta.url)

describe("extension README brand contract", () => {
  test("renders the canonical SVG in the repository", async () => {
    const readme = await Bun.file(new URL("README.md", extensionRoot)).text()

    expect(readme).toContain(
      'src="../../assets/brand/source/seseragi-icon.svg"'
    )
    expect(readme).not.toContain('src="./images/icon.png"')
  })

  test("omits the duplicate hero from LF and CRLF package input", async () => {
    const readme = await Bun.file(new URL("README.md", extensionRoot)).text()

    expect(packagedReadme(readme)).not.toContain(
      'src="../../assets/brand/source/seseragi-icon.svg"'
    )
    const crlfReadme = readme.replaceAll("\r\n", "\n").replaceAll("\n", "\r\n")
    expect(packagedReadme(crlfReadme)).not.toContain(
      'src="../../assets/brand/source/seseragi-icon.svg"'
    )
  })

  test("rejects package input without the repository brand", () => {
    expect(() => packagedReadme("# Seseragi\n")).toThrow(
      "extension README is missing the repository brand block"
    )
  })

  test("restores the repository README after packaging", async () => {
    const packager = await Bun.file(
      new URL("scripts/package-extension.ts", extensionRoot)
    ).text()

    expect(packager).toContain("writeFileSync(readmePath, repositoryReadme)")
  })
})
