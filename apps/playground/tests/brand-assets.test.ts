import { describe, expect, test } from "bun:test"

const playgroundRoot = new URL("..", import.meta.url)
const repositoryRoot = new URL("../../..", import.meta.url)
const canonicalBrand = new URL("assets/brand/", repositoryRoot)
const publicBrand = new URL("public/brand/", canonicalBrand)

const pngSignature = [137, 80, 78, 71, 13, 10, 26, 10]

const bytes = async (url: URL): Promise<Uint8Array> =>
  new Uint8Array(await Bun.file(url).arrayBuffer())

const pngDimensions = async (url: URL): Promise<[number, number]> => {
  const content = await bytes(url)
  expect([...content.slice(0, 8)]).toEqual(pngSignature)
  const view = new DataView(content.buffer, content.byteOffset, content.byteLength)
  return [view.getUint32(16), view.getUint32(20)]
}

const sha256 = async (url: URL): Promise<string> =>
  new Bun.CryptoHasher("sha256").update(await bytes(url)).digest("hex")

describe("Playground brand asset contract", () => {
  test("uses the canonical icon SVG without a surface-specific redesign", async () => {
    const canonical = await Bun.file(
      new URL("source/seseragi-icon.svg", canonicalBrand)
    ).text()
    const distributed = await Bun.file(
      new URL("seseragi-icon.svg", publicBrand)
    ).text()

    expect(distributed).toBe(canonical)
  })

  test("ships a self-contained 1200x630 social preview", async () => {
    const source = await Bun.file(
      new URL("social/seseragi-social-preview.svg", canonicalBrand)
    ).text()

    expect(source).not.toContain("<image")
    expect(source).not.toContain("../source/")
    expect(source).toContain('viewBox="300 420 1400 420"')
    expect(
      await pngDimensions(new URL("seseragi-social-preview.png", publicBrand))
    ).toEqual([1200, 630])
    expect(
      await sha256(new URL("seseragi-social-preview.png", publicBrand))
    ).toBe("3c8da9d2cb6b5827d12300fbd148270242ad494f96fbd110dc169f8b22327c33")
  })

  test("ships the required browser and install icon sizes", async () => {
    const expected = new Map<string, [number, number]>([
      ["favicon-16x16.png", [16, 16]],
      ["favicon-32x32.png", [32, 32]],
      ["favicon-48x48.png", [48, 48]],
      ["apple-touch-icon.png", [180, 180]],
    ])

    for (const [filename, dimensions] of expected) {
      expect(await pngDimensions(new URL(filename, publicBrand))).toEqual(
        dimensions
      )
    }

    expect(await sha256(new URL("apple-touch-icon.png", publicBrand))).toBe(
      "8cb5daf56eec1cb73790722b7ca3c1bd40eef73ea7412c4464632f3bba88f1f4"
    )

    const ico = await bytes(new URL("favicon.ico", publicBrand))
    const view = new DataView(ico.buffer, ico.byteOffset, ico.byteLength)
    expect(view.getUint16(0, true)).toBe(0)
    expect(view.getUint16(2, true)).toBe(1)
    expect(view.getUint16(4, true)).toBe(3)
  })

  test("injects shared favicon, manifest, social metadata, and header branding", async () => {
    const vite = await Bun.file(new URL("vite.config.ts", playgroundRoot)).text()
    const css = await Bun.file(new URL("brand.css", publicBrand)).text()
    const manifest = JSON.parse(
      await Bun.file(new URL("site.webmanifest", publicBrand)).text()
    ) as { icons: { src: string; sizes: string }[] }

    expect(vite).toContain('name: "seseragi-brand-surface"')
    expect(vite).toContain('href: "/brand/brand.css"')
    expect(vite).toContain('href: "/brand/seseragi-icon.svg"')
    expect(vite).toContain("seseragi-social-preview.png")
    expect(vite).toContain("summary_large_image")
    expect(vite).toContain('context.filename.endsWith("/tour/index.html")')

    expect(css).toContain(".topbar .brand::before")
    expect(css).toContain(".tour-topbar .tour-brand::before")
    expect(css).toContain("@media (max-width: 430px)")

    expect(manifest.icons).toEqual([
      expect.objectContaining({
        src: "/brand/seseragi-icon.svg",
        sizes: "any",
      }),
    ])
  })
})
