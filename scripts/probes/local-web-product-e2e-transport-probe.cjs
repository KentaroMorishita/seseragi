// Isolate the pipe used by @vscode/test-electron without network or extraction.
// Bun's result is diagnostic evidence; Node must preserve every byte because
// the product SDK runner uses Node. Neither result replaces the product journey.
const fs = require("node:fs")
const os = require("node:os")
const path = require("node:path")
const crypto = require("node:crypto")
const { spawn } = require("node:child_process")

async function main() {
  const output = path.resolve(process.argv[2])
  fs.mkdirSync(path.dirname(output), { recursive: true })
  const temporary = fs.mkdtempSync(
    path.join(os.tmpdir(), "seseragi-pipe-probe-")
  )
  const input = path.join(temporary, "input")
  const data = Buffer.alloc(32 * 1024 * 1024 + 7919, 87)
  const expected = crypto.createHash("sha256").update(data).digest("hex")
  fs.writeFileSync(input, data)
  const report = {
    schema: 1,
    runtime: process.versions,
    inputBytes: data.length,
    inputSha256: expected,
    cases: [],
  }
  const receiver =
    "import sys,hashlib,json,time\nh=hashlib.sha256();n=0\nwhile True:\n b=sys.stdin.buffer.read(65536)\n if not b:break\n n+=len(b);h.update(b);time.sleep(.0005)\nprint(json.dumps(dict(bytes=n,sha256=h.hexdigest())))\n"
  try {
    for (const chunkBytes of [65536, 1048576]) {
      const child = spawn("python3", ["-c", receiver], { stdio: "pipe" })
      const result = { chunkBytes, submittedBytes: 0 }
      let stdout = ""
      let stderr = ""
      child.stdout.on("data", (chunk) => {
        stdout += chunk
      })
      child.stderr.on("data", (chunk) => {
        stderr += chunk
      })
      child.stdin.on("error", (error) => {
        result.stdinError = String(error)
      })
      const write = child.stdin.write
      child.stdin.write = function (chunk, ...args) {
        result.submittedBytes += chunk.length
        return write.call(this, chunk, ...args)
      }
      const stream = fs.createReadStream(input, { highWaterMark: chunkBytes })
      stream.on("error", (error) => {
        child.stdin.destroy(error)
        child.kill()
      })
      const completed = new Promise((resolve, reject) => {
        child.once("error", reject)
        child.once("close", (code) => {
          result.exit = code
          resolve()
        })
      })
      stream.pipe(child.stdin)
      await completed
      if (result.exit !== 0) throw new Error(`byte receiver failed: ${stderr}`)
      result.received = JSON.parse(stdout)
      result.matches =
        result.received.bytes === data.length &&
        result.received.sha256 === expected
      report.cases.push(result)
    }
    report.completed = true
    report.lossObserved = report.cases.some((result) => !result.matches)
    fs.writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`)
    console.log(JSON.stringify(report))
    if (!process.versions.bun && report.lossObserved) {
      throw new Error("Node SDK transport did not preserve archive bytes")
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true })
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
