import { describe, expect, test } from "bun:test"

const extensionRoot = new URL("..", import.meta.url)

describe("extension README brand contract", () => {
  test("renders the canonical SVG in the repository", async () => {
    const readme = await Bun.file(new URL("README.md", extensionRoot)).text()

    expect(readme).toContain(
      'src="../../assets/brand/source/seseragi-icon.svg"'
    )
    expect(readme).not.toContain('src="./images/icon.png"')
  })

  test("omits the duplicate hero from the transient VSIX package input", async () => {
    const packager = await Bun.file(
      new URL("scripts/package-extension.ts", extensionRoot)
    ).text()

    expect(packager).toContain(
      'const repositoryBrand = `<p align="center">'
    )
    expect(packager).toContain('return source.replace(repositoryBrand, "")')
    expect(packager).toContain("writeFileSync(readmePath, repositoryReadme)")
  })
})
