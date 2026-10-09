# Docs Reboot 廃止・移行台帳

作業順と公開条件の正本は [#601](https://github.com/KentaroMorishita/seseragi/issues/601)。
編集基準は [editorial-contract.md](editorial-contract.md) に集約する。
この台帳は #763 の具体的な退役判断・依存・検証記録を管理する。
旧leaf Issue、`*-work-item.md`、q/Oキューは履歴・移行素材であり作業キューではない。

## 判断と状態

| 判断 | 意味 | 退役の条件 |
| --- | --- | --- |
| KEEP | identity・意味・基盤・検証済みsourceを保持。必要な改善は可能 | 別方式への置換を理由に消さない |
| REWRITE | 旧本文・構成・導線を素材として、新契約で作り直す | 同じroute/identityの新本文を日英・実行・表示で検証後に旧本文を除去 |
| RETIRE | 固定説明生成、重複copy、不要なadapterをなくす | 全consumerを置換・検証し、import/build/example参照がゼロになってから削除 |
| MERGE | 重複する説明やhelperを一つの意味の所有先にまとめる | 全semantic obligationとdeep linkを移行し、旧URL/anchorの行先を検証する |

判断は進捗ではない。次の軸を独立して各batchに記録する。

| 状態軸 | 記録する値の例 |
| --- | --- |
| 置換 | 未着手 / 候補あり / 置換実装あり |
| 技術検証 | 未実施 / pass / fail / blocked（実行・日英・URL/anchor・build/browserの内訳付き） |
| agent編集レビュー | 未実施 / 所見あり / 修正待ち |
| 作者の美学判断 | 未確認 / 判断待ち / 承認 / 修正要求（実際の判断と対象範囲） |
| 実読者の理解 | 未確認 / session記録あり / 再確認待ち（読者背景と結果） |
| 公開・退役 | 旧本文公開中 / 新本文候補 / 置換公開 / 旧source退役 |

#765の最初の代表章は人間の読解・作者の美学を重点検証し、#767 bulk移行の基準にする。
残りの個々のAPI/routeに実読者承認を一律に義務化しない。技術的置換・編集レビューを
満たして退役するbatchでも「実読者未確認」を記録できる。公開条件は#601と対象Issueに
従い、読者承認が明示的な条件の対象では省略しない。作者判断・実読者結果をagentが
代行したとは宣言しない。REWRITE/RETIRE対象も置換前は公開維持する。
計画だけのrouteを公開済みと扱わない。

## 棚卸しの範囲

基準commit: `46755c08f4e925859a045ed352a72168aa4a32fe`（#763着手時のmain）。
open PR #761/#762 はPlayground/Web UI検証のscopeで、#763との重複はなかった。

[migration-inventory.tsv](migration-inventory.tsv) はsource調査時点の監査用snapshot。
サイトのroute/identityの生成元ではなく、buildは読み込まない。
列は英日route、既存identity、source、実装状態、判断、担当Issue、旧composition。
`source`がpage moduleなら隣接する `en.ssrg` / `ja.ssrg` / `guide.ssrg` /
`details.ssrg` もそのbatchの所有範囲に含む。

- 現行catalogのauthored pageは113件（Languageの106件と入口等7件）。
- `reference-content-map.md`の351件は掲載品質の完成台帳ではない。
  そのうち106件にauthored本文、49件にgenerated moduleが対応し、196件はplanned-only。
- Compiler Referenceは63 module。台帳の旧351件にない14 moduleもsnapshotに含める。
- 個々のsymbol/instanceは `apps/site/scripts/reference.ts::compilerReferenceModules`
  が読む `examples/spec/artifacts/stdlib-schema-1/{reference,prelude}/module.json`
  と `apps/site/src/reference/catalog.ssrg` が正本。全1,812 symbol/instanceを保持し、snapshotへsignatureを複写しない。
  全symbolの `(identity, namespace, itemKind)` とrouteはbuild manifestで比較する。
  新本文のないgenerated pageもKEEP対象で、長文説明を量産して埋めない。

planned-onlyの196件は未掲載のsemantic coverageを示す。#767で必要性と所有先を
判断し直す。単に台帳から削って完成扱いにはせず、仕様の義務と未掲載の状態を残す。
旧#629/#630のcloseは、これらのrouteが全て公開・承認されたことを意味しない。

## ファイルごとの処置

以下のpathはrepository rootからの相対path。台帳の旧composition列とimport検索で
各helperのconsumerを確定する。consumer数は型参照も含むため本文数として数えない。

| ファイル・範囲 | 判断 / 担当 | 置換先と削除条件 |
| --- | --- | --- |
| `apps/site/src/model/{page,locale,build}.ssrg` | KEEP/EVOLVE / #764 | `PageDefinition`/`Block`/typed locale/外部BuildInputを保持。必要な編集APIは既存Blockを自由に組む |
| `apps/site/src/{main.ssrg,render/,components/article.ssrg,layouts/article.ssrg}` | KEEP/EVOLVE / #764 | pure Html/process SSGの単一renderer。新しいMarkdown/TS rendererを追加しない |
| `apps/site/src/pages/language/explanation.ssrg` | RETIRE / #764→#765→#767 | `understand-this`等のprepend方式をpage-owned ordered blocksへ置換。最後のguide・型参照・test fixture importを除去して削除 |
| `apps/site/src/pages/language/reference-article.ssrg` | RETIRE / #764→#767 | rule/typing/evaluation/diagnosticsの一律順を廃止。全consumerがpage-owned blocksへ移った後に削除 |
| `apps/site/src/pages/language/reader-article.ssrg` | RETIRE / #764→#765→#767 | `ReaderCopy`と同じmain解説を廃止。型だけ使うlocaleも移行してから削除 |
| `apps/site/src/pages/language/{syntax,types}/article.ssrg` | RETIRE / #764→#765→#767 | `SyntaxCopy`/`TypeCopy`の固定fieldをページの自然な構成に置換。対応locale/importを含めゼロになってから削除 |
| `apps/site/src/pages/language/model/{concept,principle,reader-principle}.ssrg` | RETIRE / #764→#766→#767 | concept/principleの固定構成とcopy契約をpage-owned blocksへ。表示部品だけは共通rendererへ統合 |
| `apps/site/src/pages/language/syntax/reader-details.ssrg` | MERGE→RETIRE / #764→#767 | 特定の`mistakes`anchor前へ挿入する処理を、page.ssrgの明示した順序へ統合。参照ゼロ後に削除 |
| `apps/site/src/pages/language/**/{guide,en,ja,details,page}.ssrg` | REWRITE / #765/#766/#767 | TSVで担当とrouteを選ぶ。guideの説明を新本文へ統合し、同じ説明を二重掲載しない。local detailsの意味・診断は保全 |
| `apps/site/src/reference/editorial/model.ssrg` | RETIRE fixed generation / #764→#767 | `ApiCopy`/`apiEditorial`/`setupCopy`/汎用main・do説明を廃止。正確なEditorial identity selectionはKEEPし、自由なBlock構成へ移す |
| `apps/site/src/reference/editorial/**/{model,page,en,ja}.ssrg` | REWRITE / #767 | 既存API familyのsource/出力/境界を保全。簡潔なusageまたは必要な深掘りへ。familyごとの旧modelを最後のconsumer後に削除 |
| `apps/site/src/reference/{catalog.ssrg,editorial/catalog.ssrg,editorial/*catalog.ssrg}` | KEEP identity / REWRITE composition / #764/#767 | symbol tuple・module帰属・generated route・exact dispatchを保持。手書きAPI説明は量ではなく必要性で選ぶ |
| `apps/site/src/reference/{module-copy,instance-copy}.ssrg` | REWRITE / #767 | 正確な情報を保持し、module用途とinstanceの必要な説明へ。signatureを本文に手コピーしない |
| `apps/site/src/pages/home/{page,en,ja}.ssrg`, `apps/site/src/layouts/home.ssrg` | REWRITE / #766 | Hero+一つのcode/demo+短い紹介+導線。現在の例・URLは新例の検証前に削除しない |
| `apps/site/src/pages/{docs,examples,releases}/`, `apps/site/src/pages/language/overview/` | REWRITE / #766 | First Run/Examples/Releasesの実装済み成果を保持し、目的・技術名の入口を再構成 |
| `apps/site/src/navigation/catalog.ssrg`, `apps/site/src/components/{sidebar,page-navigation,on-this-page}.ssrg` | KEEP/EVOLVE / #765/#766 | typed identityを共有してchapter/目的別導線を整理。navだけの別content inventoryを作らない |
| `apps/site/{client/,styles/}`, `apps/site/src/{i18n/,components/language-menu.ssrg,components/mobile-sidebar.ssrg}` | KEEP/EVOLVE / #764/#766 | no-JS、locale、keyboard、mobile、reduced-motionを維持して見た目を整える |
| `examples/spec/`, `apps/site/examples/`, `apps/site/scripts/{canonical-example,check-examples}.ts` | KEEP / 各batch | source/出力/invalid診断/Playground source一致を継続検証。廃止表示から参照されなくてもcanonical sourceを一括削除しない |
| `apps/site/scripts/{build,render-generator,reference,coverage,check-content-map,static-site-handler}.ts`, `apps/site/tests/` | KEEP/EVOLVE / #764/#706/#740 | bounded SSG、決定性、route/link/fragment、metadata freshness、search/SEO/a11y等のgateを維持。固定本文数assertionは意味のcoverageを保って調整 |
| `apps/site/vercel.json`, root `vercel.json`, `apps/playground/` | KEEP / #631 | DocsとPlaygroundの独立配布を維持。root切替はこの台帳の退役作業に含めない |

テンプレートは別rendererではない。移行期間だけ既存rendererへ旧adapterと新blocksを
渡せる。adapterには上記の担当と「最後のconsumer」削除条件を設け、各PRで残る
consumerを記録する。最終#767の完了時には旧field/説明生成を全廃する。

### 関数と記法の最初の移行

| route（英語。日本語は `/ja` mirror） | 担当 / 判断 | 保全するもの |
| --- | --- | --- |
| `/docs/language/syntax/function-application/` | #764 first page / REWRITE | `language.syntax.function-application`、spec 1.4 / 2.7 / 3.1、通常適用/部分適用/Unit/型引数/括弧の規則とinvalid source |
| `/docs/language/types/function-types-and-currying/` | #765 / REWRITE | `language.types.function-types`、spec 2の関数型・結合・自動カリー化、既存source/診断 |
| `/docs/language/expressions/lambdas/` | #765 / REWRITE | `language.expressions.lambdas`、spec 3のlambda/operator section、高階関数、型と既存source |
| `/docs/language/syntax/pipelines-and-low-precedence-application/` | #765 / REWRITE | `language.syntax.pipelines`、spec 1の適用・優先順位・行継続、invalid source |
| `/docs/language/syntax/operator-precedence/` | #765 / KEEP rules + REWRITE copy | 詳細を直接引けるroute/anchor、演算子の正確な結合規則 |
| `/docs/language/traits/do-notation/`, `/docs/language/effects/{maybe,either}/` | #765 exits / KEEP until #767 | 既存identityと参照先。章の出口を用意し、関連先を全改稿してからでないと章を公開できない条件にしない |

`syntax/method-calls/`と`traits/method-calls/`のような似たrouteは統合候補になり得るが、
異なるsemantic obligationがあるためこの段階でroute MERGEを決定しない。
現時点のMERGE確定対象は重複説明・helperの構成であり、無検証のURL削除ではない。

## URL・anchor・provenance保全の手順

1. 対象commitのbuild manifestから英日routeと全symbol tupleを採取する。
   各routeのHTMLからid/fragment、locale links、canonical URL、breadcrumb、
   内外リンクを保存する。sourceのpath/hash/出力/診断、spec sectionと旧PR/Issueの
   実装履歴もbatch記録へ結び付ける。本文は移行素材、検証履歴は退行防止の証拠。
2. まず同じroute/identityで日英の新本文を作る。アンカー名を変更する場合は、
   旧fragmentを同じ意味の位置にaliasとして残すか、明示した行先へ互換対応する。
   正規spec/API情報を変えて本文の例に合わせない。
3. routeを統合する場合は `old URL + fragment → new URL + fragment` を列挙する。
   HTTP redirectだけでは旧fragmentが新文書に解決しない場合がある。
   新先の互換anchorも保持し、英日両方のdirect accessを確認する。
   現行SSGでalias/redirectを表現できなければ#764側で仕組みを検証してから使う。
4. canonical/locale/nav/検索/internal linkは新しい一つのidentityへ向ける。
   redirectページを正規記事として二重掲載しない。外部deep linkの互換確認も残す。
5. sourceをcompile・実行・整形し、日英の条件/出力/診断を照合する。
   buildのglobal route/link/fragment/translation/metadata checkと必要なbrowser gateを
   実行する。desktop/mobile/no-JS/keyboardで旧URL・fragment・locale往復も確認する。
6. 置換後に旧本文/helperを除去する。import・文字列のexample参照・test fixture・
   build host・nav・検索・CSSの残存を調べ、scopeに応じたgateを再実行する。
   canonical/spec sourceは不要になった本文と同じ理由で削除しない。

## 各移行PRへ残す記録

- 対象Issue、commit、route/identity、旧sourceと履歴、判断、置換先/redirect/anchor表。
- source hash、現行CLI version、実行command、stdout/diagnostic、format結果、spec/API provenance。
- 日英意味確認、agent自己レビュー、作者判断、実読者背景/理解/助け/再確認。
- build/browser/desktop/mobile/no-JS/a11y/locale/link/fragment結果。失敗と未実施理由。
- 削除したfile、残るadapter consumerと担当、公開影響、残件。チェック数を品質と混同しない。

## 763の検証記録

このPRは設計・監査・編集例のみ。公開本文・renderer・Compiler Reference・specは
変更しない。`editorial-examples/verify.py` は説明と実行sourceを照合する再現用の
scoped verifierで、site buildの入力や第二のlanguage compilerではない。

```sh
python3 docs/design/docs-site/editorial-examples/verify.py --cli target/release/seseragi
bun apps/site/scripts/check-content-map.ts
bun run check:site
```

最終結果と残件はこの節へ追記する。作者の美学承認・実読者の理解は未取得。
#764以後のPRはこの契約・台帳を参照し、#601の依存条件を再確認して着手する。


### #769編集レビューの反映

レビュー `5471192856` とinline `4231017725 / 4231017734 / 4231017741 / 4231017754`
を確認。記号の積極利用と必然性、部分適用例の限定とHeroへの自動転用禁止、
発見から文章を起こして日英通読する文体、実読者検証と退役の独立状態を契約・台帳へ
反映した。レビューは編集上の所見であり、作者の美学承認や技術gate成功とは扱わない。

### 実行済みのscoped検証

- 現行mainからrelease CLIをbuild: `seseragi 0.61.23`, commit `46755c08f4e9`。
- `editorial-examples/verify.py`: good/badともstdout `360\n`、canonical format、
  文書のコードと実sourceの一致を確認。372件の英日route/既存sourceと351件の
  semantic mapの包含を確認。
- good source SHA-256: `8edc39e23aed15c9d88fb704f27031e451a32549889323473582ce835c4381d2`。
- bad source SHA-256: `b50680a049f416fb9355a31b5aba7c651b0bacba9281dbe5fba5c3dee3889933`。
- `check-content-map.ts`: 351 unique routes、4 named spec source、pass。
- `compilerReferenceModules()`: 63 module / 1,812 symbol・instance、重複tuple検証pass。
- 設計文書のlocal linksと `git diff --check`: pass。
- `bun run check:site`: **fail / incomplete**。formatter/TypeScript/prose、現行release
  CLI build、canonical example実行、compiler metadata freshnessは通過。
  最初の全3,976 routeのproduction SSGは313,628msで成功し、662件のLanguage
  page-name linkと1,795件のLibrary page-name linkを確認。決定性用の2回目が
  既存420,000ms上限を超え、420,223msで `spawnSync bun ETIMEDOUT`。
  元のBun testは失敗後も別suiteへ進むため、所有するcheck process treeを停止した。
  残りのsuite・最終browser gateは未完了。timeout/assertion/coverageは緩めていない。
  このPRにsite implementation diffはなく、成功を主張せず#706側へ再検証を残す。

新しい公開ページのbrowser/実読者レビューはこの設計PRの成果ではない。
次の実装PRでsource・route・英日表示の実物に対して記録する。

## 764: 最初の自由な記事構成

#769の契約・台帳を基にした依存PRの実装候補。作者の美学承認は未取得のままで、
新本文を公開承認済みとは扱わない。#765の全章・#767のbulk移行はこの一ページの
成功から自動的に完了扱いにしない。

- 新しい `apps/site/src/model/article.ssrg` は、title/summaryのmetadataだけを共有し、
  `articlePage`へ任意の `Array<Block>` を渡す。`prose`は日英の一つの段落を同じ
  位置に置く補助。code/output/spec/callout/relatedは既存Blockを必要な順で組む。
  一律の本文field、導入/診断の義務、汎用main/do解説は生成しない。
  この足し算例はarchitectureの最小検証例で、章やHeroの代表コードではない。
  #765で、書いてみたくなる小さな関数の合成と記法の必然性が見える例を別途選ぶ。
- `syntax/function-application/{page,en,ja}.ssrg` を移行。旧guideを削除し、
  `ExplanationCopy`/`ReaderCopy`/`explainPage`への直接依存を除去。
  記事は小さなfn→呼び出し→出力→再利用→正確な規則→誤り→目的別の出口を選ぶ。
- KEEP: `language.syntax.function-application`、英日route、見出しtitle、
  既存6 anchor、canonical valid/invalid source、出力 `3`、診断 `SES-T0101`。
  Unit、型引数、左から右の評価、Effect値と実行の区別、括弧/タプル/演算子規則を保全。
- RETIRE済みの候補: このrouteの `guide.ssrg` と旧locale copy field。
  shared old templatesはまだ他のconsumerがあるので削除していない。
  `migration-inventory.tsv`のold_compositionは着手時snapshotとして保存し、
  新compositionはこのcheckpointとPRに記録する。
- 構造テストの旧guide/固定field assertionを、このrouteではpaired prose・canonical
  source・実際のrendered identity/anchor/規則へ置き換える。全conceptのinventory数、
  全pilotの実行・invalid・render coverageは保持する。短い一段落だけの記事も日英で
  同じrendererへ通し、不要な見出しや定型文が挿入されないことを検証する。
- browser verifierは実際のproduction出力で英日・320/390/1280px・JS有無を確認。
  詳細anchor、関連先へ移って戻る操作、キーボードlocale切替、overflow、page/console/
  response error、canonical code/出力/診断を確認し、既存browser gateからも呼び出す。

最終scoped/build/browser結果は追記する。残件は作者レビュー、章全体の実読者検証、
残る固定templateのconsumer移行、独立#706/#740の全gate再確認。

### 764の検証結果

- `bun test apps/site/tests/values-functions.test.ts apps/site/tests/explanations.test.ts`:
  **8 pass / 0 fail / 15,039 assertions**。6既存pilotの実行・invalid診断・修復と日英
  renderを維持。追加の一段落compositionを含め14 rendered pagesを確認。
- 現行release CLI `0.61.23 / 46755c08f4e9` を指定したproduction build:
  **3,976 routes、pass**。新しいpackage content digestをlock updateで更新後に実行。
  全体route/link/fragment/translation/metadataの既存検証も通過。
- `article-composition-browser.ts`をそのproduction出力で実行:
  **12 locale/viewport/JS cases、pass**（英日×320/390/1280px×JS有無）。
  valid/invalid source・出力・診断、詳細anchor、関連先往復、keyboard locale、
  horizontal overflow、page/console/response errorを確認。各caseのscreenshotを保存。
- Biome全site、対象TypeScript、4変更Seseragi sourceのcanonical format、diff whitespace:
  **pass**。`check-content-map.ts`と#763の限定例verifierも引き続きpass。
- 日本語320px/no-JSの全ページscreenshotをagentが目視確認。コードと出力が区別され、
  詳細規則と目的別出口まで読める。これは作者・実読者による美学/読解承認ではない。
- 初期scoped verifierの一段落HTML assertionはrendererのinline spanを考慮して修正。
  keyboard locale検証はEnter後のnavigation完了を明示的に待つよう修正して再実行。
- 全siteの `build.test.ts` は単独で再検証中。旧日本語の一文そのものを要求していた
  assertionを「空白での引数区切り」と `add 1 2` の意味の確認へ置換した。
  route数・link数・二回buildのhash比較・deadlineは変えない。
- `check:site`全suite完走は未確認。#763のrepeat-build timeoutと未完了suiteは上の
  記録を保持し、今回のscoped passを全suiteの成功として扱わない。

状態: 置換実装あり / 上記技術scope pass（全決定性は実行中） / agent編集・表示所見あり /
作者判断未確認 / 実読者未確認 / 新本文はPR候補。#769の契約に依存するPR #770。
