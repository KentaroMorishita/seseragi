type Reading = { en: string; ja: string }
type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}

// LoopControl has public Continue/Break constructors despite its opaque registry kind.
// Correct only the Japanese function-only guidance; keep canonical declarations intact.
export function effectSequencingReading(
  item: Identity,
  fallback: Reading
): Reading {
  if (
    item.identity !== "std/effect::LoopControl" ||
    item.module !== "std/effect" ||
    item.namespace !== "type" ||
    item.kind !== "opaque-type"
  )
    return fallback
  const before =
    "この型の内部の表現は非公開です。値を作ったり取り出したりするには、同じモジュールが提供する関数を使ってください。"
  if (!fallback.ja.includes(before))
    throw new Error(
      "LoopControl Japanese reading baseline changed; review the exact override"
    )
  return {
    en: fallback.en,
    ja: fallback.ja.replace(
      before,
      "LoopControlは、次の要素へ進むContinueと、正常に終了するBreakを持ちます。公開された二つの値をそのまま使い、matchで区別できます。Effectの成功値として返すときはsucceedで包みます。"
    ),
  }
}
