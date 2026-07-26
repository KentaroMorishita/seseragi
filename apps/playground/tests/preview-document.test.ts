import { describe, expect, test } from "bun:test"
import {
  createPreviewDocument,
  previewUtilityCss,
} from "../src/preview-document"

describe("Playground preview document", () => {
  test("injects a Tailwind-compatible utility subset without scripts", () => {
    const document = createPreviewDocument(
      '<main class="min-h-screen bg-emerald-50 p-8 sm:p-12"></main>'
    )

    expect(document).toContain("<style>")
    expect(document).toContain("script-src 'none'")
    expect(document).toContain("form-action 'none'")
    expect(document).toContain("default-src 'none'")
    expect(document).toContain(".min-h-screen")
    expect(document).toContain(".sm\\:p-12")
    expect(document).toContain('class="min-h-screen bg-emerald-50 p-8 sm:p-12"')
    expect(document).not.toContain("<script")
  })

  test("keeps the utility vocabulary bounded and host-owned", () => {
    expect(previewUtilityCss).toContain(".grid-cols-2")
    expect(previewUtilityCss).toContain(".sm\\:grid-cols-3")
    expect(previewUtilityCss).toContain(".hover\\:bg-emerald-600:hover")
    expect(previewUtilityCss).not.toContain("@import")
  })

  test("allows bounded image sources without relaxing scripts", () => {
    const document = createPreviewDocument(
      [
        '<img src="/seseragi-mark.png" alt="Local">',
        '<img src="https://images.unsplash.com/photo.jpg" alt="HTTPS">',
        '<img src="data:image/png;base64,AA==" alt="Data">',
        '<img src="blob:https://example.test/id" alt="Blob">',
      ].join("")
    )
    const policy = document.match(
      /http-equiv="Content-Security-Policy" content="([^"]+)"/
    )?.[1]

    expect(policy).toBe(
      "default-src 'none'; base-uri 'none'; form-action 'none'; script-src 'none'; style-src 'unsafe-inline'; img-src 'self' https: data: blob:"
    )
    const imageSources = policy
      ?.split("; ")
      .find((directive) => directive.startsWith("img-src "))
      ?.split(" ")
      .slice(1)
    expect(imageSources).toEqual(["'self'", "https:", "data:", "blob:"])
    expect(imageSources).not.toContain("http:")
    expect(policy).toContain("script-src 'none'")
    expect(document).not.toContain("<script")
  })
})
