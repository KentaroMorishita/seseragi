import { sourceFromPlaygroundUrl } from "./source-link"
import { workspacePersistenceKey } from "./persistence"

const sharedPrefix = "seseragi.playground.shared-workspace:"
const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/

/** Shared links get their own save slot, never the previous local workspace. */
export function playgroundSession(href: string) {
  const url = new URL(href)
  const linkedSource = sourceFromPlaygroundUrl(href)
  const existing = url.searchParams.get("workspace")
  const id =
    linkedSource !== undefined
      ? crypto.randomUUID()
      : existing !== null && uuid.test(existing)
        ? existing
        : undefined
  if (linkedSource !== undefined) {
    url.searchParams.delete("source")
    url.searchParams.set("workspace", id!)
  }
  return {
    linkedSource,
    persistenceKey:
      id === undefined ? workspacePersistenceKey : `${sharedPrefix}${id}`,
    importedUrl: linkedSource === undefined ? undefined : url.href,
    shared: id !== undefined,
  }
}
