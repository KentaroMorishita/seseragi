import { beforeAll, expect, test } from "bun:test"
import type {
  CompileResponse,
  ProjectCompileResponse,
} from "../src/compiler/types"
import {
  executeGeneratedModule,
  executeGeneratedProject,
} from "../src/runtime/browser-execution"
import init, {
  compile_project,
  compile_single_file,
} from "../src/wasm/pkg/seseragi_wasm"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})

test("alias heads keep selected dictionaries, closures and resource lifetimes", async () => {
  for (const name of [
    "instance-alias",
    "instance-disjoint",
    "constructor-disjoint",
    "alias-constructor",
    "local-dictionaries",
    "instance-alias-effects",
    "local-dictionary-effects",
  ]) {
    const base = new URL(
      `../../../examples/spec/fixtures/rust-backend/${name}`,
      import.meta.url
    )
    let source = await Bun.file(`${base.pathname}.ssrg`).text()
    if (!name.endsWith("effects"))
      source += "\npub effect fn main = println (show (probe ()))\n"
    const expected = (
      await Bun.file(`${base.pathname}.stdout`).text()
    ).trimEnd()
    const response = JSON.parse(
      compile_single_file("λ.ssrg", "fixture/instance-alias", source)
    ) as CompileResponse
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error(JSON.stringify(response))
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected, debug: "()" })
  }
})

test("unused duplicate and unrepresented alias heads fail before generation", () => {
  for (const [declarations, key] of [
    [
      'type Box<A> = | Box A\nalias Wrapped<A> = Box<A>\ninstance<A> Show<Wrapped<A>> { fn show value: Box<A> -> String = "generic" }\ninstance Show<Box<Int>> { fn show value: Box<Int> -> String = "int" }\n',
      "trait.instance-duplicate",
    ],
    [
      'type Box<A> = | Box A\ninstance<A> Add<Box<A>, Int, Int> { fn add left: Box<A> -> right: Int -> Int = right }\ninstance Add<Box<Bool>, Int, String> { fn add left: Box<Bool> -> right: Int -> String = "bad" }\n',
      "trait.instance-duplicate",
    ],
    [
      'type Label = | Label\nalias Name = Label\ninstance Show<Name> { fn show value: Label -> String = "alias" }\ninstance Show<Label> { fn show value: Label -> String = "label" }\n',
      "trait.instance-duplicate",
    ],
    [
      'alias Number = Int\ninstance Show<Number> { fn show value: Int -> String = "number" }\n',
      "trait.instance-duplicate",
    ],
    [
      'type Label = | Label\nalias Erased<A> = Label\ninstance<A> Show<Erased<A>> { fn show value: Label -> String = "label" }\n',
      "trait.instance-head-unresolved",
    ],
    [
      'type Pair<A, B> = | Pair (A, B)\nalias Repeated<A> = Pair<A, A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Repeated> { fn tag<A> value: Pair<A, A> -> String = "box" }\n',
      "trait.instance-head-unresolved",
    ],
  ]) {
    const source = `// 雫🙂\n${declarations}\npub effect fn main = println "MUST_NOT_RUN"\n`
    const response = JSON.parse(
      compile_single_file("λ.ssrg", "fixture/alias-negative", source)
    ) as CompileResponse
    expect(response.status).toBe("failure")
    expect("generated" in response).toBe(false)
    expect(
      response.diagnostics.diagnostics.some((d) => d.messageKey === key)
    ).toBe(true)
    const bytes = new TextEncoder().encode(source)
    for (const diagnostic of response.diagnostics.diagnostics) {
      expect(diagnostic.primary.start).toBeGreaterThanOrEqual(0)
      expect(diagnostic.primary.end).toBeLessThanOrEqual(bytes.length)
      expect(() =>
        new TextDecoder("utf-8", { fatal: true }).decode(
          bytes.slice(diagnostic.primary.start, diagnostic.primary.end)
        )
      ).not.toThrow()
    }
  }
})

test("dictionary heads retain provider owners across renamed facade exports", async () => {
  for (const name of [
    "instance-alias",
    "alias-constructor",
    "imported-return",
    "local-dictionaries",
  ]) {
    const base = new URL(
      `../../../examples/spec/fixtures/rust-backend-projects/${name}/`,
      import.meta.url
    )
    const paths = [
      ["main.ssrg", "src/main.ssrg"],
      ["facade.ssrg", "src/facade.ssrg"],
      ["model.ssrg", "vendor/model/src/lib.ssrg"],
    ] as const
    const files = await Promise.all(
      paths.map(async ([path, file]) => ({
        path,
        source: (await Bun.file(new URL(file, base)).text()).replaceAll(
          'from "model"',
          'from "./model"'
        ),
      }))
    )
    const manifest =
      '[package]\nname="fixture/alias-projection"\nversion="0.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\n'
    const response = JSON.parse(
      compile_project(JSON.stringify({ schema: 1, manifest, files }))
    ) as ProjectCompileResponse
    expect(response.status).toBe("success")
    if (response.status !== "success" || response.entry.contract === undefined)
      throw new Error(JSON.stringify(response))
    expect(response.modules).toHaveLength(3)
    const expected = (
      await Bun.file(new URL("expected.stdout", base)).text()
    ).trimEnd()
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({
      stdout: expected,
      debug: "()",
    })
  }
})

test("constructor overlap and unproved HKT heads fail without generated code", async () => {
  for (const [fixture, key] of [
    ["constructor-section-duplicate", "trait.instance-duplicate"],
    ["constructor-coherence-unproved", "trait.instance-coherence-unproved"],
    ["alias-constructor-duplicate", "trait.instance-duplicate"],
    ["alias-constructor-noneta", "trait.instance-head-unresolved"],
    ["alias-method-hole-contract", "trait.instance-method-signature-mismatch"],
  ]) {
    const source = await Bun.file(
      new URL(
        `../../../examples/spec/artifacts/semantic-diagnostics-schema-1/${fixture}/main.ssrg`,
        import.meta.url
      )
    ).text()
    const response = JSON.parse(
      compile_single_file("雫.ssrg", "fixture/constructor-coherence", source)
    ) as CompileResponse
    expect(response.status).toBe("failure")
    expect("generated" in response).toBe(false)
    const error = response.diagnostics.diagnostics.find(
      (diagnostic) => diagnostic.messageKey === key
    )
    expect(error).toBeDefined()
    if (!error) throw new Error(JSON.stringify(response))
    const bytes = new TextEncoder().encode(source)
    expect(
      new TextDecoder("utf-8", { fatal: true }).decode(
        bytes.slice(error.primary.start, error.primary.end)
      )
    ).toStartWith(fixture === "alias-method-hole-contract" ? "fn" : "instance")
  }
})
