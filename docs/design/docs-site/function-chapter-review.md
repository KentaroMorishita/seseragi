# 関数と記法 — 章のレビュー記録

この章は #765 の候補。正本は #601 と [editorial-contract.md](editorial-contract.md)。
[reader-contract.md](reader-contract.md)に従い、実装agentの通読、作者の美学、初見読者の
理解、技術検証を別々に記録する。未取得の確認を成功扱いにしない。

## 確認する実物

英語は下記route、日本語は同じrouteに `/ja` を付ける。新しいSSG出力かPRのpreviewを使い、
旧公開版を新本文のレビューと取り違えない。

1. `/docs/language/syntax/function-application/`: 入力の空白・大文字を整える `key`。
2. `/docs/language/types/function-types-and-currying/`: 検索文字を固定する `matches`、
   関数をfilter/map/reduceへ渡す。ラムダと演算子section `(+)`。
3. `/docs/language/syntax/pipelines-and-low-precedence-application/`: 検索と表示を合成し、
   不在を `<$>` で保つ。出力の順序を `do` でつなぐ。

metadata title/既存routeは保全し、本文のh2は技術名と用途を示す。
章の次/前/入口は本文の目的付きリンクで辿れる。元のtype/syntaxカテゴリの
reference-sequenceは保持しており、章の経路とは区別する。

## 作者と実読者の確認項目

これは確認用の課題で、回答・承認をすでに得たという記録ではない。

- 言語経験を記録し、日英どちらを読んだか、viewport、preview/revisionを残す。
- 3記事を順に通読して、気になる重複、意味の飛躍、教材っぽい語り、取って付けた
  比喩、自然に書いてみたくなった箇所を具体的な段落で示す。
- `matches "b"`がどんな型の値か説明する。検索を `"c"` に替え、元のラベルと表示用
  ラベルの違い、`Just`と`Nothing`になる条件を予測してから実行する。
- 通常適用・`$`・`|>`のどれを選ぶか理由を述べる。`badge <$> picked`を普通の
  `badge picked`に替えたら何が合わないか、Effectがどこで実行されるかを説明する。
- mobile/no-JSで詳細規則へ飛び、次の記事と章の入口へ戻る。日英切替後も同じページを
  読めるか確認する。理解のために必要だった戻り先を記録する。
- 指摘を直した後、同じ読者による再確認とrevisionを残す。

## 状態

| Scope | 状態 | 記録 |
| --- | --- | --- |
| 新source / native / WASM | 検証中 | 結果はmigration ledgerとPRに記録 |
| 英日render / canonical source / URL / fragment | 検証中 | 既存scoped testsとproduction build |
| 章を順に読むmobile/desktop/JS/no-JS/keyboard | 検証中 | 章専用browser verifier・screenshots |
| 実装agentの通読 | 候補の構成を確認、表示は検証中 | 題材を同じ検索へ揃え、各記事の役割を分けた。数学用の小例は詳細規則へ置いた |
| 独立agent simulated reader | 未取得 | この実装agentの確認を独立/初見レビューとは扱わない |
| 作者の美学判断 | 未取得 | draft候補。新章/Heroの承認を主張しない |
| 実読者の初見理解 | 未取得 | 読者背景・感想・修正後の再確認も未取得 |
| 公開/退役 | 未実施 | 旧公開siteは維持。branch内の旧guide削除は検証付き置換の候補 |

`#765`の人間による重点検証が完了するまで、章の受け入れ済みを宣言しない。
`#766`のHeroや`#767`のbulk移行へこの候補を承認済み基準として使わない。
