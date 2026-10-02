type ReaderSymbol = {
  identity: string
  module: string
  namespace: string
  kind: string
}
type Reading = { en: string; ja: string }

// Exact site-owned explanation only. Canonical declarations and other readings
// remain unchanged, including similarly named symbols and other opaque types.
export function filesystemReaderReading(
  item: ReaderSymbol,
  fallback: Reading
): Reading {
  if (
    item.identity === "std/fs::FileSystem" &&
    item.module === "std/fs" &&
    item.namespace === "type" &&
    item.kind === "opaque-type"
  ) {
    return {
      ja: "FileSystemは、ファイル操作の実行に必要なサービスの型です。with FileSystemで要求を宣言し、実行環境からサービスを受け取ります。std/fsのimportだけではサービスは作られません。",
      en: "FileSystem names the service required to execute file operations. Write with FileSystem to declare this requirement; the execution environment supplies the service. Importing std/fs does not construct it.",
    }
  }
  if (
    item.identity === "std/fs::FileSystemError" &&
    item.module === "std/fs" &&
    item.namespace === "type" &&
    item.kind === "struct"
  ) {
    return {
      ja: "この構造体は、失敗したファイル操作のoperation、path、otherPath、kindを持ちます。この宣言に型引数はありません。",
      en: "This struct stores operation, path, otherPath, and kind for one failed file operation. This declaration has no type parameters.",
    }
  }
  return fallback
}
