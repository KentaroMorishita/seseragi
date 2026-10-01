type Reading = { en: string; ja: string }
type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}

// Preserve exact declarations and generic parameter guidance. Only these two
// identities need more precise branch/public-constructor reading guidance.
export function resultFoundationReading(
  item: Identity,
  reading: Reading
): Reading {
  if (
    item.identity === "std/validation::Validation" &&
    item.module === "std/validation" &&
    item.namespace === "type" &&
    item.kind === "opaque-type"
  ) {
    const before =
      "この型の内部の表現は非公開です。値を作ったり取り出したりするには、同じモジュールが提供する関数を使ってください。"
    if (!reading.ja.includes(before))
      throw new Error(
        "Validation declaration-reading baseline changed; review its exact override"
      )
    return {
      ...reading,
      ja: reading.ja.replace(
        before,
        "この型の内部の表現は非公開です。公開されたValidとInvalidで値を作り、matchで中身を読めます。"
      ),
    }
  }
  if (
    item.identity !== "std/either::swap" ||
    item.module !== "std/either" ||
    item.namespace !== "value" ||
    item.kind !== "function"
  )
    return reading
  const en = "Use match to handle Left for failure and Right for success."
  const ja =
    "失敗はLeft、成功はRightで表します。matchでどちらの場合も扱ってください。"
  if (!reading.en.includes(en) || !reading.ja.includes(ja)) {
    throw new Error(
      "swap declaration-reading baseline changed; review its exact override"
    )
  }
  return {
    en: reading.en.replace(
      en,
      "After swapping, Left contains the original Right value and Right contains the original Left value. The payloads are unchanged."
    ),
    ja: reading.ja.replace(
      ja,
      "入れ替えた後のLeftには元のRightの値、Rightには元のLeftの値が入ります。中の値そのものは変わりません。"
    ),
  }
}
