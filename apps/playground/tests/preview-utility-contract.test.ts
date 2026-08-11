import { describe, expect, test } from "bun:test"
import {
  extractHtmlClassTokens,
  extractPreviewUtilityTokens,
  extractSeseragiClassTokens,
  previewUtilityTokens,
  validatePreviewSourceReadability,
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

  test("collects direct class, cx and expected HTML tokens", () => {
    const source = `
      let label = "日本語 class: missing-string-class"
      // html.div { class: "missing-comment-class" }
      let cardClass = cx ["rounded-xl", "bg-white"]
      let view = html.div {
        class: cardClass,
        style: html.style [("padding", "24px")],
        children: html.p {
          class: "text-sm text-slate-700",
          children: "Preview"
        }
      }
    `
    expect(extractSeseragiClassTokens(source)).toEqual({
      tokens: ["bg-white", "rounded-xl", "text-slate-700", "text-sm"],
      dynamicClasses: [],
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
              content: 'html.main { class: "bg-missing" }',
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
      content: "html.main { class: selectedClass }",
      format: "seseragi" as const,
    }
    expect(() =>
      validatePreviewUtilityUsage([
        { id: "dynamic-preview", sources: [dynamicSource] },
      ])
    ).toThrow("has dynamic class")

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

  test("requires long class literals to use a cx value", () => {
    expect(() =>
      validatePreviewSourceReadability([
        {
          id: "wide-card",
          sources: [
            {
              path: "main.ssrg",
              content:
                'html.main { class: "rounded-xl bg-white p-6 shadow-lg text-slate-900" }',
              format: "seseragi",
            },
          ],
        },
      ])
    ).toThrow("5-token class literal")

    expect(() =>
      validatePreviewSourceReadability([
        {
          id: "readable-card",
          sources: [
            {
              path: "main.ssrg",
              content: `fn cx classes: Array<String> -> String =
  join " " classes

let cardClass = cx ["rounded-xl", "bg-white", "p-6", "shadow-lg", "text-slate-900"]

let view = html.main { class: cardClass, children: "Preview" }
`,
              format: "seseragi",
            },
          ],
        },
      ])
    ).not.toThrow()
  })

  test("allows compact cx lists and rejects undiscoverable project helpers", () => {
    expect(() =>
      validatePreviewSourceReadability([
        {
          id: "compressed",
          sources: [
            {
              path: "main.ssrg",
              content: `fn cx classes: Array<String> -> String =
  join " " classes
let cardClass = cx ["rounded-xl", "bg-white", "p-6", "shadow-lg", "text-slate-900"]
`,
              format: "seseragi",
            },
          ],
        },
      ])
    ).not.toThrow()

    expect(() =>
      validatePreviewSourceReadability([
        {
          id: "project",
          sources: [
            {
              path: "main.ssrg",
              content: `import { cx } from "./helpers"
let cardClass = cx [
  "rounded-xl",
  "bg-white",
  "p-6",
  "shadow-lg",
  "text-slate-900"
]
`,
              format: "seseragi",
            },
            {
              path: "helpers.ssrg",
              content:
                'pub fn cx classes: Array<String> -> String = join " " classes\n',
              format: "seseragi",
            },
          ],
        },
      ])
    ).toThrow("workspace styles.ssrg")
  })
})
