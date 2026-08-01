import { describe, expect, test } from "bun:test"
import {
  extractHtmlClassTokens,
  extractPreviewUtilityTokens,
  extractSeseragiClassTokens,
  previewUtilityTokens,
  validatePreviewUtilityUsage,
} from "../src/preview-utility-contract"

describe("Preview utility contract", () => {
  test("derives responsive, pseudo and arbitrary tokens from CSS selectors", () => {
    expect(
      extractPreviewUtilityTokens(String.raw`
        .plain { display: block; }
        .hover\:bg-emerald-600:hover { background: green; }
        @media (min-width: 640px) {
          .sm\:p-10 { padding: 2.5rem; }
          .grid-cols-\[1fr_auto\] { grid-template-columns: 1fr auto; }
        }
      `)
    ).toEqual([
      "grid-cols-[1fr_auto]",
      "hover:bg-emerald-600",
      "plain",
      "sm:p-10",
    ])
    expect(previewUtilityTokens).toContain("hover:bg-emerald-600")
    expect(previewUtilityTokens).toContain("sm:grid-cols-3")
  })

  test("collects direct className, cx and expected HTML tokens", () => {
    const source = `
      let label = "日本語 className: missing-string-class"
      // html.div { className: "missing-comment-class" }
      let cardClass = cx ["rounded-xl", "bg-white"]
      let view = html.div {
        className: cardClass,
        style: html.style [("padding", "24px")],
        children: html.p {
          className: "text-sm text-slate-700",
          children: "Preview"
        }
      }
    `
    expect(extractSeseragiClassTokens(source)).toEqual({
      tokens: ["bg-white", "rounded-xl", "text-slate-700", "text-sm"],
      dynamicClassNames: [],
    })
    expect(
      extractHtmlClassTokens(
        '<main class="p-6 custom-hook" style="color: red"></main>'
      )
    ).toEqual(["custom-hook", "p-6"])
  })

  test("rejects an undefined utility with sample, file and token", () => {
    expect(() =>
      validatePreviewUtilityUsage([
        {
          id: "broken-preview",
          sources: [
            {
              path: "main.ssrg",
              content: 'html.main { className: "bg-missing" }',
              format: "seseragi",
            },
          ],
        },
      ])
    ).toThrow(
      'sample broken-preview file main.ssrg uses undefined utility "bg-missing"'
    )
  })

  test("requires dynamic classes and semantic custom classes to be explicit", () => {
    const dynamicSource = {
      path: "main.ssrg",
      content: "html.main { className: selectedClass }",
      format: "seseragi" as const,
    }
    expect(() =>
      validatePreviewUtilityUsage([
        { id: "dynamic-preview", sources: [dynamicSource] },
      ])
    ).toThrow("has dynamic className")

    expect(() =>
      validatePreviewUtilityUsage([
        {
          id: "declared-preview",
          sources: [dynamicSource],
          dynamicUtilities: ["bg-white"],
          customClasses: ["sample-hook"],
        },
      ])
    ).not.toThrow()
  })
})
