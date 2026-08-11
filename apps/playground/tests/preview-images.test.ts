import { describe, expect, test } from "bun:test"
import {
  createPreviewDocument,
  previewUtilityCss,
} from "../src/preview-document"
import { samples } from "../src/samples"

const workspaceImage =
  "https://images.unsplash.com/photo-1497366811353-6870744d04b2?fit=crop&w=960&h=480&q=80"
const plannerImage =
  "https://images.unsplash.com/photo-1484480974693-6ca0a78fb36b?fit=crop&w=720&h=360&q=80"

describe("Playground remote sample images", () => {
  test("uses fixed HTTPS photo ids with stable accessible layout", () => {
    const component = samples.find((sample) => sample.id === "html-components")
    const todo = samples.find((sample) => sample.id === "form-todo")

    expect(component?.source).toContain(`parseSampleUrl "${workspaceImage}"`)
    expect(todo?.source).toContain(`parseSampleUrl "${plannerImage}"`)
    for (const source of [component?.source, todo?.source]) {
      for (const utility of [
        "aspect-2-1",
        "h-auto",
        "w-full",
        "rounded-xl",
        "object-cover",
      ]) {
        expect(source).toContain(`"${utility}"`)
      }
      expect(source).toContain('loading: "eager"')
      expect(source).not.toContain("/seseragi-mark.png")
      expect(source).not.toContain("source.unsplash.com")
      expect(source).not.toContain("/random")
    }
    expect(component?.source).toContain(
      'alt: "大きな窓とテーブルのある明るい共同作業スペース"'
    )
    expect(component?.source).toContain("class: heroImageClass")
    expect(todo?.source).toContain('alt: "手帳へ次の一歩を書き込む手元"')
    expect(todo?.source).toContain("class: heroImageClass")
    expect(component?.source).toMatch(/width: 960,\n\s+height: 480/u)
    expect(todo?.source).toMatch(/width: 720,\n\s+height: 360/u)
    expect(previewUtilityCss).toContain(".aspect-2-1")
    expect(previewUtilityCss).toContain(".h-auto")
    expect(previewUtilityCss).toContain(".object-cover")
  })

  test("keeps meaningful fallback text inside the HTTPS image policy", () => {
    const document = createPreviewDocument(
      '<img class="aspect-2-1 h-auto w-full object-cover" src="https://images.unsplash.com/missing-photo" alt="画像を取得できない場合の共同作業スペース" width="960" height="480">'
    )

    expect(document).toContain('alt="画像を取得できない場合の共同作業スペース"')
    expect(document).toContain('width="960" height="480"')
    expect(document).toContain('class="aspect-2-1 h-auto w-full object-cover"')
    expect(document).toContain("img-src 'self' https: data: blob:")
    expect(document).not.toContain("http:")
    expect(previewUtilityCss).toContain("aspect-ratio: 2 / 1")
  })
})
