type Reading = { en: string; ja: string }
type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}
const opaqueJa =
  "この型の内部の表現は非公開です。値を作ったり取り出したりするには、同じモジュールが提供する関数を使ってください。"
const opaqueEn =
  "This type keeps its representation private. Use the owning module's constructors and operations rather than accessing its internals."

// Registry kinds stay unchanged. Clarify only the selected public JSON access paths.
export function jsonReaderReading(item: Identity, fallback: Reading): Reading {
  if (
    item.identity === "std/json::record" &&
    item.module === "std/json" &&
    item.namespace === "value" &&
    item.kind === "function"
  ) {
    const en =
      "An argument containing -> is a function value; pass a named function or lambda with the matching input and result types."
    const ja =
      "->を含む引数の型は関数です。入力と結果の型が合う関数名かラムダを渡します。"
    if (!fallback.en.includes(en) || !fallback.ja.includes(ja))
      throw new Error(
        "JSON record callback reading baseline changed; review its exact override"
      )
    return {
      en: fallback.en.replace(
        en,
        "The first argument is an array of field-name/checker pairs. Each checker is a function taking Json and returning Either<DecodeError, A>; put a named checker or a matching lambda in each pair."
      ),
      ja: fallback.ja.replace(
        ja,
        "最初の引数は、フィールド名と読み取り関数を組にした配列です。各組の2番目に、Jsonを受け取ってEither<DecodeError, A>を返す関数名かラムダを渡します。"
      ),
    }
  }
  if (item.module !== "std/json" || item.namespace !== "type") return fallback
  if (
    item.identity === "std/json::DecodeError" &&
    item.kind === "opaque-struct"
  ) {
    if (!fallback.en.includes(opaqueEn) || !fallback.ja.includes(opaqueJa)) {
      throw new Error(
        "DecodeError reading baseline changed; review its exact override"
      )
    }
    return {
      en: fallback.en.replace(
        opaqueEn,
        "DecodeError has readable path and kind fields. Go through error.path and match each JsonField or JsonIndex segment. Match error.kind to handle its reason. You cannot construct DecodeError { ... } or use that shape as a struct pattern; use an error returned by a decoder."
      ),
      ja: fallback.ja.replace(
        opaqueJa,
        "DecodeErrorのpathとkindは読み取れます。error.pathの各要素をJsonFieldかJsonIndexに分けて読み、error.kindをmatchで調べて理由を扱います。DecodeError { ... }の形で値を作ったり、その形をmatchに書いたりはできません。読み取り関数から返ったエラーを使います。"
      ),
    }
  }
  if (item.kind !== "opaque-type") return fallback
  const replacement =
    item.identity === "std/json::Json"
      ? "JsonNullやJsonStringなどの公開された名前でJSONの値を作り、matchで値の種類と中身を読み取れます。JSON形式のテキストから値を得るにはparseを使います。"
      : item.identity === "std/json::JsonParseError"
        ? "JsonParseErrorには、構文の問題を表すInvalidJsonSyntaxと、重複したフィールドを表すDuplicateJsonFieldがあります。公開された二つの名前をmatchに書くと、それぞれの詳しい情報を読み取れます。"
        : item.identity === "std/json::JsonReadError"
          ? "JsonReadErrorには、JSONの読み取りに失敗したJsonSyntaxFailureと、値の型や形の確認に失敗したJsonDecodeFailureがあります。公開された二つの名前をmatchに書くと、中のエラーを読み取れます。"
          : undefined
  if (replacement === undefined) return fallback
  if (!fallback.ja.includes(opaqueJa))
    throw new Error(
      `${item.identity} Japanese reading baseline changed; review its exact override`
    )
  return { en: fallback.en, ja: fallback.ja.replace(opaqueJa, replacement) }
}
