import contract from "./linux-native-contract.json"

export function assertGlibcVersions(output: string): string[] {
  const versions = [...output.matchAll(/\bName:\s+(GLIBC_[^\s]+)/gu)].map(
    (match) => match[1]
  )
  if (versions.length === 0)
    throw new Error("ELF has no GLIBC version requirements")
  const maximum = contract.glibcMaximum.split(".").map(Number)
  for (const version of versions) {
    const match = /^GLIBC_(\d+)\.(\d+)(?:\.(\d+))?$/u.exec(version)
    if (!match) throw new Error(`unsupported GLIBC requirement: ${version}`)
    const parts = match.slice(1).map((part) => Number(part ?? 0))
    const comparison = parts.findIndex(
      (part, index) => part !== (maximum[index] ?? 0)
    )
    if (comparison >= 0 && parts[comparison] > (maximum[comparison] ?? 0)) {
      throw new Error(
        `${version} exceeds linux-x64 glibc ${contract.glibcMaximum}`
      )
    }
  }
  return [...new Set(versions)].sort()
}

function readelf(binary: string, option: string): string {
  const result = Bun.spawnSync(["readelf", "--wide", option, binary], {
    stdout: "pipe",
    stderr: "pipe",
  })
  if (!result.success) throw new Error(`readelf ${binary}: ${result.stderr}`)
  return result.stdout.toString()
}

export function assertLinuxAbi(binary: string): void {
  const header = readelf(binary, "--file-header")
  if (
    !/Class:\s+ELF64/u.test(header) ||
    !/Machine:\s+Advanced Micro Devices X86-64/u.test(header)
  ) {
    throw new Error(`${binary}: expected x86-64 ELF`)
  }
  if (
    !readelf(binary, "--program-headers").includes(
      "/lib64/ld-linux-x86-64.so.2"
    )
  ) {
    throw new Error(`${binary}: linux-x64 requires the GNU/glibc loader`)
  }
  const allowed = new Set([
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "libpthread.so.0",
    "librt.so.1",
    "libgcc_s.so.1",
    "ld-linux-x86-64.so.2",
  ])
  const dynamic = readelf(binary, "--dynamic")
  for (const match of dynamic.matchAll(/\(NEEDED\).*\[([^\]]+)\]/gu)) {
    if (!allowed.has(match[1]))
      throw new Error(`${binary}: unsupported runtime library ${match[1]}`)
  }
  if (/\((?:RPATH|RUNPATH)\)/u.test(dynamic))
    throw new Error(`${binary}: embedded runtime search path is not portable`)
  const versions = assertGlibcVersions(readelf(binary, "--version-info"))
  console.log(
    `${binary}: GNU x86-64, GLIBC <= ${contract.glibcMaximum}; ${versions.join(", ")}`
  )
}

if (import.meta.main) {
  if (process.argv.length < 3)
    throw new Error("usage: linux-native-abi.ts ELF...")
  for (const binary of process.argv.slice(2)) assertLinuxAbi(binary)
}

export function assertNativeBuildIdentity(
  cli: Record<string, unknown>,
  lsp: Record<string, unknown>
): void {
  for (const field of ["version", "commit", "channel", "dirty", "target"]) {
    if (cli[field] === undefined || cli[field] !== lsp[field]) {
      throw new Error(`CLI/LSP metadata mismatch: ${field}`)
    }
  }
  // CLI JSON uses null for a development build; LSP omits the optional tag.
  if ((cli.releaseTag ?? null) !== (lsp.releaseTag ?? null)) {
    throw new Error("CLI/LSP metadata mismatch: releaseTag")
  }
}
