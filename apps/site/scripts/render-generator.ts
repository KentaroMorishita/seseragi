import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import {
  closeSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { dirname, join } from "node:path"

export type RenderedPage = { route: string; html: string }
export type RenderSelection = {
  schema: 1
  mode: "plan" | "render"
  offset: number
  count: number
}
type RenderResponse = {
  schema: number
  mode: string
  offset: number
  total: number
  routes: string[]
  pages: RenderedPage[]
}
export const RENDER_BATCH_SIZE = 32
const requestTimeout = 90_000
const routePattern = /^\/(?:[a-z0-9-]+\/)*$/u

function response(value: unknown, selection: RenderSelection): RenderResponse {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "Invalid render response"
  )
  const decoded = value as Partial<RenderResponse>
  assert.equal(decoded.schema, 1, "Unsupported render response schema")
  assert.equal(decoded.mode, selection.mode, "Render response mode mismatch")
  assert.equal(
    decoded.offset,
    selection.offset,
    "Render response offset mismatch"
  )
  assert.ok(
    Number.isSafeInteger(decoded.total) && (decoded.total ?? -1) >= 0,
    "Invalid render total"
  )
  assert.ok(Array.isArray(decoded.routes), "Missing render route inventory")
  assert.ok(
    decoded.routes.every(
      (route) => typeof route === "string" && routePattern.test(route)
    ),
    "Unsafe route in render inventory"
  )
  assert.ok(Array.isArray(decoded.pages), "Missing rendered page records")
  for (const page of decoded.pages) {
    assert.ok(page && typeof page === "object", "Invalid rendered page")
    assert.ok(
      typeof page.route === "string" && routePattern.test(page.route),
      "Unsafe rendered route"
    )
    assert.equal(typeof page.html, "string", `Invalid HTML for ${page.route}`)
  }
  return decoded as RenderResponse
}

// A callback makes the protocol's completeness checks independently testable.
// It cannot choose the page inventory: that inventory comes from Seseragi.
export function collectRenderedPages(
  request: (selection: RenderSelection) => unknown,
  batchSize = RENDER_BATCH_SIZE
): RenderedPage[] {
  assert.ok(
    Number.isSafeInteger(batchSize) &&
      batchSize > 0 &&
      batchSize <= RENDER_BATCH_SIZE,
    "Render batch size must be 1 to 32"
  )
  const selection: RenderSelection = {
    schema: 1,
    mode: "plan",
    offset: 0,
    count: 0,
  }
  const plan = response(request(selection), selection)
  assert.equal(plan.pages.length, 0, "A render plan must not contain HTML")
  assert.equal(
    plan.routes.length,
    plan.total,
    "Incomplete render route inventory"
  )
  assert.ok(plan.total > 0, "Empty render route inventory")
  assert.equal(new Set(plan.routes).size, plan.total, "Duplicate planned route")
  const pages: RenderedPage[] = []
  for (let offset = 0; offset < plan.total; offset += batchSize) {
    const count = Math.min(batchSize, plan.total - offset)
    const batchSelection: RenderSelection = {
      schema: 1,
      mode: "render",
      offset,
      count,
    }
    const batch = response(request(batchSelection), batchSelection)
    assert.equal(
      batch.total,
      plan.total,
      "Render catalog changed between batches"
    )
    assert.equal(
      batch.routes.length,
      0,
      "A render batch must not replace the route inventory"
    )
    assert.deepEqual(
      batch.pages.map(({ route }) => route),
      plan.routes.slice(offset, offset + count),
      "Missing, extra, duplicate, or reordered rendered routes"
    )
    pages.push(...batch.pages)
  }
  assert.deepEqual(
    pages.map(({ route }) => route),
    plan.routes,
    "Incomplete rendered site"
  )
  return pages
}

export function renderGenerator(
  entry: string,
  input: object,
  batchSize = RENDER_BATCH_SIZE
): RenderedPage[] {
  // Serialize external input once: every process gets precisely the same full
  // metadata. Fresh processes release per-batch typed JSON/HTML allocations.
  const encodedInput = JSON.stringify(input)
  const temporary = mkdtempSync(join(dirname(entry), ".render-"))
  const inputPath = join(temporary, "render-input.jsonl")
  const outputPath = join(temporary, "rendered-batch.json")
  try {
    return collectRenderedPages((selection) => {
      writeFileSync(
        inputPath,
        `${JSON.stringify(selection).slice(0, -1)},"input":${encodedInput}}\n`
      )
      let inputDescriptor: number | undefined
      let outputDescriptor: number | undefined
      try {
        inputDescriptor = openSync(inputPath, "r")
        outputDescriptor = openSync(outputPath, "wx")
        const result = spawnSync("bun", [entry], {
          cwd: dirname(entry),
          encoding: "utf8",
          stdio: [inputDescriptor, outputDescriptor, "pipe"],
          timeout: requestTimeout,
        })
        assert.equal(
          result.status,
          0,
          `${selection.mode} ${selection.offset}+${selection.count}: ${result.stderr?.toString() || result.error?.message || `generator status ${result.status}, signal ${result.signal}`}`
        )
        return JSON.parse(readFileSync(outputPath, "utf8")) as unknown
      } finally {
        if (inputDescriptor !== undefined) closeSync(inputDescriptor)
        if (outputDescriptor !== undefined) closeSync(outputDescriptor)
        rmSync(inputPath, { force: true })
        if (outputDescriptor !== undefined) rmSync(outputPath, { force: true })
      }
    }, batchSize)
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}
