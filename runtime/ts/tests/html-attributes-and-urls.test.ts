import { describe, expect, test } from "bun:test"
import {
  attribute,
  createSvgElement,
  p,
  parseWebUrl,
  renderToString,
} from "../src/html"

describe("html attributes and URLs", () => {
  test("rejects custom attributes colliding with typed props case-insensitively", () => {
    const aria = attribute("aria-label", "custom")
    expect(aria.tag).toBe("Right")
    if (aria.tag !== "Right") return

    expect(() =>
      renderToString(
        p({
          ariaLabel: "typed",
          attributes: [aria.value],
          children: "x",
        })
      )
    ).toThrow("duplicate HTML custom attribute aria-label")

    expect(
      renderToString(
        p({
          attributes: [aria.value],
          children: "x",
        })
      )
    ).toBe('<p aria-label="custom">x</p>')

    const inert = attribute("INERT", "true")
    expect(inert.tag).toBe("Right")
    if (inert.tag !== "Right") return

    expect(() =>
      renderToString(
        p({
          inert: false,
          attributes: [inert.value],
          children: "x",
        })
      )
    ).toThrow("duplicate HTML custom attribute INERT")

    const viewBox = attribute("VIEWBOX", "0 0 20 20")
    expect(viewBox.tag).toBe("Right")
    if (viewBox.tag !== "Right") return

    expect(() =>
      renderToString(
        createSvgElement("svg", {
          viewBox: "0 0 10 10",
          attributes: [viewBox.value],
          children: "",
        })
      )
    ).toThrow("duplicate HTML custom attribute VIEWBOX")
  })

  test("writes SVG dash and text presentation props as SVG attributes", () => {
    expect(
      renderToString(
        createSvgElement("path", { d: "M0 0", strokeDasharray: "4 2" })
      )
    ).toBe('<path d="M0 0" stroke-dasharray="4 2"></path>')
    expect(
      renderToString(
        createSvgElement("text", {
          fontSize: "7px",
          fontWeight: "700",
          letterSpacing: "0.1em",
          children: "N",
        })
      )
    ).toBe(
      '<text font-size="7px" font-weight="700" letter-spacing="0.1em">N</text>'
    )
  })

  test("accepts internationalized domain URLs and preserves spelling", () => {
    for (const url of [
      "https://bücher.example/a",
      "https://xn--bcher-kva.example/a",
      "https://日本語.jp/path",
    ]) {
      const parsed = parseWebUrl(url)
      expect(parsed.tag).toBe("Right")
      if (parsed.tag === "Right") {
        expect(parsed.value.value).toBe(url)
      }
    }

    const bad = parseWebUrl("https://xn--a.example/a")
    expect(bad.tag).toBe("Left")
  })
})
