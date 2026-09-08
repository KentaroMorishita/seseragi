import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import {
  packageNativeBinary,
  verifyPackage,
} from "../extensions/seseragi/scripts/verify-package"
import { assertLinuxAbi, assertNativeBuildIdentity } from "./linux-native-abi"
import { verifyNativeRelease } from "./native-release"

function run(command: string[], cwd?: string, stdin?: Buffer): string {
  const result = Bun.spawnSync(command, {
    cwd,
    stdin,
    stdout: "pipe",
    stderr: "pipe",
    timeout: 30_000,
  })
  if (!result.success) throw new Error(`${command.join(" ")}: ${result.stderr}`)
  return result.stdout.toString()
}

const [archive, vsix, version, release] = process.argv.slice(2)
if (!archive || !vsix || !version || (release && release !== "--release")) {
  throw new Error(
    "usage: linux-native-smoke.ts ARCHIVE VSIX VERSION [--release]"
  )
}
await verifyNativeRelease({
  archive,
  version,
  target: "linux-x64",
  release: release === "--release",
})
await verifyPackage(vsix, "linux-x64")
const directory = await mkdtemp(path.join(tmpdir(), "seseragi-linux-abi-"))
try {
  run(["tar", "-xzf", path.resolve(archive), "-C", directory])
  const cli = path.join(directory, "seseragi")
  const lsp = path.join(directory, "seseragi-lsp")
  const cliMetadata = JSON.parse(run([cli, "--version-json"]))
  const lspMetadata = JSON.parse(run([lsp, "--version-json"]))
  assertNativeBuildIdentity(cliMetadata, lspMetadata)
  assertLinuxAbi(cli)
  assertLinuxAbi(lsp)
  const bundled = await packageNativeBinary(vsix, "linux-x64")
  if (!bundled || !Buffer.from(bundled).equals(await readFile(lsp))) {
    throw new Error("VSIX LSP differs from the native archive LSP")
  }
  const messages = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: { capabilities: {} },
    },
    { jsonrpc: "2.0", id: 2, method: "shutdown" },
    { jsonrpc: "2.0", method: "exit" },
  ]
  const input = Buffer.from(
    messages
      .map((message) => {
        const body = JSON.stringify(message)
        return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`
      })
      .join("")
  )
  const output = run([lsp], directory, input)
  if (!output.includes('"capabilities"') || !output.includes('"serverInfo"')) {
    throw new Error("downloaded LSP did not initialize")
  }
  const source = path.join(directory, "main.ssrg")
  await writeFile(
    source,
    'pub effect fn main = do { println "linux ABI smoke" }\n'
  )
  const invalid = path.join(directory, "invalid.ssrg")
  await writeFile(invalid, 'pub let value: Int = "wrong"\n')
  const rejected = Bun.spawnSync(
    [cli, "build", invalid, "--out-dir", path.join(directory, "invalid-built")],
    { stdout: "pipe", stderr: "pipe", timeout: 30_000 }
  )
  if (rejected.success || !rejected.stderr.toString().includes("SES-T0101")) {
    throw new Error("downloaded CLI did not reject the type-invalid source")
  }
  run(
    [cli, "build", source, "--out-dir", path.join(directory, "built")],
    directory
  )
  for (const output of [
    run([cli, "run", source], directory),
    run(["bun", "run", "entry.ts"], path.join(directory, "built")),
  ]) {
    if (output.trim() !== "linux ABI smoke")
      throw new Error(`unexpected execution output: ${output}`)
  }
  console.log(
    "Downloaded CLI/LSP/VSIX: checksum, ABI, identical LSP, initialize, type-check diagnostics/build/run passed"
  )
} finally {
  await rm(directory, { recursive: true, force: true })
}
