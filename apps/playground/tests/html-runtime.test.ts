import { describe, expect, test } from "bun:test"
import {
  button,
  div,
  renderForDom,
  renderToString,
  style,
} from "../../../runtime/ts/src/html"

describe("HTML browser runtime", () => {
  test("serializes record styles and CSS variables with escaping", () => {
    const node = div({
      style: style({
        variables: { cardShadow: '0 4px 16px "#0002"' },
        backgroundColor: "#fff",
        boxShadow: "var(--card-shadow)",
      }),
      children: "Styled",
    })

    expect(renderToString(node)).toBe(
      '<div style="--card-shadow: 0 4px 16px &quot;#0002&quot;; background-color: #fff; box-shadow: var(--card-shadow)">Styled</div>'
    )
  })

  test("keeps click messages out of SSR and exposes them to the DOM adapter", () => {
    const message = { tag: "Increment" } as const
    const node = button({ onClick: message, children: "+1" })

    expect(renderToString(node)).toBe('<button type="button">+1</button>')
    const rendered = renderForDom(node)
    expect(rendered.html).toBe(
      '<button data-ssrg-click="0" type="button">+1</button>'
    )
    expect(rendered.clickMessages.get("0")).toBe(message)
  })
})
