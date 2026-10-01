export interface StorageReader {
  getItem(key: string): string | null
}
export function readName(storage: StorageReader): string {
  try {
    const value = storage.getItem("name")
    return value === null ? "missing" : `found: [${value}]`
  } catch (error) {
    return error instanceof Error ? error.message : String(error)
  }
}
