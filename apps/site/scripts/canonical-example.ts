import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { playgroundUrlForSource } from "../../playground/src/workspace/source-link"
import { highlightTypeScript } from "./typescript-highlight"

const root = resolve(import.meta.dir, "../../..")

export function canonicalExample(
  id: string,
  sourcePath: string,
  playgroundUrl: string,
  standalone = true,
  language: "seseragi" | "typescript" = "seseragi"
) {
  const source = readFileSync(join(root, sourcePath), "utf8")
  return {
    id,
    sourcePath,
    source,
    sha256: createHash("sha256").update(source).digest("hex"),
    playgroundUrl:
      standalone && language === "seseragi"
        ? playgroundUrlForSource(playgroundUrl, source)
        : "",
    highlighted:
      language === "seseragi"
        ? highlightSeseragi(source).map(({ text, classes }) => ({
            text,
            className: classes,
          }))
        : highlightTypeScript(source),
  }
}
