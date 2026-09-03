import { UNICODE_VERSION } from "./unicode-version-data"

export { UNICODE_VERSION } from "./unicode-version-data"

/** Called by every generated source module before its first source initializer. */
export function assertUnicodeVersion(required: string): void {
  if (required !== UNICODE_VERSION) {
    throw new Error(
      `Seseragi runtime ABI mismatch: artifact requires Unicode ${required}, runtime provides ${UNICODE_VERSION}`
    )
  }
}
