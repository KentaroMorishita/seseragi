# Seseragi 仕様横断監査

この文書は、章単位では見つけにくい矛盾を追跡する非規範な監査記録です。言語の意味は
[`spec/`](./spec/README.md)、機能の網羅状況は[`SPEC_COVERAGE.md`](./SPEC_COVERAGE.md)を正とします。

## 監査の観点

各passでは、少なくとも次を本文、Appendix grammar、diagnostic、exampleの間で突き合わせます。

- 採用しないと宣言した構文や型が、別章の公開signatureへ混入していないか。
- 同じ型、operation、糖衣のsignatureと展開規則が章ごとに一致するか。
- pure / Effect、typed failure / defect / cancellationの境界が変わっていないか。
- resource、Stream、Signal、DOMのorderingとlifetimeが最適化規則を含めて一致するか。
- sourceとして書ける構文がAppendix grammarにあり、失敗にはstable diagnosticがあるか。
- lessonとfixtureが、本文にない意味を独自に作っていないか。

## 完了したpass

### 2026-07-05: 型構文と性能境界

| 確認項目                           | 結果                                                                                         |
| ---------------------------------- | -------------------------------------------------------------------------------------------- |
| arbitrary union / intersection禁止 | generic Effect APIに未定義の`R & { service: Service }`が残っていた                           |
| requirement合成                    | 一般intersectionではない第一型引数限定のrequirement mergeとして定義した                      |
| grammar / diagnostic               | `&`のparse骨格、`SES-T0501`、positive / negative fixtureを追加した                           |
| syntax preview                     | 単独`&`をlogical `&&`やcustom operatorと別tokenへ分類した                                    |
| fixture checker                    | diagnostic codeとseverityが12.13 registryに存在・一致することを検査するようにした            |
| Signal abstraction                 | Functor / Applicativeのみで、Monadを持たない正本は一致していた                               |
| performance optimization           | 評価順、Effect、cancellation、finalizer、Stream demand、DOM event順をas-if境界として確認した |
| recursion                          | direct self tail callのstack safetyを型章と性能章で一致させた                                |

requirement mergeの解決では、既存の公開APIを一般intersection型へ拡張していません。`R & S`は
Effect / Streamのrequirement field集合をcompile時に正規化する型表現だけです。値のintersection、
runtime merge、row remainderを導入しません。

### 2026-07-05: inherent methodとtrait instance

同じ`impl` keywordがnominal型固有methodと型クラスinstanceの二つを表し、headの形だけで意味を
切り替えていました。両者は名前解決、visibility、coherence、orphan rule、backend loweringが異なるため、
次のsurfaceへ分離しました。

```seseragi
impl User {
  pub fn displayName self: User -> String = self.name
}

instance Show<User> {
  fn show value: User -> String = value.displayName
}
```

`impl`はinherent methodと標準operator糖衣、`instance`はtrait dictionaryだけに使います。旧形の
`impl Trait<Type>`は`SES-T0502`とし、grammar、reserved word、syntax preview、lesson、positive / negative
fixtureを同期しました。

## 次のpass

1. 標準ライブラリsignatureをmodule別にcatalog化し、重複宣言、parameter順、empty value / function、
   failure nestingを照合する。
2. module、package、entry point、TypeScript ABI、`.d.ts`変換のvisibilityと初期化境界を照合する。
3. Effect / Stream / Signal / DOMのcancellation、simultaneous failure、finalizer優先順位をtrace表で照合する。
4. grammarの全productionをlesson / fixture tokenへ対応づけ、formatter・highlightとのtoken差を調べる。
5. standard APIの計算量、allocation、storage retention記述を14章のcost classと照合する。

未完了passがあるため、この文書は仕様全体に矛盾がないことを証明しません。passごとに発見事項を本文へ
反映し、解決をfixtureへ固定してから完了として追記します。
