# 関数と記法 — 章のレビュー記録

この章は #765 の候補。正本は #601 と [editorial-contract.md](editorial-contract.md)。
[reader-contract.md](reader-contract.md)に従い、実装agentの通読、作者の美学、初見読者の
理解、技術検証を別々に記録する。未取得の確認を成功扱いにしない。

## 確認する実物

英語は下記route、日本語は同じrouteに `/ja` を付ける。新しいSSG出力かPRのpreviewを使い、
旧公開版を新本文のレビューと取り違えない。

1. `/docs/language/syntax/function-application/`: 入力の空白・大文字を整える `key`。
2. `/docs/language/types/function-types-and-currying/`: 検索文字を固定する `matches`、
   同じ条件を候補とショートカットのfilterへ渡す。独立した文字数計算でラムダと `(+)`。
3. `/docs/language/syntax/pipelines-and-low-precedence-application/`: 検索と表示を合成し、
   不在を `<$>` で保つ。`$` の改行・括弧なしifと左右の評価順を確かめる。

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
| 新source / native / WASM | 最新scoped pass、全checker再確認待ち | 新4例、検索変更/Maybe拒否、if FalseのWAITING、format。旧全checker passとはrevisionを区別 |
| 英日render / canonical source / URL / fragment | 最新scoped pass、CI全決定性未確認 | 27 tests / 16,221 assertions。旧3,976 routes生成とdeadline failは台帳の履歴 |
| 章を順に読むmobile/desktop/JS/no-JS/keyboard | 旧headで36 page cases / 12章通読 pass、最新headのfull browser待ち | 旧48 screenshotsは改善後の証拠ではない。最新scoped layout確認は下記に追記 |
| 実装agentの通読 | 構成・英日mobile/desktopの表示を確認 | 題材を同じ検索へ揃え、各記事の役割を分けた。数学用の小例は詳細規則へ置いた。独立/初見レビューではない |
| 独立agent simulated reader | 未取得 | この実装agentの確認を独立/初見レビューとは扱わない |
| 作者の美学判断 | 未取得 | draft候補。新章/Heroの承認を主張しない |
| 実読者の初見理解 | 未取得 | 読者背景・感想・修正後の再確認も未取得 |
| 公開/退役 | 未実施 | 旧公開siteは維持。branch内の旧guide削除は検証付き置換の候補 |

`#765`の人間による重点検証が完了するまで、章の受け入れ済みを宣言しない。
`#766`のHeroや`#767`のbulk移行へこの候補を承認済み基準として使わない。

## 2026-10-10 独立レビューへの修正

[#771 comment6091348119](https://github.com/KentaroMorishita/seseragi/pull/771#issuecomment-6091348119)
の4点に対応した。記事2で初めて出す`do`の直後に、出力Effectを順につなぎ、
mainが返したEffectをrunnerが実行することを短く説明した。部分適用のcaptionは
「Set up a search condition; reuse it」とし、候補とショートカットへ同じpredicateを渡す。
検索コードから文字数計算を外し、Unicode scalarの`lengths`を数える独立例へ分離した。
記事3では`$`直後の改行と括弧なしif/match/do/lambdaを復元。短いif例とTrue/Falseの結果、
`$`は左関数→右引数→適用、`|>`は左値→右関数→適用を英日で示した。

Bun1.3.11とrelease-built CLIで4 suite: **27 pass / 0 fail / 16,221 assertions**
（173.85s）。新4 sourceのnative/format/WASM/Playground、既存11構文記事と6関数系記事の
英日render、完全source/出力/診断/identity/link、追加規則とdo説明を確認。
初回renderは新しい補助sourceをtestのBuildInputへ渡しておらず失敗し、同じ正本を
fixtureへ含めて修正した。既存の意味・coverage assertionは保持した。
TypeScript、変更TSのBiome、prose tests、content map351 routes、diff whitespaceもpass。
最新revisionの全site二回hash一致・browserは#706修正の#772を統合した通常CIで再検証する。

描画済み英日3記事をproductionと同じCSSでlocal browser確認:
320/390/1280px・no-JSの18ケースで、h1・追加do/評価規則・完全出力と横overflowを確認。
日本語390pxの第二記事、英語1280pxの第三記事の実物を通読した。
scoped import closureには全catalog/sidebarとproduction JSを含めないため、
全site/browser/navigation gateとは区別する。表示確認後、canonical formatterでも
`$`直後の改行が残る短いコメントをif例に追加した。影響するsyntax/chapter suiteを再実行:
**19 pass / 0 fail / 1,129 assertions**（92.11s）。native/format/WASM/Playgroundと英日render、
改行が残ること、READY/WAITINGを再確認した。
