import { describe, expect, test } from "bun:test"
import {
  createPreviewDocument,
  previewUtilityCss,
} from "../src/preview-document"
import { samples } from "../src/samples"

describe("Playground remote sample images", () => {
  test("draws the ticket booking showcase without an external image (#1371)", () => {
    const booking = samples.find((sample) => sample.id === "form-todo")
    const allSources =
      booking?.workspace.files.map(({ source }) => source).join("\n") ?? ""

    expect(allSources).not.toContain("images.unsplash.com")
    expect(allSources).not.toContain("html.img {")
    expect(allSources).toContain("svg.toHtml")
    expect(previewUtilityCss).toContain(".aspect-2-1")
    expect(previewUtilityCss).toContain(".h-auto")
    expect(previewUtilityCss).toContain(".object-cover")
  })

  test("draws the component sample without an external image (#1371)", () => {
    const component = samples.find((sample) => sample.id === "html-components")

    expect(component?.source).not.toContain("images.unsplash.com")
    expect(component?.source).not.toContain("html.img {")
    expect(component?.source).toContain("svg.toHtml")
    expect(component?.source).toContain("ariaLabel: `${book.title}の表紙`")
  })

  test("keeps meaningful fallback text inside the HTTPS image policy", () => {
    const document = createPreviewDocument(
      '<img class="aspect-2-1 h-auto w-full object-cover" src="https://images.unsplash.com/missing-photo" alt="画像を取得できない場合の共同作業スペース" width="960" height="480">',
      true
    )

    expect(document).toContain('alt="画像を取得できない場合の共同作業スペース"')
    expect(document).toContain('width="960" height="480"')
    expect(document).toContain('class="aspect-2-1 h-auto w-full object-cover"')
    expect(document).toContain("img-src 'self' https: data: blob:")
    expect(document).not.toContain("http:")
    expect(previewUtilityCss).toContain("aspect-ratio: 2 / 1")
  })
})
