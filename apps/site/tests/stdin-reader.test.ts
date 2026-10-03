import { expect, setDefaultTimeout, test } from "bun:test"
import { spawn, spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import ts from "typescript"
import {
  stdinReaderCases,
  stdinReaderExamples,
  stdinReaderRoutes,
} from "../scripts/stdin-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const node = Bun.which(process.env.SESERAGI_NODE ?? "node")
if (!node)
  throw new Error("The reader suite requires an installed Node executable")
const runtimes = [
  ["node", node],
  ["bun", bun],
] as const
const evidenceRoot = resolve(
  process.env.STDIN_READER_EVIDENCE_DIR ??
    join(tmpdir(), "seseragi-stdin-reader-evidence")
)
mkdirSync(evidenceRoot, { recursive: true })
const evidence = mkdtempSync(join(evidenceRoot, "run-"))
const fixtures = join(root, "apps/site/tests/fixtures/stdin-reader")
const examples = stdinReaderExamples("https://seseragi.vercel.app/")
for (const part of ["home", "tmp"]) mkdirSync(join(evidence, part))
// Every child gets synthetic settings, closed synthetic input, and a timeout.
// The staged-pipe test is the sole bounded, explicitly managed open input.
const environment = {
  PATH: `${dirname(bun)}:${dirname(node)}:/usr/bin:/bin`,
  HOME: join(evidence, "home"),
  TMPDIR: join(evidence, "tmp"),
  LANG: "C.UTF-8",
}
const commands: unknown[] = []
const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
function save(name: string, value: unknown) {
  writeFileSync(join(evidence, name), `${JSON.stringify(value, null, 2)}\n`)
}
function command(
  label: string,
  executable: string,
  args: string[],
  input: string | Buffer = "",
  cwd = evidence
) {
  const inputBytes = Buffer.from(input)
  const result = spawnSync(executable, args, {
    cwd,
    env: environment,
    input: inputBytes,
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  const streams = {
    stdin: inputBytes,
    stdout: result.stdout ?? Buffer.alloc(0),
    stderr: result.stderr ?? Buffer.alloc(0),
  }
  const files = Object.entries(streams).map(([stream, bytes]) => {
    const path = `${label}.${stream}.bin`
    writeFileSync(join(evidence, path), bytes)
    return { stream, path, bytes: bytes.length, sha256: sha256(bytes) }
  })
  commands.push({
    label,
    executable,
    args,
    cwd,
    environment,
    stdinClosedByHarness: true,
    timeoutMs: 60_000,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    files,
  })
  save("commands.json", commands)
  return result
}
function succeeded(result: ReturnType<typeof command>, output?: string) {
  const diagnostic = result.stderr?.toString() ?? ""
  expect(result.error, diagnostic).toBeUndefined()
  expect(result.status, diagnostic).toBe(0)
  expect(result.stderr).toEqual(Buffer.alloc(0))
  if (output !== undefined) expect(result.stdout).toEqual(Buffer.from(output))
}
function sample(slug: string) {
  const source = examples.find((item) => item.id === `stdin-reader-${slug}`)
  if (!source) throw new Error(`Missing stdin source: ${slug}`)
  return source
}
const entries = new Map<string, string>()
function buildSource(slug: string, source = sample(slug).source) {
  const prior = entries.get(slug)
  if (prior) return prior
  const directory = join(evidence, "native", slug)
  mkdirSync(directory, { recursive: true })
  writeFileSync(join(directory, "main.ssrg"), source)
  for (const [label, args] of [
    ["format", ["format", "--check", "main.ssrg"]],
    ["lint", ["lint", "main.ssrg", "--deny-warnings"]],
    [
      "build",
      [
        "build",
        "main.ssrg",
        "--target",
        "process",
        "--profile",
        "release",
        "--out-dir",
        "release",
      ],
    ],
  ] as const)
    succeeded(command(`${slug}.${label}`, cli, [...args], "", directory))
  const manifest = JSON.parse(
    readFileSync(join(directory, "release/.seseragi-build.json"), "utf8")
  )
  expect(manifest).toMatchObject({
    profile: "release",
    target: "process",
    entry: "entry.js",
  })
  const entry = join(directory, "release", manifest.entry)
  entries.set(slug, entry)
  return entry
}
let typescriptOutput: string | undefined
function compileTypeScript() {
  if (typescriptOutput) return typescriptOutput
  const directory = join(evidence, "typescript")
  const source = join(directory, "source")
  const output = join(directory, "output")
  mkdirSync(join(source, "native"), { recursive: true })
  mkdirSync(join(source, "proof-only"))
  const paths = []
  for (const item of stdinReaderCases) {
    const path = join(source, "native", `${item.slug}.ts`)
    writeFileSync(path, sample(`${item.slug}-ts`).source)
    paths.push(path)
  }
  // Keep the displayed validator unchanged and add two synthetic calls to
  // cover TypeScript's wider number input without changing the page output.
  const validationPath = join(source, "native", "limit-config-validation.ts")
  writeFileSync(
    validationPath,
    `${sample("limit-config-ts").source}\nstdout.write(describe(4.5) + "\\n")\nstdout.write(describe(9007199254740992) + "\\n")\n`
  )
  paths.push(validationPath)
  for (const name of readdirSync(join(fixtures, "proof-only"))) {
    const path = join(source, "proof-only", name)
    writeFileSync(path, readFileSync(join(fixtures, "proof-only", name)))
    paths.push(path)
  }
  succeeded(
    command("strict-typescript", bun, [
      join(root, "node_modules/typescript/bin/tsc"),
      "--strict",
      "--noUncheckedIndexedAccess",
      "--exactOptionalPropertyTypes",
      "--target",
      "ES2022",
      "--lib",
      "ESNext,DOM",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      "--rootDir",
      source,
      "--outDir",
      output,
      "--types",
      "node",
      "--typeRoots",
      join(root, "node_modules/@types"),
      ...paths,
    ])
  )
  writeFileSync(join(output, "package.json"), '{"type":"module"}\n')
  typescriptOutput = output
  return output
}
const protectedSources = {
  "one-line":
    "3da2a0a10804c02c391cee2ec589983dcb4370168566c8f5b175a1111a5dcd15",
  "three-lines":
    "71511a3fea683c5069333ab1c8f23c5299bccfed25506758c35c9ff6671e2822",
  "limited-lines":
    "3f50678a27f0d5a371f205f1c83ca9d377a0b2d13446169e91c6331d58958aad",
  "limit-config":
    "8161f10419085caff9438d531cbb8c0d0181a9a1df60fb86a3c799a4bdea62ad",
  "default-limit":
    "d71ea0a134804ff8abe947cc6f257cb8d6a57ac24053310229ef5726c9adb7d1",
}
save(
  "source-hashes.json",
  [
    ...examples.map((item) => item.sourcePath),
    "apps/site/scripts/stdin-reader.ts",
    "apps/site/tests/stdin-reader.test.ts",
    ...readdirSync(fixtures, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) =>
        join(entry.parentPath, entry.name).slice(root.length + 1)
      ),
    "examples/spec/artifacts/stdlib-schema-1/reference/module.json",
    "apps/playground/src/wasm/pkg/seseragi_wasm.js",
    "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
    "apps/playground/src/runtime/browser-execution.ts",
    "apps/playground/src/runtime/runtime-modules.ts",
    "apps/playground/src/main.ts",
    "runtime/ts/src/browser/host.ts",
    "runtime/ts/src/browser/stdin.ts",
    "runtime/ts/src/stdin-service.ts",
    "runtime/ts/src/effect.ts",
  ].map((path) => ({ path, sha256: sha256(readFileSync(join(root, path))) }))
)

test("Stdin12 keeps exact catalog identities, owners and twelve source panels", () => {
  const selection = JSON.parse(
    readFileSync(join(fixtures, "selected12.json"), "utf8")
  )
  const catalog = JSON.parse(
    readFileSync(
      join(
        root,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  expect(stdinReaderCases).toHaveLength(6)
  expect(stdinReaderRoutes).toHaveLength(12)
  expect(examples).toHaveLength(12)
  expect(new Set(stdinReaderRoutes.map((item) => item.identity)).size).toBe(12)
  expect(stdinReaderRoutes.map((item) => item.identity)).toEqual(
    selection.map((item: { identity: string }) => item.identity)
  )
  for (const [index, route] of stdinReaderRoutes.entries()) {
    const selected = selection[index]
    expect(route).toMatchObject({
      identity: selected.identity,
      owner: selected.module,
      module: selected.module,
      name: selected.name,
      namespace: selected.namespace,
      kind: selected.itemKind,
      route: selected.route,
    })
    const module = catalog.modules.find(
      (item: { specifier: string }) => item.specifier === route.owner
    )
    expect(module).toBeDefined()
    expect(module.targets).toEqual(selected.targets)
    if (route.namespace !== "module") {
      expect(
        module.items.find(
          (item: { identity: string }) => item.identity === route.identity
        )
      ).toMatchObject({
        identity: selected.identity,
        module: selected.module,
        namespace: selected.namespace,
        kind: selected.itemKind,
        name: selected.name,
        signature: selected.signature,
      })
    }
    expect(sample(route.example).source).toContain("pub effect fn main")
  }
  for (const item of stdinReaderCases) {
    for (const suffix of ["", "-ts"]) {
      const source = sample(`${item.slug}${suffix}`)
      expect(source.sha256).toBe(sha256(source.source))
      expect(source.highlighted.map((span) => span.text).join("")).toBe(
        source.source
      )
      expect(source.source).not.toContain("strict-line-reader")
      if (item.portable && !suffix)
        expect(new URL(source.playgroundUrl).searchParams.get("source")).toBe(
          source.source
        )
      else expect(source.playgroundUrl).toBe("")
    }
    if (item.slug !== "limit-config")
      expect(sample(`${item.slug}-ts`).source).toContain('from "node:readline"')
  }
  expect(
    stdinReaderCases.filter((item) => item.portable).map((item) => item.slug)
  ).toEqual(["one-line"])
  for (const [slug, hash] of Object.entries(protectedSources))
    expect(sample(slug).sha256).toBe(hash)
  save("routes.json", stdinReaderRoutes)
})

test("every exact displayed source compiles and demonstrates its input on Node and Bun", () => {
  for (const [label, executable] of [["cli", cli], ...runtimes])
    succeeded(
      command(`${label}-version`, executable!, [
        label === "cli" ? "--version-json" : "--version",
      ])
    )
  const output = compileTypeScript()
  for (const item of stdinReaderCases) {
    const native = buildSource(item.slug)
    for (const [runtime, executable] of runtimes) {
      succeeded(
        command(`${item.slug}.${runtime}`, executable, [native], item.input),
        item.output
      )
      succeeded(
        command(
          `${item.slug}.typescript.${runtime}`,
          executable,
          [join(output, "native", `${item.slug}.js`)],
          item.input
        ),
        item.output
      )
      if (item.slug === "limit-config")
        succeeded(
          command(
            `limit-config.typescript-number-validation.${runtime}`,
            executable,
            [join(output, "native/limit-config-validation.js")]
          ),
          `${item.output}Use a safe whole number: 4.5\nUse a safe whole number: 9007199254740992\n`
        )
    }
  }
  succeeded(
    command(
      "one-line.cli-run",
      cli,
      ["run", "main.ssrg"],
      "parcel\nignored\n",
      join(evidence, "native/one-line")
    ),
    "Line: parcel\n"
  )
})

const textCases = [
  ["empty", "", "EOF\n"],
  ["blank-lf", "\n", "Blank line\n"],
  ["blank-crlf", "\r\n", "Blank line\n"],
  ["first-line", "parcel\nignored\n", "Line: parcel\n"],
  ["final-content", "parcel", "Line: parcel\n"],
  ["unicode", "荷物\r\n", "Line: 荷物\n"],
] as const
test("one-line readers preserve blank, EOF, Unicode and first-line boundaries", () => {
  const output = compileTypeScript()
  for (const slug of ["one-line", "qualified-one-line"]) {
    const native = buildSource(slug)
    for (const [name, input, expected] of textCases)
      for (const [runtime, executable] of runtimes)
        for (const [implementation, entry] of [
          ["seseragi", native],
          ["typescript", join(output, "native", `${slug}.js`)],
        ])
          succeeded(
            command(
              `${slug}.${name}.${implementation}.${runtime}`,
              executable,
              [entry!],
              input
            ),
            expected
          )
  }
})

const byteCases: {
  slug: string
  name: string
  input: Buffer
  output: string
}[] = [
  {
    slug: "three-lines",
    name: "repeated-eof",
    input: Buffer.from(""),
    output: "EOF\nEOF\nEOF\n",
  },
  {
    slug: "three-lines",
    name: "blank-final",
    input: Buffer.from("\r\nparcel"),
    output: "Blank line\nLine: parcel\nEOF\n",
  },
  {
    slug: "three-lines",
    name: "bare-cr",
    input: Buffer.from("alpha\rbeta\nend\r"),
    output: "Line: alpha\rbeta\nLine: end\r\nEOF\n",
  },
  {
    slug: "three-lines",
    name: "invalid-after-crlf",
    input: Buffer.from([
      103, 111, 111, 100, 13, 10, 120, 255, 10, 111, 107, 10,
    ]),
    output: "Line: good\nInvalid UTF-8 at byte 7\nLine: ok\n",
  },
  {
    slug: "three-lines",
    name: "overlong-offset",
    input: Buffer.from([111, 107, 10, 97, 192, 128, 10, 111, 107, 10]),
    output: "Line: ok\nInvalid UTF-8 at byte 4\nLine: ok\n",
  },
  {
    slug: "three-lines",
    name: "truncated-at-eof",
    input: Buffer.from([226, 130]),
    output: "Invalid UTF-8 at byte 0\nEOF\nEOF\n",
  },
  {
    slug: "three-lines",
    name: "surrogate-recovery",
    input: Buffer.from([237, 160, 128, 10, 111, 107, 10]),
    output: "Invalid UTF-8 at byte 0\nLine: ok\nEOF\n",
  },
  {
    slug: "three-lines",
    name: "bad-continuation",
    input: Buffer.from([226, 40, 161, 10, 111, 107, 10]),
    output: "Invalid UTF-8 at byte 0\nLine: ok\nEOF\n",
  },
  {
    slug: "limited-lines",
    name: "exact-crlf",
    input: Buffer.from("abcd\r\nok\n"),
    output: "Line: abcd\nLine: ok\nEOF\n",
  },
  {
    slug: "limited-lines",
    name: "utf8-content-bytes",
    input: Buffer.from("😀\néé\nééa\n"),
    output: "Line: 😀\nLine: éé\nLine exceeds 4 bytes\n",
  },
  {
    slug: "limited-lines",
    name: "bare-cr-counted",
    input: Buffer.from("abc\r\nabcd\r"),
    output: "Line: abc\nLine exceeds 4 bytes\nEOF\n",
  },
  {
    slug: "limited-lines",
    name: "invalid-consumed",
    input: Buffer.from([97, 255, 10, 111, 107, 10]),
    output: "Invalid UTF-8 at byte 1\nLine: ok\nEOF\n",
  },
  {
    slug: "limited-lines",
    name: "limit-before-decoding",
    input: Buffer.from([97, 98, 99, 100, 255, 10, 111, 107, 10]),
    output: "Line exceeds 4 bytes\nLine: ok\nEOF\n",
  },
  {
    slug: "limited-lines",
    name: "oversize-eof",
    input: Buffer.from("abcde"),
    output: "Line exceeds 4 bytes\nEOF\nEOF\n",
  },
]
test("native byte limits, strict decoding and sequential recovery match explicit outcomes", () => {
  const output = compileTypeScript()
  for (const item of byteCases) {
    const native = buildSource(item.slug)
    for (const [runtime, executable] of runtimes) {
      succeeded(
        command(
          `${item.slug}.${item.name}.${runtime}`,
          executable,
          [native],
          item.input
        ),
        item.output
      )
      // Independent finite-pipe proof, deliberately absent from reader panels.
      succeeded(
        command(
          `${item.slug}.${item.name}.proof.${runtime}`,
          executable,
          [join(output, "proof-only", `${item.slug}.js`)],
          item.input
        ),
        item.output
      )
      // This displayed multibyte demonstration uses valid UTF-8 and LF, so
      // the native readline panel must produce these same exact bytes too.
      if (item.name === "utf8-content-bytes")
        succeeded(
          command(
            `${item.slug}.${item.name}.typescript.${runtime}`,
            executable,
            [join(output, "native", `${item.slug}.js`)],
            item.input
          ),
          item.output
        )
    }
  }
  for (const [name, input, output] of [
    ["one-mib-lf", `${"a".repeat(1_048_576)}\n`, "Line accepted\n"],
    ["one-mib-crlf", `${"a".repeat(1_048_576)}\r\n`, "Line accepted\n"],
    [
      "one-mib-plus-one",
      `${"a".repeat(1_048_577)}\n`,
      "Line exceeds 1048576 bytes\n",
    ],
  ])
    for (const [runtime, executable] of runtimes)
      succeeded(
        command(
          `default-limit.${name}.${runtime}`,
          executable,
          [buildSource("default-limit")],
          input!
        ),
        output
      )
  for (const [runtime, executable] of runtimes) {
    succeeded(
      command(
        `qualified-one-line.invalid.${runtime}`,
        executable,
        [buildSource("qualified-one-line")],
        Buffer.from([255, 10, 111, 107, 10])
      ),
      "Invalid UTF-8 at byte 0\n"
    )
    succeeded(
      command(
        `qualified-one-line.oversize.${runtime}`,
        executable,
        [buildSource("qualified-one-line")],
        `${"a".repeat(1_048_577)}\nok\n`
      ),
      "Line exceeds 1048576 bytes\n"
    )
  }
})

test("native readline differences are observed rather than reported as strict-byte parity", () => {
  const output = compileTypeScript()
  const results = []
  for (const [name, input, nativeOutput, typescriptOutput] of [
    [
      "malformed-byte",
      Buffer.from([255, 10]),
      "Invalid UTF-8 at byte 0\n",
      "Line: �\n",
    ],
    [
      "bare-cr",
      Buffer.from("alpha\rbeta\n"),
      "Line: alpha\rbeta\n",
      "Line: alpha\n",
    ],
  ] as const) {
    for (const [runtime, executable] of runtimes) {
      succeeded(
        command(
          `difference.${name}.seseragi.${runtime}`,
          executable,
          [buildSource("qualified-one-line")],
          input
        ),
        nativeOutput
      )
      succeeded(
        command(
          `difference.${name}.typescript.${runtime}`,
          executable,
          [join(output, "native/qualified-one-line.js")],
          input
        ),
        typescriptOutput
      )
      results.push({
        name,
        runtime,
        inputHex: input.toString("hex"),
        nativeOutput,
        typescriptOutput,
      })
    }
  }
  save("native-readline-differences.json", results)
})

test("the displayed POSIX printf recipe supplies the exact malformed bytes", () => {
  const recipe = "printf 'ok\\na\\300\\200\\nok\\n'"
  const bytes = command("octal-printf.bytes", "/bin/sh", ["-c", recipe])
  succeeded(bytes)
  expect(bytes.stdout).toEqual(
    Buffer.from([111, 107, 10, 97, 192, 128, 10, 111, 107, 10])
  )
  succeeded(
    command("octal-printf.pipeline", "/bin/sh", [
      "-c",
      `${recipe} | "$1" "$2"`,
      "stdin-reader-recipe",
      node,
      buildSource("three-lines"),
    ]),
    "Line: ok\nInvalid UTF-8 at byte 4\nLine: ok\n"
  )
})

test("unhandled input failures have nonzero status; successful empty and blank input do not", () => {
  const entry = buildSource(
    "unhandled-input",
    readFileSync(join(fixtures, "unhandled-input.ssrg"), "utf8")
  )
  for (const [runtime, executable] of runtimes) {
    for (const [name, input] of [
      ["empty", ""],
      ["blank", "\n"],
    ])
      succeeded(
        command(`unhandled.${name}.${runtime}`, executable, [entry], input!),
        ""
      )
    for (const [name, input, diagnostic] of [
      ["invalid", Buffer.from([255, 10]), "InvalidStdinUtf8"],
      [
        "oversize",
        Buffer.from(`${"a".repeat(1_048_577)}\n`),
        "StdinLineTooLong",
      ],
    ] as const) {
      const result = command(
        `unhandled.${name}.${runtime}`,
        executable,
        [entry],
        input
      )
      expect(result.error).toBeUndefined()
      expect(result.status).toBe(1)
      expect(result.stdout).toEqual(Buffer.alloc(0))
      expect(result.stderr.toString()).toContain(diagnostic)
    }
  }
})

test("four reachable diagnostics have complete canonical executable repairs", () => {
  for (const [slug, code, expected, actual, repairSlug] of [
    ["float-limit", "SES-T0101", "Int", "Float", "limit-config"],
    ["raw-int-limit", "SES-T0101", "LineLimit", "Int", "limited-lines"],
    ["maybe-as-string", "SES-T0101", "String", "Maybe<String>", "one-line"],
    ["qualified-show", "SES-T0201", "Show", "StdinError", "qualified-one-line"],
  ]) {
    const source = readFileSync(join(fixtures, `invalid/${slug}.ssrg`), "utf8")
    const directory = join(evidence, "negative", slug!)
    mkdirSync(directory, { recursive: true })
    writeFileSync(join(directory, "main.ssrg"), source)
    const rejected = command(
      `${slug}.rejected`,
      cli,
      ["lint", "main.ssrg"],
      "",
      directory
    )
    expect(rejected.error).toBeUndefined()
    expect(rejected.status, rejected.stderr.toString()).toBe(2)
    expect(rejected.stdout).toEqual(Buffer.alloc(0))
    expect(rejected.stderr.toString()).toContain(code!)
    expect(rejected.stderr.toString()).toContain(expected!)
    expect(rejected.stderr.toString()).toContain(actual!)
    const repair = readFileSync(join(fixtures, `repairs/${slug}.ssrg`), "utf8")
    expect(repair).toBe(sample(repairSlug!).source)
    const demonstration = stdinReaderCases.find(
      (item) => item.slug === repairSlug
    )!
    for (const [runtime, executable] of runtimes)
      succeeded(
        command(
          `${slug}.repaired.${runtime}`,
          executable,
          [buildSource(repairSlug!)],
          demonstration.input
        ),
        demonstration.output
      )
  }
})

test("qualified package process support and web rejection remain separate from standalone bundling", () => {
  for (const [name, slug, targets] of [
    ["qualified", "qualified-one-line", ["process", "web"]],
    ["prelude", "one-line", ["process", "web"]],
  ] as const) {
    const directory = join(evidence, "packages", name)
    mkdirSync(join(directory, "src"), { recursive: true })
    writeFileSync(join(directory, "src/main.ssrg"), sample(slug).source)
    writeFileSync(
      join(directory, "seseragi.toml"),
      `[package]\nname = "docs/stdin-reader-${name}"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "main"\ntarget = "process"\n`
    )
    succeeded(
      command(`package.${name}.lock`, cli, ["lock", "update", directory])
    )
    for (const target of targets) {
      const output = join(directory, `release-${target}`)
      const result = command(`package.${name}.${target}`, cli, [
        "build",
        directory,
        "--target",
        target,
        "--profile",
        "release",
        "--out-dir",
        output,
      ])
      if (name === "qualified" && target === "web") {
        expect(result.status).toBe(2)
        expect(result.stderr.toString()).toContain(
          "SES-K0203 provider.target-mismatch"
        )
        expect(result.stderr.toString()).toContain("std/stdin")
        expect(result.stderr.toString()).toContain("browser")
      } else {
        succeeded(result)
        if (target === "process") {
          const manifest = JSON.parse(
            readFileSync(join(output, ".seseragi-build.json"), "utf8")
          )
          for (const [runtime, executable] of runtimes)
            succeeded(
              command(
                `package.${name}.run.${runtime}`,
                executable,
                [join(output, manifest.entry)],
                "parcel\nignored\n"
              ),
              "Line: parcel\n"
            )
        }
      }
    }
  }
  // The CLI's standalone web bundle succeeds; this is not host execution proof.
  succeeded(
    command(
      "qualified.standalone-web-bundle",
      cli,
      [
        "build",
        "main.ssrg",
        "--target",
        "web",
        "--profile",
        "release",
        "--out-dir",
        "standalone-web",
      ],
      "",
      join(evidence, "native/qualified-one-line")
    )
  )
})

test("committed WASM/current browser path proves the exact portable source and raw text callbacks", async () => {
  // Computed imports keep the site typecheck from pulling in the runtime tree.
  const runtimeRegistryPath = join(
    root,
    "apps/playground/src/runtime/runtime-modules.ts"
  )
  const browserHostPath = join(root, "runtime/ts/src/browser/host.ts")
  const effectRuntimePath = join(root, "runtime/ts/src/effect.ts")
  const { runtimeModules } = await import(runtimeRegistryPath)
  const { createBrowserEnvironment } = await import(browserHostPath)
  const { createEffectExecution, run } = await import(effectRuntimePath)
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  await bindings.default({
    module_or_path: readFileSync(
      join(root, "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm")
    ),
  })
  const { executeGeneratedModule } = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  const source = sample("one-line")
  const seed = new URL(source.playgroundUrl).searchParams.get("source")!
  expect(seed).toBe(source.source)
  const compiled = JSON.parse(
    bindings.compile_single_file("main.ssrg", "playground/main", seed)
  )
  save("one-line.wasm-compiled.json", compiled)
  save("wasm-version.json", JSON.parse(bindings.toolchain_version_json()))
  expect(compiled.status, JSON.stringify(compiled.diagnostics)).toBe("success")
  expect(compiled.entry.environment).toEqual([
    { field: "console", service: "console" },
    { field: "stdin", service: "stdin" },
  ])
  expect(compiled.entry.providers ?? []).toEqual([])
  const javascript = ts.transpileModule(compiled.generated.typescript, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      strict: true,
    },
  }).outputText
  const module = { exports: {} as Record<string, unknown> }
  new Function("require", "module", "exports", javascript)(
    (specifier: string) => {
      const resolved = (runtimeModules as Record<string, unknown>)[specifier]
      if (resolved === undefined)
        throw new Error(`Unsupported runtime import: ${specifier}`)
      return resolved
    },
    module,
    module.exports
  )
  const results = []
  for (const [name, input, expected] of [
    ...textCases,
    ["bare-cr", "alpha\rbeta\n", "Line: alpha\rbeta\n"],
  ]) {
    const production = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry,
      input
    )
    expect(production).toEqual({ stdout: expected!.trimEnd(), debug: "()" })
    const chunks: string[] = []
    const execution = createEffectExecution()
    try {
      const host = createBrowserEnvironment(
        compiled.entry.environment,
        input,
        (text: string) => chunks.push(text),
        undefined,
        execution.context
      )
      const main = module.exports.main as (
        unit: undefined
      ) => Parameters<typeof run>[0]
      const outcome = await run(main(undefined), host, execution.context)
      expect(outcome.kind).toBe("success")
      expect(chunks.join("")).toBe(expected)
      results.push({
        name,
        input,
        expected,
        productionBrowserExecution: production,
        supplementalRawHost: {
          chunks,
          stdout: chunks.join(""),
          status: outcome.kind,
        },
      })
    } finally {
      await execution.close()
    }
    save("wasm-results.json", {
      sourceSha256: source.sha256,
      generatedTypeScriptSha256: sha256(compiled.generated.typescript),
      entry: compiled.entry,
      results,
    })
  }
})

test("one-line readers answer before a bounded synthetic pipe closes", async () => {
  const output = compileTypeScript()
  const results = []
  for (const [runtime, executable] of runtimes) {
    for (const [kind, entry, early] of [
      ["seseragi", buildSource("one-line"), true],
      ["readline", join(output, "native/one-line.js"), true],
      ["whole-input", join(output, "proof-only/complete-input.js"), false],
    ] as const) {
      const child = spawn(executable, [entry], {
        cwd: evidence,
        env: environment,
        stdio: ["pipe", "pipe", "pipe"],
      })
      const stdout: Buffer[] = []
      const stderr: Buffer[] = []
      child.stdout.on("data", (data: Buffer) => stdout.push(data))
      child.stderr.on("data", (data: Buffer) => stderr.push(data))
      const completion = new Promise<{
        code: number | null
        signal: string | null
      }>((resolve, reject) => {
        child.once("error", reject)
        child.once("close", (code, signal) => resolve({ code, signal }))
      })
      const kill = setTimeout(() => child.kill("SIGKILL"), 8_000)
      let beforeEof = ""
      try {
        child.stdin.write("parcel\nignored\n")
        await new Promise<void>((resolve) => setTimeout(resolve, 500))
        beforeEof = Buffer.concat(stdout).toString()
        child.stdin.end()
        const completed = await completion
        const actual = Buffer.concat(stdout).toString()
        const error = Buffer.concat(stderr).toString()
        results.push({
          runtime,
          kind,
          executable,
          args: [entry],
          environment,
          heldOpenMs: 500,
          completionTimeoutMs: 8000,
          beforeEof,
          stdout: actual,
          stderr: error,
          ...completed,
        })
        save("staged-input.json", results)
        expect(completed).toEqual({ code: 0, signal: null })
        expect(error).toBe("")
        expect(beforeEof).toBe(early ? "Line: parcel\n" : "")
        expect(actual).toBe(
          early ? "Line: parcel\n" : "Complete input: [parcel\nignored\n]\n"
        )
      } finally {
        child.stdin.end()
        clearTimeout(kill)
        if (child.exitCode === null) child.kill("SIGKILL")
      }
    }
  }
})
