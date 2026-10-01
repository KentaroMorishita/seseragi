import assert from "node:assert/strict"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import { assertReferenceLinkTitles } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

export type AuthoredLibraryPage = { route: string; kind: "api" | "module" }

// Ask the real typed dispatchers which canonical identities have authored copy.
// No source-text regex, hand-maintained page list, or generated HTML heuristic.
export function authoredLibraryPages(): AuthoredLibraryPage[] {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-authored-library-"))
  try {
    const result = renderPageClosure<AuthoredLibraryPage[]>({
      directory,
      modules: ["reference/editorial/catalog", "reference/catalog"],
      stdin: `${JSON.stringify({ schema: 1, origin: "", playgroundUrl: "", tourUrl: "", grammar: "", examples: [], referenceModules: compilerReferenceModules() })}\n`,
      entry: `import * as effects from "std/effect"
import * as inputs from "std/stdin"
import * as json from "std/json"
import * as arrays from "std/array"
import * as text from "std/text"
import { decodeBuildInput, ReferenceSymbol, ReferenceModule } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { referenceRoute } from "./reference/catalog"
type Failure deriving Show = | InputFailure | DecodeFailure | ConsoleFailure ConsoleError
struct Entry deriving JsonEncode { route: String, kind: String }
fn itemEntry symbol: ReferenceSymbol -> Array<Entry> = match editorialFor symbol {
  Just _ -> [Entry {route: referenceRoute symbol, kind: "api"}]
  Nothing -> []
}
fn moduleEntry value: ReferenceModule -> Array<Entry> =
  if arrays.length (moduleEditorial value.specifier) == 0 then []
  else [Entry {route: "/docs/library/" + text.replace "std/" "" value.specifier + "/", kind: "module"}]
fn requireInput value: Maybe<String> -> Either<Failure, String> = match value {
  Nothing -> Left InputFailure
  Just encoded -> Right encoded
}
pub effect fn main -> Unit with Console, Stdin fails Failure = do {
  limit <- inputs.lineLimit 67108864 |> effects.fromEither |> effects.mapError (\\_ -> InputFailure)
  line <- inputs.readLineWith limit |> effects.mapError (\\_ -> InputFailure)
  encoded <- requireInput line |> effects.fromEither
  input <- decodeBuildInput encoded |> effects.fromEither |> effects.mapError (\\_ -> DecodeFailure)
  let apis = arrays.concat [arrays.concat [itemEntry symbol | symbol <- module.items] | module <- input.referenceModules]
  let modules = arrays.concat [moduleEntry module | module <- input.referenceModules]
  println (json.encodeString (arrays.concat [apis, modules])) |> effects.mapError ConsoleFailure
}
`,
    })
    assert.ok(
      Array.isArray(result),
      "Authored-library inventory must be an array"
    )
    for (const entry of result) {
      assert.ok(entry.route.startsWith("/docs/library/"), entry.route)
      assert.ok(entry.kind === "api" || entry.kind === "module", entry.route)
    }
    assert.equal(
      new Set(result.map((entry) => entry.route)).size,
      result.length,
      "Authored-library routes must be unique"
    )
    return result
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}

export function authoredLibraryArticle(
  html: string,
  entry: AuthoredLibraryPage
): string {
  const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/u)
  assert.ok(article, `${entry.route}: missing authored article`)
  if (entry.kind === "api") return article[0]
  // Module metadata/public API lists intentionally use labels like “map · function”.
  // They are not authored page-name references, so keep the established boundary.
  const boundary = article[1].search(/<h2\b[^>]*\bid="using-this-module"/u)
  assert.ok(boundary >= 0, `${entry.route}: missing generated module boundary`)
  const authored = article[1].slice(0, boundary)
  assert.ok(
    authored.trim(),
    `${entry.route}: empty authored module introduction`
  )
  return `<article>${authored}</article>`
}

export function assertAuthoredLibraryTitles(
  html: string,
  titles: ReadonlyMap<string, string>,
  entry: AuthoredLibraryPage
): number {
  assert.ok(titles.has(entry.route), `${entry.route}: missing catalog title`)
  return assertReferenceLinkTitles(
    authoredLibraryArticle(html, entry),
    titles,
    entry.route
  )
}
