import { readdir } from "node:fs/promises"
import { resolve } from "node:path"

const repository = resolve(import.meta.dir, "..")
const lessonsRoot = resolve(repository, "examples/tour/lessons")
const base = process.argv[2] ?? "origin/main"
const tokenPattern =
  /\/\/[^\n]*|"(?:\\.|[^"\\])*"|`(?!\[)(?:\\.|[^`\\])*`|[\p{L}_][\p{L}\p{N}_']*|\d+(?:\.\d+)?|[^\s]/gsu
const rangePattern =
  /("sourceRange"\s*:\s*\{\s*"startLine"\s*:\s*)(\d+)(\s*,\s*"endLine"\s*:\s*)(\d+)(\s*\})/gu

type Token = { readonly raw: string; readonly line: number }

function tokenize(source: string): Token[] {
  return [...source.matchAll(tokenPattern)].map((match) => ({
    raw: match[0],
    line: source.slice(0, match.index ?? 0).split("\n").length,
  }))
}

function mapRange(
  original: readonly Token[],
  canonical: readonly Token[],
  start: number,
  end: number
): readonly [number, number] {
  const selected = original
    .map((token, index) => ({ token, index }))
    .filter(({ token }) => token.line >= start && token.line <= end)
  if (selected.length === 0) {
    const nearest = original.reduce(
      (best, token, index) => {
        const distance = Math.abs(token.line - start)
        return distance < best.distance ? { distance, index } : best
      },
      { distance: Number.POSITIVE_INFINITY, index: 0 }
    )
    const line = canonical[nearest.index]?.line ?? 1
    return [line, line]
  }
  const first = selected[0]?.index ?? 0
  const last = selected.at(-1)?.index ?? first
  return [canonical[first]?.line ?? 1, canonical[last]?.line ?? 1]
}

let updated = 0
for (const lessonId of await readdir(lessonsRoot)) {
  const lessonPath = resolve(lessonsRoot, lessonId, "lesson.json")
  if (!(await Bun.file(lessonPath).exists())) continue
  const lessonText = await Bun.file(lessonPath).text()
  const sourceName = (JSON.parse(lessonText) as { files?: { source?: string } })
    .files?.source
  if (sourceName === undefined) continue

  const relativeSource = `examples/tour/lessons/${lessonId}/${sourceName}`
  const originalResult = Bun.spawnSync(
    ["git", "show", `${base}:${relativeSource}`],
    { cwd: repository }
  )
  if (originalResult.exitCode !== 0) continue
  const original = tokenize(originalResult.stdout.toString())
  const canonical = tokenize(
    await Bun.file(resolve(repository, relativeSource)).text()
  )
  if (
    original.length !== canonical.length ||
    original.some((token, index) => token.raw !== canonical[index]?.raw)
  ) {
    throw new Error(`token sequence changed for ${relativeSource}`)
  }

  const nextText = lessonText.replace(
    rangePattern,
    (_whole, beforeStart, start, beforeEnd, end, closing) => {
      const [nextStart, nextEnd] = mapRange(
        original,
        canonical,
        Number(start),
        Number(end)
      )
      return `${beforeStart}${nextStart}${beforeEnd}${nextEnd}${closing}`
    }
  )
  if (nextText !== lessonText) {
    await Bun.write(lessonPath, nextText)
    updated += 1
  }
}

console.log(`Updated ${updated} Tour lesson source-range file(s).`)
