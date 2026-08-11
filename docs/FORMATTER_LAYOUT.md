# Formatter canonical layout

Seseragi formatterは、入力時の好みを保存するpretty printerではありません。同じtoken列と構文構造を、
CLI、LSP、WASM、Playgroundのどこから呼んでも同じcanonical bytesへ収束させます。固定値はindent 2 spaces、
目標line width 88 source columns、LF、file末尾newline一つです。長いString、URL、単一token、
改行が別expressionになるdelimiterなしapplicationは分割しません。

## 構文surface監査

| surface | canonical layout |
| --- | --- |
| import | namespaceとnamed itemを分けて扱い、alias / operator / long item groupをsource順のまま整形する |
| function / effect signature | 短ければ一行。長ければparameterの`->`、`with`、`fails`、`where`で改行 |
| lambda | parameterとbodyを同じexpressionとして空白を正規化し、長いbodyはoperator境界で改行 |
| application | 短ければ一行。長ければ既存delimiter内またはoperatorの意味境界で継続indent |
| pipeline / operator | 一行へ収まればcompact化。長ければleading operatorを揃える。spellingと順序は保持 |
| let | binding headと短いRHSを一行へ収め、長いRHSは一段継続indent |
| record / struct / array / list / tuple | 空・短いgroupはcompact。宣言を含めて88 columnsを超えるgroupだけitemごとに展開 |
| ADT | variantごとに一行とし、leading `|`を揃える |
| match | armごとの改行を保持し、arm bodyとnested delimiterを構造に従ってindent |
| pure block / do | `= {` / `= do {`が収まれば同じ行。body ownershipを一段のindentとして表す |
| impl / instance / trait / foreign | bodyを必ず展開し、implementation member間は空行一つ、bodyless memberは一行ずつ配置 |
| nested Html | element bodyと長いprops / children groupをdelimiter depthに従って展開 |
| generic / HKT | type applicationの`<...>`をtightに保ち、型parameter順を変えない |
| comments | comment textを変更せず、attached commentを次の対象から切り離さない |

## Layout規則

- lexer / lossless CSTが返したtokenだけを描画する。formatter独自のparserやoperator precedenceは持たない。
- punctuation、型適用、optional marker、prefix operatorの周囲はtoken kindから決め、通常のtoken間は空白一つにする。
- sourceの任意の改行や複数空白は出力を決めない。block item、match arm、ADT variant、commentは構造境界として残す。
- signatureは`->`の手前、constraint / effect clauseはkeywordの手前で折り返す。
- wrapの要否はwidth、break位置は構文、継続indentはowner nestingから決める。親がmultilineでも短い子groupはcompactのままにする。
- `=`とRHS openerは収まる限り同じ行へ置く。長いsignatureでは`->` / `with` / `fails` / `where`を先に使い、それでも収まらない場合だけRHSを一段下げる。
- comma groupは収まれば一行へ戻し、長い場合はitemごとに改行する。delimiterは対応するopen位置へ戻す。
- operator chainは収まれば一行へ戻し、長い場合はoperatorを一段継続indentへ置く。
- 通常applicationは、既存delimiter内またはleading operatorで意味を保持できる位置だけで折る。delimiter外で改行が別expressionになるapplicationは、tokenを追加せず意味を守るため88 columns超過を許す。
- import、field、variant、armをsortingしない。literal、identifier、custom operatorのspellingを変更しない。
- named importはcompact時に`{ item }`、long時にitemごとの行へ展開して`} from "..."`を同じ行へ保つ。現行grammarのoperator importはaliasを持たない。
- parse recovery nodeを含むsourceは書き換えず、共有parser diagnosticを呼び出し側へ返す。

## 回帰契約

入力と期待値の代表corpusは
[`canonical-layout.input.ssrg`](../crates/seseragi-formatter/tests/fixtures/canonical-layout.input.ssrg) と
[`canonical-layout.expected.ssrg`](../crates/seseragi-formatter/tests/fixtures/canonical-layout.expected.ssrg)、全surface監査は
[`style-contract.input.ssrg`](../crates/seseragi-formatter/tests/fixtures/style-contract.input.ssrg) と
[`style-contract.expected.ssrg`](../crates/seseragi-formatter/tests/fixtures/style-contract.expected.ssrg) です。
corpusは上のsurfaceを横断し、inputを一回formatしたbytes、expectedを再formatしたbytes、任意改行を変えたinputが
同じexpectedへ収束することを固定します。

共有入口は`seseragi_driver::format_module`です。native CLIの`seseragi format`、LSPの
`textDocument/formatting`、WASM API、PlaygroundのFormatはこの入口を再実装せず、その結果をそのまま返します。
