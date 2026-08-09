import { describe, expect, test } from "bun:test"
import {
  assertMarketplaceRelease,
  LEGACY_EXTENSION_ID,
  marketplaceReleaseStatus,
  OFFICIAL_EXTENSION_ID,
  OFFICIAL_TARGETS,
} from "./marketplace-release"

function response(id: string, version: string, targets: Array<string | null>) {
  const [publisher, name] = id.split(".")
  return {
    results: [
      {
        extensions: [
          {
            extensionName: name,
            publisher: { publisherName: publisher },
            versions: targets.map((target) => ({
              version,
              ...(target ? { targetPlatform: target } : {}),
            })),
          },
        ],
      },
    ],
  }
}

describe("Marketplace release verification", () => {
  test("accepts every official platform and the legacy migration stub", () => {
    const status = marketplaceReleaseStatus(
      response(OFFICIAL_EXTENSION_ID, "0.4.0", [...OFFICIAL_TARGETS]),
      response(LEGACY_EXTENSION_ID, "0.4.0", [null]),
      "0.4.0"
    )

    expect(status.officialTargets).toEqual([...OFFICIAL_TARGETS].sort())
    expect(status.legacyTargets).toEqual(["universal"])
    expect(() => assertMarketplaceRelease(status)).not.toThrow()
  })

  test("rejects an incomplete official platform set", () => {
    const status = marketplaceReleaseStatus(
      response(OFFICIAL_EXTENSION_ID, "0.4.0", ["linux-x64"]),
      response(LEGACY_EXTENSION_ID, "0.4.0", [null]),
      "0.4.0"
    )

    expect(() => assertMarketplaceRelease(status)).toThrow(
      "is missing target(s)"
    )
  })

  test("rejects a missing legacy migration release", () => {
    const status = marketplaceReleaseStatus(
      response(OFFICIAL_EXTENSION_ID, "0.4.0", [...OFFICIAL_TARGETS]),
      { results: [{ extensions: [] }] },
      "0.4.0"
    )

    expect(() => assertMarketplaceRelease(status)).toThrow("is not published")
  })
})
