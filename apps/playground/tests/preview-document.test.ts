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
    expect(document).toContain(".min-h-screen")
    expect(document).toContain(".sm\\:p-12")
    expect(document).toContain('class="min-h-screen bg-emerald-50 p-8 sm:p-12"')
    expect(document).not.toContain("<script")
  })

  test("keeps the utility vocabulary bounded and host-owned", () => {
    expect(previewUtilityCss).toContain(".grid-cols-2")
    expect(previewUtilityCss).toContain(".hover\\:bg-emerald-600:hover")
    expect(previewUtilityCss).not.toContain("@import")
  })
})
