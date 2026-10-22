import { expect, test } from "bun:test"
import { createExecutionFrameDocument } from "../src/runtime/isolated-execution"

test("keeps bundled script bytes out of HTML parser and replacement-string syntax", () => {
  const marker = "</ScRiPt><!--<script>$&$`$'"
  const html = createExecutionFrameDocument(
    `globalThis.__frameDocumentTest = ${JSON.stringify(marker)};`,
    "test-session"
  )
  const script = html.slice(
    html.indexOf('<script data-session="test-session">') +
      '<script data-session="test-session">'.length,
    html.lastIndexOf("</script>")
  )
  expect(script).not.toContain("<")
  expect(html.match(/<\/script>/gi)).toHaveLength(1)
  try {
    new Function(script)()
    expect(
      (globalThis as unknown as { __frameDocumentTest: string })
        .__frameDocumentTest
    ).toBe(marker)
  } finally {
    delete (globalThis as unknown as { __frameDocumentTest?: string })
      .__frameDocumentTest
  }
  expect(html).toContain("base-uri 'none'; form-action 'none'")
})

test("private history preserves asynchronous traversal and clears forward entries", async () => {
  const { createPreviewHistory } = await import(
    "../src/runtime/preview-history"
  )
  const history = createPreviewHistory("https://example.test/initial")
  try {
    history.push("https://example.test/one")
    history.replace("https://example.test/replaced")
    const previous = new Promise<string>((resolve) =>
      history.listen(() => resolve(history.currentHref()))
    )
    history.back()
    expect(history.currentHref()).toBe("https://example.test/replaced")
    expect(await previous).toBe("https://example.test/initial")
    const next = new Promise<string>((resolve) =>
      history.listen(() => resolve(history.currentHref()))
    )
    history.forward()
    expect(await next).toBe("https://example.test/replaced")
    const back = new Promise<string>((resolve) =>
      history.listen(() => resolve(history.currentHref()))
    )
    history.back()
    await back
    history.push("https://example.test/new")
    history.forward()
    await new Promise((resolve) => setTimeout(resolve, 10))
    expect(history.currentHref()).toBe("https://example.test/new")
  } finally {
    history.dispose()
  }
})
