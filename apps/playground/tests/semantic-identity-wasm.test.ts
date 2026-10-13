import { beforeAll, expect, test } from "bun:test"
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { checkGeneratedTypes } from "../../../scripts/check-generated-types"
import type {
  ProjectCompileResponse,
  ProjectRequest,
} from "../src/compiler/types"
import { executeGeneratedProject } from "../src/runtime/browser-execution"
import init, { compile_project } from "../src/wasm/pkg/seseragi_wasm"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})

function compile(source: string): ProjectCompileResponse {
  return compileFiles([{ path: "main.ssrg", source }])
}

function compileFiles(files: ProjectRequest["files"]): ProjectCompileResponse {
  const request: ProjectRequest = {
    schema: 1,
    entry: "main.ssrg",
    files,
  }
  return JSON.parse(compile_project(JSON.stringify(request)))
}

function reject(source: string, key: string) {
  const result = compile(source)
  expect(result.status).toBe("failure")
  if (result.status !== "failure") throw new Error("invalid source compiled")
  expect(
    result.diagnostics.flatMap(({ diagnostics }) =>
      diagnostics.diagnostics.map(({ messageKey }) => messageKey)
    )
  ).toContain(key)
}

test("WASM rejects unproved recursive and large product matches", () => {
  reject(
    "type Tree = | Leaf Int | Branch Tree\npub fn main tree: Tree -> Int = match tree { Leaf value -> value }\n",
    "match.non-exhaustive"
  )
  const types = Array(13).fill("Bool").join(", ")
  const values = Array(13).fill("True").join(", ")
  reject(
    `pub fn main flags: (${types}) -> Int = match flags { (${values}) -> 1 }\n`,
    "match.non-exhaustive"
  )
})

test("WASM rejects excessive flat patterns without trapping", () => {
  for (const length of [256, 10_000]) {
    const elements = Array(length).fill("_").join(", ")
    reject(
      `pub fn inspect values: Array<Int> -> Int = match values { [${elements}] -> 1; _ -> 0 }\n`,
      "match.coverage-limit"
    )
  }
})

test("WASM emits valid nominal annotations beside genuine scalar values", async () => {
  const names = ["Unit", "Bool", "Int", "Float", "Char", "String"]
  const genuine =
    'let genuineUnit = ()\nlet genuineBool = True\nlet genuineInt = 42\nlet genuineFloat = 1.5\nlet genuineChar = \'x\'\nlet genuineString = "hello"\npub effect fn main = do { println "audit" }\n'
  const declarations = (visibility: string) =>
    `${visibility}struct Marker { tokens: Array<Never> }\n${names
      .map((name) => `${visibility}struct ${name} { value: Marker }`)
      .join("\n")}\n`
  const readers = (prefix: string) =>
    names
      .map(
        (name) =>
          `pub fn read${name} value: ${prefix}${name} -> ${prefix}Marker = value.value`
      )
      .join("\n")
  const cases: ProjectRequest["files"][] = [
    [
      {
        path: "main.ssrg",
        source: `${declarations("")}${readers("")}\n${genuine}`,
      },
    ],
    [
      {
        path: "main.ssrg",
        source: `import * as domain from "./domain"\n${readers("domain.")}\n${genuine}`,
      },
      { path: "domain.ssrg", source: declarations("pub ") },
    ],
    [
      {
        path: "main.ssrg",
        source: `${declarations("")
          .replaceAll(
            /struct (Unit|Bool|Int|Float|Char|String) \{/g,
            "struct $1<A> {"
          )
          .replaceAll("value: Marker", "value: A")}\n${names
          .map(
            (name) =>
              `pub fn read${name} value: ${name}<Marker> -> Marker = value.value`
          )
          .join("\n")}\n${genuine}`,
      },
    ],
    ...["pub type Bool = | Flag Int", "pub newtype Bool = Int"].map(
      (declaration) => [
        {
          path: "domain.ssrg",
          source: `${declaration}\n`,
        },
        {
          path: "main.ssrg",
          source: `import {${declaration.includes("Flag") ? "Bool, Flag" : "Bool"}} from "./domain"\npub fn read value: Bool -> Int = match value { ${declaration.includes("Flag") ? "Flag" : "Bool"} amount -> amount }\n${genuine}`,
        },
      ]
    ),
    ...["Array", "List"].map((name) => [
      {
        path: "main.ssrg",
        source: `struct Marker { tokens: Array<Never> }\nstruct ${name}<A> { value: A }\npub fn read value: ${name}<Marker> -> Marker = value.value\nlet genuineArray = [1]\nlet genuineList = \`[1]\nlet nestedList = {let item = 1; \`[item]}\nlet comprehensionList = \`[value | value <- \`[1]]\n${genuine}`,
      },
    ]),
    [
      { path: "domain.ssrg", source: declarations("pub ") },
      {
        path: "bridge.ssrg",
        source: `import * as domain from "./domain"\n${names
          .map((name) => `pub alias Alias${name} = domain.${name}`)
          .join("\n")}\n`,
      },
      {
        path: "main.ssrg",
        source: `import * as domain from "./domain"\nimport {${names
          .map((name) => `Alias${name} as Renamed${name}`)
          .join(", ")}} from "./bridge"\n${names
          .map(
            (name) =>
              `pub fn read${name} value: Renamed${name} -> domain.Marker = value.value`
          )
          .join("\n")}\n${genuine}`,
      },
    ],
  ]
  for (const files of cases) {
    const result = compileFiles(files)
    expect(result.status).toBe("success")
    if (result.status !== "success") throw new Error(JSON.stringify(result))
    const generated = result.modules.find(({ path }) => path === "main.ssrg")!
      .generated.typescript
    for (const [binding, type] of [
      ["genuineUnit", "undefined"],
      ["genuineBool", "boolean"],
      ["genuineInt", "number"],
      ["genuineFloat", "number"],
      ["genuineChar", "string"],
      ["genuineString", "string"],
    ]) {
      expect(generated).toContain(`${binding}: ${type}`)
    }
    const directory = await mkdtemp(join(tmpdir(), "seseragi-nominal-wasm-"))
    try {
      await symlink(
        fileURLToPath(new URL("../../../node_modules", import.meta.url)),
        join(directory, "node_modules"),
        "dir"
      )
      for (const module of result.modules) {
        const path = join(directory, module.path.replace(/\.ssrg$/, ".ts"))
        await mkdir(dirname(path), { recursive: true })
        await writeFile(path, module.generated.typescript)
      }
      checkGeneratedTypes(directory)
    } finally {
      await rm(directory, { recursive: true, force: true })
    }
  }
}, 20_000)

test("WASM keeps primitive and nominal argument identities through effect lets", () => {
  reject(
    'struct Bool {value: Int}\nfn read value: Bool -> Int = value.value\npub effect fn main = do {let selected = read True; println "unreachable"}\n',
    "call.argument-type-mismatch"
  )
  for (const [left, right] of [
    ["True", "Bool {value: 1}"],
    ["Bool {value: 1}", "True"],
  ]) {
    reject(
      `struct Bool {value: Int}\nfn first<A> left: A -> right: A -> A = left\npub effect fn main = do {let selected = first (${left}) (${right}); println "unreachable"}\n`,
      "call.argument-type-mismatch"
    )
  }
})

test("WASM executes signed and nested collection patterns", async () => {
  const result = compile(
    "pub fn inspect values: Array<Array<Int>> -> Int = match values { [[-1, ...tail], ...rest] -> -1; _ -> 0 }\npub effect fn main = do {println (show (inspect [[-1, 2]]))}\n"
  )
  expect(result.status).toBe("success")
  if (result.status !== "success") throw new Error(JSON.stringify(result))
  const executed = await executeGeneratedProject(
    result.modules.map(({ path, generated }) => ({
      path,
      typescript: generated.typescript,
    })),
    result.entry.path,
    result.entry.contract!,
    ""
  )
  expect(executed.stdout).toBe("-1")
})

test("WASM preserves genuine and nominal succeed values through effect binds", async () => {
  reject(
    'struct Bool {value: Int}\npub effect fn main = do {flag <- succeed (Bool {value: 1}); if flag then println "unreachable" else println "unreachable"}\n',
    "if.condition-not-bool"
  )
  const result = compile(
    'struct Bool {value: Int}\nfn first<A> left: A -> right: A -> A = left\npub effect fn main = do {flag <- succeed True; if first True flag then println "true" else println "false"}\n'
  )
  expect(result.status).toBe("success")
  if (result.status !== "success") throw new Error(JSON.stringify(result))
  const executed = await executeGeneratedProject(
    result.modules.map(({ path, generated }) => ({
      path,
      typescript: generated.typescript,
    })),
    result.entry.path,
    result.entry.contract!,
    ""
  )
  expect(executed.stdout).toBe("true")
})

test("WASM execution selects primitive and nominal dictionaries independently", async () => {
  const result = compile(
    'struct Bool deriving Eq {value: Int}\nstruct Float deriving Show {value: String}\npub effect fn main = do {if True == False then println "wrong" else println "primitive"; if Bool {value: 1} == Bool {value: 2} then println "wrong" else println "nominal"; println (show (-1.0)); println (show [(-1.0)]); println (show {x: (-1.0)}); println (show (Float {value: "owned"}))}\n'
  )
  expect(result.status).toBe("success")
  if (result.status !== "success") throw new Error(JSON.stringify(result))
  const executed = await executeGeneratedProject(
    result.modules.map(({ path, generated }) => ({
      path,
      typescript: generated.typescript,
    })),
    result.entry.path,
    result.entry.contract!,
    ""
  )
  const lines = executed.stdout.trimEnd().split("\n")
  expect(lines.slice(0, 3)).toEqual(["primitive", "nominal", "-1.0"])
  expect(lines[3]).toContain("-1.0")
  expect(lines[4]).toContain("-1.0")
  expect(lines[5]).toContain("value: owned")
})
