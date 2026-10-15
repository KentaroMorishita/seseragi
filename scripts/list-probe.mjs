import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"

const module = await import(process.argv[2])
const result = module.probe()
const stdout = Array.isArray(result)
  ? `[${result.join(", ")}]\n`
  : `${result}\n`
assert.equal(stdout, readFileSync(process.argv[3], "utf8"))
console.log(
  JSON.stringify({
    normalBody: true,
    cases: Array.isArray(result) ? result.length : 1,
    stdoutSha256: createHash("sha256").update(stdout).digest("hex"),
  })
)
