import assert from "node:assert/strict"
import { mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { entranceComparison, shippingCases } from "./comparisons"

// Called by check-examples.ts, alongside the existing site-owned examples.
export function checkComparisons(cli: string, root: string, temporary: string) {
  const directory = join(temporary, "comparisons")
  mkdirSync(directory)
  const source = {
    seseragi: readFileSync(join(root, entranceComparison.seseragi), "utf8"),
    typescript: readFileSync(join(root, entranceComparison.typescript), "utf8"),
  }
  const expected = readFileSync(
    join(root, entranceComparison.expectedOutput),
    "utf8"
  )
  const run = (command: string[]) =>
    Bun.spawnSync(command, { cwd: root, stdout: "pipe", stderr: "pipe" })
  const success = (command: string[]) => {
    const result = run(command)
    assert.equal(
      result.exitCode,
      0,
      `${command.join(" ")}\n${result.stdout}${result.stderr}`
    )
    return result.stdout.toString()
  }
  const typecheck = (path: string) => [
    join(root, "node_modules/.bin/tsc"),
    "--strict",
    "--noEmit",
    "--skipLibCheck",
    "--types",
    "bun",
    "--typeRoots",
    join(root, "node_modules/@types"),
    "--target",
    "ES2022",
    "--module",
    "Preserve",
    "--moduleResolution",
    "Bundler",
    path,
  ]
  const fixture = (name: string, extension: string, text: string) => {
    const path = join(directory, `${name}.${extension}`)
    writeFileSync(path, text)
    return path
  }

  // Execute the exact panel/Playground bytes, not a hand-copied function.
  const ss = fixture("shipping-total", "ssrg", source.seseragi)
  const ts = fixture("shipping-total", "ts", source.typescript)
  success([cli, "format", "--check", ss])
  success([cli, "build", ss, "--out-dir", join(directory, "compiled")])
  success(typecheck(ts))
  assert.equal(success([cli, "run", ss]), expected)
  assert.equal(success(["bun", ts]), expected)

  // Preserve all declarations; replace only the output line for edge cases.
  // The published source above remains a separately checked complete program.
  const ssOutput = source.seseragi.indexOf("pub effect fn main =")
  const tsOutput = source.typescript.indexOf("console.log(")
  assert.ok(ssOutput > 0 && tsOutput > 0, "Missing comparison output boundary")
  const ssDeclarations = source.seseragi.slice(0, ssOutput)
  const tsDeclarations = source.typescript.slice(0, tsOutput)
  const ssCases = shippingCases.flatMap(({ freeFrom, fee, subtotal }) => [
    `totalWithShipping ${freeFrom} ${fee} ${subtotal}`,
    `(totalWithShipping ${freeFrom} ${fee}) ${subtotal}`,
  ])
  const tsCases = shippingCases.flatMap(({ freeFrom, fee, subtotal }) => [
    `totalWithShipping(${freeFrom}, ${fee}, ${subtotal})`,
    `((amount: number) => totalWithShipping(${freeFrom}, ${fee}, amount))(${subtotal})`,
  ])
  const ssMatrix = fixture(
    "shipping-cases",
    "ssrg",
    `${ssDeclarations}pub effect fn main = println (show [${ssCases.join(", ")}])\n`
  )
  const tsMatrix = fixture(
    "shipping-cases",
    "ts",
    `${tsDeclarations}console.log(JSON.stringify([${tsCases.join(", ")}]))\n`
  )
  success(typecheck(tsMatrix))
  const expectedCases = shippingCases.flatMap(({ expected }) => [
    expected,
    expected,
  ])
  assert.deepEqual(JSON.parse(success([cli, "run", ssMatrix])), expectedCases)
  assert.deepEqual(JSON.parse(success(["bun", tsMatrix])), expectedCases)

  const ssMistake = fixture(
    "shipping-wrong-type",
    "ssrg",
    `${ssDeclarations}let invalidSubtotal = standardTotal "3200"\n${source.seseragi.slice(ssOutput)}`
  )
  const tsMistake = fixture(
    "shipping-wrong-type",
    "ts",
    `${source.typescript}\nstandardTotal("3200")\n`
  )
  const rejectedSs = run([
    cli,
    "build",
    ssMistake,
    "--out-dir",
    join(directory, "rejected"),
  ])
  assert.notEqual(rejectedSs.exitCode, 0)
  assert.match(rejectedSs.stderr.toString(), /SES-T0101/u)
  assert.match(rejectedSs.stderr.toString(), /expected Int, received String/u)
  const rejectedTs = run(typecheck(tsMistake))
  assert.notEqual(rejectedTs.exitCode, 0)
  assert.match(rejectedTs.stdout.toString(), /TS2345/u)

  // The shared domain is a promise by the example, not an input validator.
  // number accepts a decimal that Int rejects. Keep that difference explicit.
  const ssDecimal = fixture(
    "shipping-decimal",
    "ssrg",
    `${ssDeclarations}let decimalSubtotal = standardTotal 3200.5\n${source.seseragi.slice(ssOutput)}`
  )
  const rejectedDecimal = run([
    cli,
    "build",
    ssDecimal,
    "--out-dir",
    join(directory, "rejected-decimal"),
  ])
  assert.notEqual(rejectedDecimal.exitCode, 0)
  assert.match(
    rejectedDecimal.stderr.toString(),
    /expected Int, received Float/u
  )
  const tsDecimal = fixture(
    "shipping-decimal",
    "ts",
    `${tsDeclarations}console.log(standardTotal(3200.5))\n`
  )
  success(typecheck(tsDecimal))
  assert.equal(success(["bun", tsDecimal]), "3700.5\n")

  const ssOverflow = fixture(
    "shipping-overflow",
    "ssrg",
    `${ssDeclarations}pub effect fn main = println (show (totalWithShipping 9007199254740991 500 9007199254740990))\n`
  )
  success([cli, "build", ssOverflow, "--out-dir", join(directory, "overflow")])
  const overflow = run([cli, "run", ssOverflow])
  assert.notEqual(overflow.exitCode, 0)
  assert.match(overflow.stderr.toString(), /runtime defect/u)
  const tsOverflow = fixture(
    "shipping-overflow",
    "ts",
    `${tsDeclarations}console.log(totalWithShipping(9007199254740991, 500, 9007199254740990))\n`
  )
  success(typecheck(tsOverflow))
  assert.equal(success(["bun", tsOverflow]), "9007199254741490\n")
  console.info(
    `Verified shipping-total: identical output, ${shippingCases.length} direct/partial cases, shared type error, decimal/overflow differences`
  )
}
