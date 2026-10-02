type Reading = { en: string; ja: string }
type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}

export function bytesInspectionReading(
  item: Identity,
  fallback: Reading
): Reading {
  if (
    item.module !== "std/bytes" ||
    item.namespace !== "type" ||
    item.kind !== "opaque-type" ||
    item.identity !== "std/bytes::BytesSliceError"
  )
    return fallback
  return {
    en: "BytesSliceError is the error type for a rejected Bytes slice. Keeping its representation private does not prohibit constructing or matching values through its public constructor. InvalidByteRange carries start, end and length. Match InvalidByteRange { start, end, length } to read the requested bounds and the input's byte length. The range includes start and excludes end. Constructing a reason value does not validate a range.",
    ja: "BytesSliceErrorは、Bytesの切り出しに失敗した理由を表す型です。内部表現が非公開という分類は、公開されたコンストラクターで値を作ったり照合したりする操作まで禁止するものではありません。公開されたInvalidByteRangeコンストラクターは、start、end、lengthのフィールドを持ちます。InvalidByteRange { start, end, length }をmatchに書くと、指定した開始位置と終了位置、入力のバイト数を読み取れます。範囲は開始位置を含み、終了位置を含みません。コンストラクターで理由の値を作るだけでは、範囲の検証は行われません。",
  }
}
