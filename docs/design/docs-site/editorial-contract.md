# Docs Reboot 編集契約

正本の入口は [#601](https://github.com/KentaroMorishita/seseragi/issues/601)、
この契約の作業は [#763](https://github.com/KentaroMorishita/seseragi/issues/763)。
Docsの文章・コード・章構成・視覚表現を決める規範はこの文書一つに集約する。
`reader-contract.md`はレビュー方法、`site-architecture.md`は導線、
`implementation-architecture.md`は実装境界を担当し、別の編集規範を作らない。
言語の意味は引き続き `docs/spec/`、APIの型・identityはCompiler Reference、
実行の主張は現行compiler/runtimeで検証したsourceを正本とする。

状態: #763のレビュー候補。61問の合意を実装可能な形にした契約であり、
新しい例の美学や実読者の理解を作者が承認済みとは扱わない。
旧Issueの順序・q/Oキュー・旧記事の完成宣言を今後の作業条件に使わない。
依存関係と公開条件は#601に従う。


## 最優先の訂正（2026-10-10）

旧本文・旧URL・anchor・章構成・旧記事数・生成ページ数の保全方針は撤回済み。
本文と導線は61問の判断からスクラッチで構築する。旧記事との対応表やredirect、
全件移植、旧consumerの置換を待ってから削除する作業は要求しない。
コンパイラの仕様と独立したPlaygroundの動作は守り、便利な技術資産だけ再利用する。
検証済みのまとまりはDocs用Vercel本番へ随時公開できる。作者・実読者の未確認状態は
正直に記録するが、通常の実装・PR・マージ・公開を承認待ちで止めない。

## 読者と語り口

プログラミング経験者に向けて書く。経験言語は限定しない。
関数、引数、戻り値、条件分岐には馴染みがあるとしてよい。
Rust、Haskell、圏論、関数型用語の知識は要求しない。
Seseragiの記法は必要になった場所で読み方を示す。deep linkから来た読者も、
例の目的・入力・結果をそのページで理解できるようにする。

作者の個人サイトで、隣のエンジニアとコードについて話している感覚を目指す。
単に技術書の語尾を柔らかくするだけで会話調とみなさない。
「このコードを読んだ瞬間に何に気づけるか」から文章を起こす。
「こんなときは〜してみよう」「〜だよね」は使えるが、語尾を機械的に揃えない。
英語も自然な会話調で書き、関西弁や日本語の語尾を直訳しない。
比喩、小さな脱線、遊び心は積極的に歓迎する。読者がコードの関係を掴む助けに
なるものを選び、必ず比喩を入れる型や、取って付けた例えにはしない。
日英それぞれ完成記事を通読して、作者と自然に話している温度か確かめる。
ポエム、誇張、自画自賛、毎段落の問いかけ、コードの逐語説明は避ける。
設計思想は実例の理由として伝え、一般的なFP礼賛や数学を前提にしない。
数学・法則は知りたい人が寄り道できる出口に置く。

## コードを読む過程を面白くする

小さな実用のコードから始め、少し変えて結果を確かめ、概念の名前に出会う。
前半は体験と発見、後半は型・評価・制約などの詳しい規則を引けるようにする。
これは編集の流れであり、全記事に同じ節や段落を要求するテンプレートではない。
短いAPI usageに不要な導入・失敗・診断節を足さない。
実行入口の説明が必要な例では、その例で必要な分だけ説明する。
全ページへ同じ `pub effect fn main` / `do` の解説を挿入しない。

一つの記事は一つの読者の目的を持つ。章内には自然な流れを作るが、
前の記事を読まないと例の名前さえ分からない構成にしない。
章冒頭は短い見取り図、記事末は目的を示した複数の出口にする。
Tourは別の順序付き学習コース。DocsにTourの複製を作らない。
記事見出しは技術名と「何ができるか」を結び、詳細規則へ直接移れるanchorを持つ。

他言語の比較は難所の補助線として使う。どの言語も自然なコードにし、
同じ入力・出力・失敗条件を比較する。TSをわざと冗長にしたり、
OOPやメソッドchainをSeseragiの標準スタイルにしたりしない。
ホームや各記事に比較を必須化しない。

## サンプルコードの美学

- 意味の小さい関数、簡潔で自然な命名、関数の合成を使う。
  式を一つの巨大関数へ集約したり、説明を関数名へ詰め込んだりしない。
- 型は関数境界で示し、内部の局所値は推論を活かす。
  初見の型注釈や記号は必要な分だけその場で読む。
- 短い式は一行。長い式は使う記法の構造が見える改行にする。
  コメントは判断に必要な箇所へ短く添える。
- 通常適用は素直な呼び出し、`$`は右側をまとめて渡す適用、`|>`は値から処理を
  辿る流れとして選ぶ。意味のないリテラル先頭pipelineを標準例にしない。
- `<$>` / `<*>` / `>>=` / `do` などはSeseragiの重要な表現手段として
  **積極的に活かす**。文脈が合うなら記号を第一候補として検討する。
  名前付きのmap/flatMapや `|>` に回収せず、通常適用・名前付き操作との意味の差が
  コードから分かる自然な合成を選ぶ。記号の美しさと処理の必然性を両立させる。
  `<$>`は包まれた値への変換、`<*>`は包まれた関数と引数の適用、`>>=`は
  前の結果に依存する合成、`do`はその合成を順に読む記法として、
  違いが必要になった場所で扱う。具体的な型・instanceの規則はspecと照合する。
  記号を全種類使うノルマや、全てpipelineにする方針は設けない。
- 純粋な計算と主効果を分けて示す。`effect fn`をprintの飾りとして説明しない。
  実行可能な完全sourceと、表示する抜粋の関係を明示する。
- 動く例は現行実装で実行し、実際のstdout、型、診断を文章と突合する。
  未実装の仕様は「未実装」と示し、実装済みの実験として見せない。

## 同じ計算、違う書き方

**部分適用を説明する範囲での良い候補**。個々の関数や値に一つずつ役割があり、
再利用を対比できる。全Docsの美学のお手本や、#766のHero候補ではない。
この例を代表コードへ自動転用しない。
完全sourceは `editorial-examples/good/src/main.ssrg`。

```seseragi
fn price unit: Int -> count: Int -> Int = unit * count

let notebook = price 120
let total = notebook 3

pub effect fn main = println (show total)
```

編集上の悪い候補。同じ `360` を出すが、説明名に計算を詰め込み、リテラルから
出力までpipelineにするための構造になっている。構文エラーの例ではない。
完全sourceは `editorial-examples/bad/src/main.ssrg`。

```seseragi
fn calculateTotalPriceForThreeNotebooks unitPrice: Int -> Int = unitPrice * 3

pub effect fn main = 120 |> calculateTotalPriceForThreeNotebooks |> show |> println
```

どちらもIntの小さい入力での計算例で、丸め・税・通貨・overflowを扱う
請求処理の設計ではない。「悪い」は新しいDocsの編集基準に対する評価であり、
言語がその書き方を禁止しているという意味ではない。
実行・整形の結果と再現方法は[移行台帳](migration-ledger.md)に残す。

### 良い文章の候補

> `price` は二つ引数を取るのに、ここでは `price 120` で止めている。
> 残りの冊数はあとで渡せるんだ。これに `notebook` と名前を付けておけば、
> 3冊は `notebook 3`、5冊は `notebook 5`。単価を毎回書かずに使い回せる。
>
> この「引数を一部だけ渡す」書き方を部分適用と呼ぶ。
> `price` は単価と冊数を受け取るけど、`notebook` が待つのは冊数だけ。
> 今回の `total` は360になる。最後の行は、この計算結果を文字列にして出力する
> 主効果を `main` から返す。実行機構がその効果を開始する。

英語は同じ例の意味を自然に書き直す。

> `price` takes two arguments, but we stop at `price 120` here. We can supply
> the quantity later. Call that function `notebook`, and we get `notebook 3`
> for three or `notebook 5` for five, without repeating the unit price.
>
> Supplying only some of a function's arguments is called partial application.
> `price` takes a unit price and a quantity; `notebook` only needs the quantity.
> Here, `total` is 360. The last line returns an effect from `main` that prints
> the result as text. The runner starts that effect.

### 悪い文章の例

> Seseragiは驚くほど美しく、強力で洗練された関数型言語です。
> まずfnで関数を定義します。unitはIntです。countもIntです。
> unitにcountを掛けます。letでnotebookを定義します。letでtotalを定義します。
> この革新的な機能により、簡潔で読みやすいコードを書けます。

> Seseragi's revolutionary elegance empowers you to write powerful code.
> First we define a function. Then we define a variable. This is concise and readable.

逐語説明は読者が発見できる関係を隠し、褒め言葉は部分適用の役割を説明しない。
良い候補も作者の文体レビューと実読者レビューを別々に受ける。
コンパイル成功を文章の面白さの承認に置き換えない。

## APIと視覚表現

APIのidentity、namespace、kind、signature、constraints、availabilityは
compiler metadataから生成する。多くのAPIは正確な宣言と簡潔なusageでよい。
Seseragiらしい抽象化や、signatureだけでは難しいAPIに編集の厚みを集中させる。
module overviewは用途と使い分けを説明し、同じ説明を全symbolへ複写しない。

タイポグラフィと余白を軸に、コードを主役にする。ホームはHero、
一つの良質な `fn` / `main` のコードまたはデモ、短い紹介、明確なリンクで構成する。
機能カードの過積載や全面IDEを玄関にしない。
目的からでも技術名からでも同じ記事identityへ辿れるようにする。
Examplesは検証済みsourceへ、Releasesは実在するreleaseへ繋ぐ。

図はコードだけでは分かりにくい関係を補う場合に使う。動く図も同じ基準で選ぶ。
記事内編集・実行・型表示は必要時に開く追加体験として、既存WASM/Analysisを
再利用する。静的本文・出力・リンクはno-JSでも読める。
#768を本文の完成やホーム公開の必須条件にしない。

## 最初の章と受け入れ

#765「関数と記法」を最初の完成品質モデルにする。
関数を書く → 関数を値として扱う → 関数を組み合わせる、という流れを候補にし、
最後に `<$>` / `<*>` / `>>=` / `do` へ進める出口を設ける。
記事数は固定しない。型規則と個別記法には、新構成に合った直接参照先を設ける。旧URLやanchorは制約にしない。
#765では上の限定例とは別に、短く意味のある `fn` と `main`、小さな関数同士の
自然な合成、記法が必要になる理由が見える検証済みの代表例を選ぶ。
動くだけの無難な教材を量産しない。「自分でもこう書きたい」と感じるかを、
コードの正しさと分けてレビューする。#766のHeroもその観点で独立に選定する。
#764はこの章の最初の一ページを、新しいtyped compositionで日英表示する。

| 確認軸 | 合格の証拠 |
| --- | --- |
| 文章・章構成 | 通読して重複や途切れがない。agent自己レビュー、作者の美学判断、実読者の感想を区別して記録する |
| コードの美学 | 小さい関数、自然な命名、適用・部分適用・合成の選択理由を実例で説明できる |
| 実行・意味 | 現行CLIのversion/commit、source、stdoutまたは診断、spec/API provenance、整形結果を記録する |
| 日英 | 同じsource・入力・出力・制約・出口。各言語の文章は自然で、構造上必要な情報に片側の欠落がない |
| 読者の理解 | 読者が目的・重要な記法・結果・次に使う方法を自分の言葉で説明できる。知識背景と必要だった助けを記録する |
| 表示・導線 | desktop/mobile、no-JS、a11y、locale往復、URL/fragment、章の入口と出口を確認する |
| 基盤・退行 | scoped checkと必要なproduction/browser/full-site gate。再利用したSSG、compiler由来の宣言、実行検証、独立したPlaygroundを確認する |

#765の最初の代表章では作者の美学判断と実読者の理解を重点的に検証する。
残りのroute/APIは、技術検証・agent編集レビュー・作者判断・実読者検証・公開退役を
独立した状態として記録する。全API・全routeの実読者承認を一律の退役条件に
しない。実読者未確認の移行はその状態を明示し、agent自己レビューで埋めない。
本文数・見出し数・文字数・翻訳ファイルの存在・旧Issueのcloseだけでは合格にしない。
失敗したgateと未検証の項目はそのまま残件に記録する。
自然な編集順へ変える際のassertion修正は#740と調整し、同等の意味・導線の
coverageを保つ。現在の公開構成と検証結果は[制作記録](migration-ledger.md)に残す。
