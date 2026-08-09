const MARKETPLACE_QUERY =
  "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery?api-version=7.2-preview.1"

export const OFFICIAL_EXTENSION_ID = "seseragi-dev.seseragi"
export const LEGACY_EXTENSION_ID = "seseragi-dev.seseragi-spec-preview"
export const OFFICIAL_TARGETS = [
  "darwin-arm64",
  "darwin-x64",
  "linux-x64",
  "win32-x64",
] as const

type MarketplaceVersion = {
  version?: unknown
  targetPlatform?: unknown
}

type MarketplaceExtension = {
  extensionName?: unknown
  publisher?: { publisherName?: unknown }
  versions?: MarketplaceVersion[]
}

type MarketplaceResponse = {
  results?: Array<{ extensions?: MarketplaceExtension[] }>
}

export type MarketplaceReleaseStatus = {
  version: string
  officialTargets: string[]
  legacyTargets: string[]
}

function fail(message: string): never {
  throw new Error(`Marketplace release: ${message}`)
}

function extensionId(extension: MarketplaceExtension): string | null {
  const publisher = extension.publisher?.publisherName
  const name = extension.extensionName
  return typeof publisher === "string" && typeof name === "string"
    ? `${publisher}.${name}`
    : null
}

function targets(
  response: MarketplaceResponse,
  id: string,
  version: string
): string[] {
  const extension = response.results
    ?.flatMap((result) => result.extensions ?? [])
    .find((candidate) => extensionId(candidate) === id)
  if (!extension) return []
  return [
    ...new Set(
      (extension.versions ?? [])
        .filter((candidate) => candidate.version === version)
        .map((candidate) =>
          typeof candidate.targetPlatform === "string"
            ? candidate.targetPlatform
            : "universal"
        )
    ),
  ].sort()
}

export function marketplaceReleaseStatus(
  official: MarketplaceResponse,
  legacy: MarketplaceResponse,
  version: string
): MarketplaceReleaseStatus {
  return {
    version,
    officialTargets: targets(official, OFFICIAL_EXTENSION_ID, version),
    legacyTargets: targets(legacy, LEGACY_EXTENSION_ID, version),
  }
}

export function assertMarketplaceRelease(
  status: MarketplaceReleaseStatus
): void {
  const missingOfficial = OFFICIAL_TARGETS.filter(
    (target) => !status.officialTargets.includes(target)
  )
  if (missingOfficial.length > 0) {
    fail(
      `${OFFICIAL_EXTENSION_ID}@${status.version} is missing target(s): ${missingOfficial.join(", ")}`
    )
  }
  if (status.legacyTargets.length === 0) {
    fail(`${LEGACY_EXTENSION_ID}@${status.version} is not published`)
  }
}

async function query(id: string): Promise<MarketplaceResponse> {
  const response = await fetch(MARKETPLACE_QUERY, {
    method: "POST",
    headers: {
      Accept: "application/json;api-version=7.2-preview.1",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      filters: [{ criteria: [{ filterType: 7, value: id }] }],
      flags: 914,
    }),
  })
  if (!response.ok) {
    fail(`query for ${id} returned HTTP ${response.status}`)
  }
  return (await response.json()) as MarketplaceResponse
}

function integerOption(name: string, fallback: number): number {
  const index = process.argv.indexOf(name)
  if (index === -1) return fallback
  const value = Number(process.argv[index + 1])
  if (!Number.isInteger(value) || value < 1) {
    fail(`${name} must be a positive integer`)
  }
  return value
}

async function verify(version: string): Promise<MarketplaceReleaseStatus> {
  return marketplaceReleaseStatus(
    await query(OFFICIAL_EXTENSION_ID),
    await query(LEGACY_EXTENSION_ID),
    version
  )
}

async function main(): Promise<void> {
  if (process.argv[2] !== "verify" || !process.argv[3]) {
    fail(
      "usage: marketplace-release.ts verify VERSION [--attempts N] [--delay-ms N]"
    )
  }
  const version = process.argv[3]
  const attempts = integerOption("--attempts", 1)
  const delayMs = integerOption("--delay-ms", 10_000)
  let latestError: unknown
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const status = await verify(version)
      assertMarketplaceRelease(status)
      console.log(JSON.stringify(status, null, 2))
      return
    } catch (error) {
      latestError = error
      if (attempt === attempts) break
      console.error(
        `Marketplace verification attempt ${attempt}/${attempts} is not ready; retrying in ${delayMs}ms.`
      )
      await Bun.sleep(delayMs)
    }
  }
  throw latestError
}

if (import.meta.main) await main()
