import { mkdir, readFile, writeFile } from "node:fs/promises"
import { dirname, resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const sourceRoot = resolve(root, "runtime/unicode")
type Manifest = {
  version: string
  baseUrl: string
  files: { file: string; sha256: string }[]
}
type Range = [number, number, number]
type Mapping = Record<number, number[]>

export const categoryNames = [
  "UppercaseLetter",
  "LowercaseLetter",
  "TitlecaseLetter",
  "ModifierLetter",
  "OtherLetter",
  "NonspacingMark",
  "SpacingMark",
  "EnclosingMark",
  "DecimalNumber",
  "LetterNumber",
  "OtherNumber",
  "ConnectorPunctuation",
  "DashPunctuation",
  "OpenPunctuation",
  "ClosePunctuation",
  "InitialPunctuation",
  "FinalPunctuation",
  "OtherPunctuation",
  "MathSymbol",
  "CurrencySymbol",
  "ModifierSymbol",
  "OtherSymbol",
  "SpaceSeparator",
  "LineSeparator",
  "ParagraphSeparator",
  "Control",
  "Format",
  "PrivateUse",
  "Unassigned",
] as const
const categoryCodes = [
  "Lu",
  "Ll",
  "Lt",
  "Lm",
  "Lo",
  "Mn",
  "Mc",
  "Me",
  "Nd",
  "Nl",
  "No",
  "Pc",
  "Pd",
  "Ps",
  "Pe",
  "Pi",
  "Pf",
  "Po",
  "Sm",
  "Sc",
  "Sk",
  "So",
  "Zs",
  "Zl",
  "Zp",
  "Cc",
  "Cf",
  "Co",
  "Cn",
]
export const graphemeClasses = [
  "Other",
  "CR",
  "LF",
  "Control",
  "Extend",
  "ZWJ",
  "Regional_Indicator",
  "Prepend",
  "SpacingMark",
  "L",
  "V",
  "T",
  "LV",
  "LVT",
] as const

function rows(text: string): string[][] {
  return text.split(/\r?\n/u).flatMap((line) => {
    const value = line.split("#")[0]!.trim()
    return value ? [value.split(";").map((field) => field.trim())] : []
  })
}

function codePoints(value: string): number[] {
  return value
    .split(" ")
    .filter(Boolean)
    .map((part) => Number.parseInt(part, 16))
}

function range(value: string): [number, number] {
  const [first, last = first] = value.split("..")
  return [Number.parseInt(first!, 16), Number.parseInt(last!, 16)]
}

function mergeRanges(input: Range[]): Range[] {
  const output: Range[] = []
  for (const item of input.sort((left, right) => left[0] - right[0])) {
    const previous = output.at(-1)
    if (previous && previous[1] + 1 === item[0] && previous[2] === item[2]) {
      previous[1] = item[1]
    } else output.push([...item])
  }
  return output
}

export function generateUnicodeTables(
  sources: Map<string, string>,
  version: string
) {
  const source = (name: string) => {
    const value = sources.get(name)
    if (value === undefined) throw new Error(`missing UCD source ${name}`)
    return rows(value)
  }
  const categories: Range[] = []
  const combining: Range[] = []
  const canonicalDecomposition: Mapping = {}
  const compatibilityDecomposition: Mapping = {}
  const lower: Mapping = {}
  const upper: Mapping = {}
  let firstRange: number | undefined
  for (const row of source("UnicodeData.txt")) {
    const point = Number.parseInt(row[0]!, 16)
    if (row[1]!.endsWith(", First>")) {
      firstRange = point
      continue
    }
    const start = firstRange ?? point
    firstRange = undefined
    const category = categoryCodes.indexOf(row[2]!)
    // Surrogates are not Seseragi scalars; the runtime rejects them at input.
    if (category >= 0) categories.push([start, point, category])
    const ccc = Number(row[3])
    if (ccc !== 0) combining.push([start, point, ccc])
    const decomposition = row[5]!
    if (decomposition.startsWith("<")) {
      compatibilityDecomposition[point] = codePoints(
        decomposition.slice(decomposition.indexOf(">") + 1).trim()
      )
    } else if (decomposition)
      canonicalDecomposition[point] = codePoints(decomposition)
    if (row[12]) upper[point] = codePoints(row[12])
    if (row[13]) lower[point] = codePoints(row[13])
  }
  const conditionalLower: Mapping = {}
  for (const row of source("SpecialCasing.txt")) {
    const point = Number.parseInt(row[0]!, 16)
    if (!row[4]) {
      lower[point] = codePoints(row[1]!)
      upper[point] = codePoints(row[3]!)
    } else if (row[4] === "Final_Sigma") {
      conditionalLower[point] = codePoints(row[1]!)
    } else if (!/^(lt|tr|az)( |$)/u.test(row[4])) {
      throw new Error(
        `unsupported locale-independent special casing: ${row.join(";")}`
      )
    }
  }
  const simpleFold: Mapping = {}
  const fullFold: Mapping = {}
  for (const row of source("CaseFolding.txt")) {
    const point = Number.parseInt(row[0]!, 16)
    if (row[1] === "C" || row[1] === "S")
      simpleFold[point] = codePoints(row[2]!)
    if (row[1] === "C" || row[1] === "F") fullFold[point] = codePoints(row[2]!)
  }
  const binary = new Map<string, Range[]>()
  const indic: Range[] = []
  for (const row of source("DerivedCoreProperties.txt")) {
    if (row[1] === "InCB") {
      const value = ["None", "Consonant", "Extend", "Linker"].indexOf(row[2]!)
      if (value < 0) throw new Error(`unknown InCB ${row[2]}`)
      indic.push([...range(row[0]!), value])
    } else if (
      [
        "Alphabetic",
        "Uppercase",
        "Cased",
        "Case_Ignorable",
        "XID_Start",
        "XID_Continue",
      ].includes(row[1]!)
    ) {
      const entries = binary.get(row[1]!) ?? []
      entries.push([...range(row[0]!), 1])
      binary.set(row[1]!, entries)
    }
  }
  for (const [file, property] of [
    ["PropList.txt", "White_Space"],
    ["emoji/emoji-data.txt", "Extended_Pictographic"],
  ]) {
    binary.set(
      property!,
      source(file!)
        .filter((row) => row[1] === property)
        .map((row) => [...range(row[0]!), 1])
    )
  }
  const excluded = new Set<number>()
  for (const row of source("DerivedNormalizationProps.txt")) {
    if (row[1] !== "Full_Composition_Exclusion") continue
    const [start, end] = range(row[0]!)
    for (let point = start; point <= end; point++) excluded.add(point)
  }
  const compositions: Record<string, number> = {}
  for (const [point, values] of Object.entries(canonicalDecomposition)) {
    if (values.length === 2 && !excluded.has(Number(point)))
      compositions[values.join(",")] = Number(point)
  }
  const graphemes: Range[] = source("auxiliary/GraphemeBreakProperty.txt").map(
    (row) => {
      const value = graphemeClasses.indexOf(
        row[1] as (typeof graphemeClasses)[number]
      )
      if (value < 0) throw new Error(`unknown grapheme class ${row[1]}`)
      return [...range(row[0]!), value]
    }
  )
  const declarations: [string, unknown, string][] = [
    [
      "CATEGORY_NAMES",
      categoryNames,
      `readonly ${JSON.stringify(categoryNames)}`,
    ],
    ["GENERAL_CATEGORY", mergeRanges(categories).flat(), "readonly number[]"],
    ["COMBINING_CLASS", mergeRanges(combining).flat(), "readonly number[]"],
    [
      "CANONICAL_DECOMPOSITION",
      canonicalDecomposition,
      "Readonly<Record<number, readonly number[]>>",
    ],
    [
      "COMPATIBILITY_DECOMPOSITION",
      compatibilityDecomposition,
      "Readonly<Record<number, readonly number[]>>",
    ],
    ["COMPOSITIONS", compositions, "Readonly<Record<string, number>>"],
    ["LOWER", lower, "Readonly<Record<number, readonly number[]>>"],
    ["UPPER", upper, "Readonly<Record<number, readonly number[]>>"],
    [
      "FINAL_SIGMA",
      conditionalLower,
      "Readonly<Record<number, readonly number[]>>",
    ],
    ["SIMPLE_FOLD", simpleFold, "Readonly<Record<number, readonly number[]>>"],
    ["FULL_FOLD", fullFold, "Readonly<Record<number, readonly number[]>>"],
    ["GRAPHEME_BREAK", mergeRanges(graphemes).flat(), "readonly number[]"],
    ["INDIC_CONJUNCT_BREAK", mergeRanges(indic).flat(), "readonly number[]"],
    ...[...binary].map(([name, entries]): [string, unknown, string] => [
      name.toUpperCase(),
      mergeRanges(entries).flat(),
      "readonly number[]",
    ]),
  ]
  const rustMappings = (name: string, mapping: Mapping) =>
    Object.keys(mapping).length <= 1
      ? `pub const ${name}: &[(u32, &[u32])] = &[${Object.entries(mapping)
          .map(([point, values]) => `(${point}, &[${values.join(", ")}])`)
          .join(", ")}];`
      : `pub const ${name}: &[(u32, &[u32])] = &[\n${Object.entries(mapping)
          .map(([point, values]) => `    (${point}, &[${values.join(", ")}]),`)
          .join("\n")}\n];`
  const rustRanges = (name: string) =>
    `pub const ${name.toUpperCase()}: &[[u32; 3]] = &[\n${mergeRanges(
      binary.get(name)!
    )
      .map((entry) => `    [${entry.join(", ")}],`)
      .join("\n")}\n];`
  return {
    typescript: `// Unicode ${version}, generated from runtime/unicode/manifest.json by scripts/generate-unicode.ts.\n// Unicode License v3; see UNICODE-LICENSE in the runtime package. Do not edit.\n\n${declarations.map(([name, value, type]) => `export const ${name}: ${type} = ${JSON.stringify(value)}`).join("\n\n")}\n`,
    rust: `// Unicode ${version}, generated by scripts/generate-unicode.ts; do not edit.\n// Unicode License v3; see runtime/unicode/LICENSE.\n\n${[rustMappings("LOWER", lower), rustMappings("FINAL_SIGMA", conditionalLower), ...["Cased", "Case_Ignorable", "White_Space", "Uppercase"].map(rustRanges)].join("\n\n")}\n`,
  }
}

async function writeOrCheck(file: string, value: string, check: boolean) {
  if (check) {
    if ((await readFile(file, "utf8")) !== value)
      throw new Error(
        `stale Unicode projection: ${file}; run bun run unicode:generate`
      )
  } else {
    await mkdir(dirname(file), { recursive: true })
    await writeFile(file, value)
  }
}

if (import.meta.main) {
  const args = process.argv.slice(2)
  if (
    args.some((value) => !["--download", "--check"].includes(value)) ||
    (args.includes("--download") && args.includes("--check"))
  ) {
    throw new Error("usage: generate-unicode.ts [--download | --check]")
  }
  const manifest = JSON.parse(
    await readFile(resolve(sourceRoot, "manifest.json"), "utf8")
  ) as Manifest
  if (
    !/^\d+\.\d+\.\d+$/u.test(manifest.version) ||
    manifest.baseUrl !==
      `https://www.unicode.org/Public/${manifest.version}/ucd/`
  )
    throw new Error("invalid pinned Unicode version / origin")
  const sources = new Map<string, string>()
  for (const entry of manifest.files) {
    if (entry.file.includes("..") || entry.file.startsWith("/"))
      throw new Error("invalid UCD path")
    const file = resolve(sourceRoot, "ucd", entry.file)
    let content: Uint8Array
    if (args.includes("--download")) {
      const response = await fetch(new URL(entry.file, manifest.baseUrl))
      if (!response.ok)
        throw new Error(`UCD download ${entry.file}: ${response.status}`)
      content = new Uint8Array(await response.arrayBuffer())
    } else content = await readFile(file)
    const digest = new Bun.CryptoHasher("sha256").update(content).digest("hex")
    if (digest !== entry.sha256)
      throw new Error(`UCD checksum mismatch: ${entry.file}`)
    if (args.includes("--download")) {
      await mkdir(dirname(file), { recursive: true })
      await writeFile(file, content)
    }
    sources.set(entry.file, new TextDecoder().decode(content))
  }
  const check = args.includes("--check")
  const tables = generateUnicodeTables(sources, manifest.version)
  await writeOrCheck(
    resolve(root, "runtime/ts/src/unicode-data.ts"),
    tables.typescript,
    check
  )
  await writeOrCheck(
    resolve(root, "crates/seseragi-syntax/src/unicode_data.rs"),
    tables.rust,
    check
  )
  await writeOrCheck(
    resolve(root, "runtime/ts/src/unicode-version-data.ts"),
    `// Generated from runtime/unicode/manifest.json; do not edit.\nexport const UNICODE_VERSION = "${manifest.version}"\n`,
    check
  )
  await writeOrCheck(
    resolve(root, "runtime/ts/UNICODE-LICENSE"),
    await readFile(resolve(sourceRoot, "LICENSE"), "utf8"),
    check
  )
  await writeOrCheck(
    resolve(root, "apps/playground/public/UNICODE-LICENSE.txt"),
    await readFile(resolve(sourceRoot, "LICENSE"), "utf8"),
    check
  )
  await writeOrCheck(
    resolve(root, "crates/seseragi-release/src/unicode_version.rs"),
    `// Generated from runtime/unicode/manifest.json; do not edit.\npub const UNICODE_VERSION: &str = "${manifest.version}";\npub const UNICODE_VERSION_TUPLE: (u8, u8, u8) = (${manifest.version.split(".").join(", ")});\n`,
    check
  )
  console.log(
    `Unicode ${manifest.version}: ${manifest.files.length} pinned sources ${check ? "verified" : "projected"}.`
  )
}
