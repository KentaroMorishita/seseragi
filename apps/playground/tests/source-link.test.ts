import { expect, test } from "bun:test"
import {
  playgroundUrlForSource,
  sourceFromPlaygroundUrl,
} from "../src/workspace/source-link"

test("round-trips Docs source through a Playground URL", () => {
  // biome-ignore lint/suspicious/noTemplateCurlyInString: Seseragi source fixture
  const source = "fn greet name: String -> String = `hello ${name}`\n"
  const url = playgroundUrlForSource("https://seseragi.vercel.app/", source)
  expect(sourceFromPlaygroundUrl(url)).toBe(source)
  expect(new URL(url).origin).toBe("https://seseragi.vercel.app")
})

test("ignores absent and oversized source links", () => {
  expect(
    sourceFromPlaygroundUrl("https://seseragi.vercel.app/")
  ).toBeUndefined()
  const oversized = `https://seseragi.vercel.app/?source=${"a".repeat(70_000)}`
  expect(sourceFromPlaygroundUrl(oversized)).toBeUndefined()
})
