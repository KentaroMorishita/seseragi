const { test } = require("node:test")
const assert = require("node:assert/strict")
const fs = require("node:fs/promises")
const os = require("node:os")
const path = require("node:path")
const { runProductJourney } = require("./local-web-product-e2e-node.cjs")

async function fixture(t) {
  const temporary = await fs.mkdtemp(
    path.join(os.tmpdir(), "seseragi-sdk-test-")
  )
  t.after(() => fs.rm(temporary, { recursive: true, force: true }))
  const output = path.join(temporary, "output")
  await fs.mkdir(output)
  return {
    temporary,
    output,
    repositoryRoot: "/fixture/repository",
    project: "/fixture/project",
    cli: "/fixture/native/seseragi",
    vsix: "/fixture/official.vsix",
    version: "0.0.0",
    vscodeVersion: "1.133.0",
    extensionsDirectory: "/fixture/extensions",
    userDataDirectory: "/fixture/user-data",
  }
}

test("uses one private verified SDK install for the installed-product journey", async (t) => {
  const config = await fixture(t)
  const calls = []
  let downloaded
  const report = await runProductJourney(config, {
    async downloadAndUnzipVSCode(options) {
      calls.push("download")
      downloaded = options
      assert.equal(options.version, config.vscodeVersion)
      assert.equal(path.dirname(options.cachePath), config.temporary)
      return path.join(options.cachePath, "code")
    },
    async runVSCodeCommand(args, options) {
      calls.push("install")
      assert.deepEqual(options, downloaded)
      assert(args.includes(config.vsix))
      assert(args.includes(`--extensions-dir=${config.extensionsDirectory}`))
    },
    async runTests(options) {
      calls.push("journey")
      assert.equal(
        options.vscodeExecutablePath,
        path.join(downloaded.cachePath, "code")
      )
      assert.equal(options.extensionTestsEnv.SESERAGI_E2E_CLI, config.cli)
      assert.equal(options.extensionTestsEnv.SESERAGI_E2E_VSIX, config.vsix)
      assert.equal(
        options.extensionTestsEnv.SESERAGI_E2E_PROJECT,
        config.project
      )
      assert(options.launchArgs.includes(config.project))
      await fs.writeFile(
        path.join(config.output, "report.json"),
        JSON.stringify({ schema: 1, result: "passed" })
      )
    },
  })
  assert.deepEqual(calls, ["download", "install", "journey"])
  assert(report.downloadCompleted && report.journeyCompleted)
  await assert.rejects(fs.stat(report.cachePath), { code: "ENOENT" })
})

test("rejects truncated provisioning before installation and removes incomplete caches", async (t) => {
  const config = await fixture(t)
  let cachePath
  await assert.rejects(
    runProductJourney(config, {
      async downloadAndUnzipVSCode(options) {
        cachePath = options.cachePath
        await fs.writeFile(
          path.join(cachePath, "partial-archive"),
          "incomplete"
        )
        throw new Error("archive byte length / SHA256 mismatch")
      },
      async runVSCodeCommand() {
        assert.fail("installation must not start")
      },
      async runTests() {
        assert.fail("journey must not start")
      },
    }),
    /archive byte length/
  )
  const report = JSON.parse(
    await fs.readFile(path.join(config.output, "provisioning.json"), "utf8")
  )
  assert.equal(report.downloadCompleted, false)
  assert.equal(report.journeyCompleted, false)
  assert.match(report.error, /SHA256 mismatch/)
  await assert.rejects(fs.stat(cachePath), { code: "ENOENT" })
})

test("propagates product assertion failures after a valid download", async (t) => {
  const config = await fixture(t)
  await assert.rejects(
    runProductJourney(config, {
      async downloadAndUnzipVSCode(options) {
        return path.join(options.cachePath, "code")
      },
      async runVSCodeCommand() {},
      async runTests() {
        throw new Error("original source map assertion failed")
      },
    }),
    /original source map assertion failed/
  )
  const report = JSON.parse(
    await fs.readFile(path.join(config.output, "provisioning.json"), "utf8")
  )
  assert.equal(report.downloadCompleted, true)
  assert.equal(report.journeyCompleted, false)
})

test("does not accept an SDK exit without a passing product report", async (t) => {
  for (const product of [undefined, { schema: 1, result: "failed" }]) {
    const config = await fixture(t)
    await assert.rejects(
      runProductJourney(config, {
        async downloadAndUnzipVSCode(options) {
          return path.join(options.cachePath, "code")
        },
        async runVSCodeCommand() {},
        async runTests() {
          if (product) {
            await fs.writeFile(
              path.join(config.output, "report.json"),
              JSON.stringify(product)
            )
          }
        },
      })
    )
    const report = JSON.parse(
      await fs.readFile(path.join(config.output, "provisioning.json"), "utf8")
    )
    assert.equal(report.downloadCompleted, true)
    assert.equal(report.journeyCompleted, false)
  }
})
