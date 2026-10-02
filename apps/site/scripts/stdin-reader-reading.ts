type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}
type Reading = { en: string; ja: string }

// Preserve the exact three selected identities and opaque classification.
// Stdin is host-supplied; the two errors have public constructor patterns.
export function stdinReaderReading(item: Identity, fallback: Reading): Reading {
  if (
    item.module !== "std/stdin" ||
    item.namespace !== "type" ||
    item.kind !== "opaque-type"
  )
    return fallback
  if (item.identity === "std/prelude::Stdin") {
    return {
      ja: "Stdinは実行環境が用意する入力サービスの型です。std/stdinをstdinという別名で読み込むprocess向けの例では、with stdin: stdin.Stdinで要求を宣言します。読み取り操作を実行するたびに、同じ入力の続きへ進みます。Stdinを自分で作る公開コンストラクターはありません。",
      en: "Stdin is an input service supplied by the execution host. In process examples that import std/stdin as stdin, declare the requirement with stdin: stdin.Stdin. Each executed read continues from the same input cursor. There is no public Stdin constructor.",
    }
  }
  if (item.identity === "std/prelude::StdinError") {
    return {
      ja: "StdinErrorは標準入力の読み取りの失敗を表します。内部の表現は非公開ですが、公開コンストラクターの形をmatchで区別できます。std/stdinをstdinという別名で読み込んだ場合、stdin.InvalidStdinUtf8 { offset }で入力全体の先頭から数える0始まりのバイト位置、stdin.StdinLineTooLong { limitBytes }で設定した上限を受け取れます。",
      en: "StdinError represents a failed standard-input read. Its internal representation is private, but its public constructor shapes can be matched. With std/stdin imported as stdin, stdin.InvalidStdinUtf8 { offset } gives the zero-based byte position from the start of the entire input, and stdin.StdinLineTooLong { limitBytes } gives the configured limit.",
    }
  }
  if (item.identity === "std/stdin::StdinConfigError") {
    return {
      ja: "StdinConfigErrorは入力の読み方を設定する値の検証に失敗したことを表します。内部の表現は非公開ですが、公開コンストラクターの形をmatchで区別できます。lineLimitの結果では、stdin.NonPositiveLineLimit bytesやstdin.LineLimitTooLarge bytesで、受け付けられなかったバイト数を受け取れます。この設定の検証では標準入力を消費しません。",
      en: "StdinConfigError represents a failed validation of an input-reading setting. Its internal representation is private, but its public constructor shapes can be matched. For a lineLimit result, stdin.NonPositiveLineLimit bytes and stdin.LineLimitTooLarge bytes give the rejected byte count. Validating this setting consumes no standard input.",
    }
  }
  return fallback
}
