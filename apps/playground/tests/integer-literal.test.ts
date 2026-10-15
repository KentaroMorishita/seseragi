import { beforeAll, expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import init, {
  compile_single_file,
  format_single_file,
} from "../src/wasm/pkg/seseragi_wasm"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})

test("executes canonical integer bases, signed patterns and safe boundaries through WASM", async () => {
  const fixture = new URL(
    "../../../examples/spec/fixtures/rust-backend/integer-literals.ssrg",
    import.meta.url
  )
  const source = `${await Bun.file(fixture).text()}\npub effect fn main = println (show (probe ()))\n`
  const expected = await Bun.file(
    new URL("./integer-literals.stdout", fixture)
  ).text()
  const response = JSON.parse(
    compile_single_file("numbers.ssrg", "fixture/numbers", source)
  ) as CompileResponse
  expect(response.status).toBe("success")
  if (response.status !== "success" || !response.entry) {
    throw new Error("missing integer literal execution entry")
  }
  expect(
    await executeGeneratedModule(response.generated.typescript, response.entry)
  ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
  const formatted = JSON.parse(format_single_file("numbers.ssrg", source))
  expect(formatted.status).toBe("success")
  for (const spelling of [
    "0b1010_0110",
    "0O7_55",
    "0Xff_00",
    "0x1F_FFFF_FFFF_FFFF",
  ]) {
    expect(formatted.source).toContain(spelling)
  }
  expect(
    JSON.parse(format_single_file("numbers.ssrg", formatted.source))
  ).toEqual({
    ...formatted,
    changed: false,
  })
})

test("rejects complete malformed numeric tokens with UTF-8 positions before generation", () => {
  for (const raw of [
    "0b102",
    "0o78",
    "0xFFz",
    "0x_FF",
    "42suffix",
    "0x1.fp+2",
    "0x20000000000000",
  ]) {
    const prefix = "pub let 値: Int = "
    const source = `${prefix}${raw}\n`
    const response = JSON.parse(
      compile_single_file("invalid.ssrg", "fixture/invalid", source)
    ) as CompileResponse
    expect(response.status).toBe("failure")
    expect("generated" in response).toBe(false)
    expect(response.diagnostics.diagnostics).toHaveLength(1)
    expect(response.diagnostics.diagnostics[0]).toMatchObject({
      code: "SES-P0203",
      primary: {
        start: new TextEncoder().encode(prefix).length,
        end: new TextEncoder().encode(prefix + raw).length,
      },
    })
    expect(JSON.parse(format_single_file("invalid.ssrg", source)).status).toBe(
      "failure"
    )
  }
})
