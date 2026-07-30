import { describe, expect, test } from "bun:test"

const expectedSandbox = [
  "allow-forms",
  "allow-popups",
  "allow-popups-to-escape-sandbox",
  "allow-same-origin",
  "allow-scripts",
]

function sandboxTokens(html: string, iframeId: string): string[] {
  const iframe = html.match(
    new RegExp(`<iframe[\\s\\S]*?id="${iframeId}"[\\s\\S]*?sandbox="([^"]+)"`)
  )?.[1]
  if (iframe === undefined) {
    throw new Error(`missing sandbox for ${iframeId}`)
  }
  return iframe.split(/\s+/)
}

describe("Playground preview sandbox", () => {
  test("shares the minimal popup permission set across Playground and Tour", async () => {
    const root = new URL("..", import.meta.url)
    const playground = await Bun.file(new URL("index.html", root)).text()
    const tour = await Bun.file(new URL("tour/index.html", root)).text()

    expect(sandboxTokens(playground, "html-preview")).toEqual(expectedSandbox)
    expect(sandboxTokens(tour, "tour-html-preview")).toEqual(expectedSandbox)
    expect(expectedSandbox).not.toContain("allow-top-navigation")
    expect(expectedSandbox).not.toContain(
      "allow-top-navigation-by-user-activation"
    )
    expect(expectedSandbox).not.toContain("allow-modals")
  })

  test("keeps the canonical external link HTTPS and opener-isolated", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/samples/html-components/main.ssrg",
        import.meta.url
      )
    ).text()
    const output = await Bun.file(
      new URL(
        "../../../examples/samples/html-components/stdout.html",
        import.meta.url
      )
    ).text()

    expect(source).toContain(
      'href: "https://github.com/KentaroMorishita/seseragi"'
    )
    expect(source).toContain('target: "_blank"')
    expect(source).toContain('rel: "noopener"')
    expect(output).toContain(
      'href="https://github.com/KentaroMorishita/seseragi" target="_blank" rel="noopener"'
    )
  })
})
