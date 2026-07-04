# 12. Parser・formatter・language server契約

## 12.1 目的

compiler、formatter、language serverは、同じsourceを異なる構文として解釈してはなりません。
特にcustom operator、generic delimiter、編集中の不完全なsourceがあっても、file全体を読める
共通のsyntax pipelineを使います。

この章は特定のparser libraryや内部class構成を要求しません。要求するのは、各段階の入力、
決定できる情報、error recovery後も保つ意味です。

## 12.2 共通syntax pipeline

sourceは概念上、次の順で処理します。

1. raw scan: triviaを含むtoken列を作る。
2. header scan: import、re-export、operator headerを集める。
3. module interface resolution: dependencyの公開名とoperator fixityを解決する。
4. lossless CST parse: 括弧とoperand順を保った構文木を作る。
5. fixity resolution: flat operator chainを結合規則に従って式treeへ変換する。
6. name resolutionとtype checking。

compiler、formatter、language serverは少なくとも1から5まで同じ規則を共有します。型検査結果を
使ってtoken境界やoperatorの結合を変更してはなりません。

## 12.3 raw operator token

raw scannerはcustom operatorの宣言やimportを参照しません。operator文字が連続する部分を一つの
raw operator runとして読み、spelling、前後の空白、source rangeを保存します。

run全体が `->`、`<-`、`|>`、`>>=`、`<$>`、`<*>`、`??`、`:=` などの固定tokenと完全一致する
場合だけ、固定tokenとして分類します。それ以外はcustom operator候補です。run全体が `//` なら
comment開始、`.`、`..`、`..=`、`...` のいずれかと完全一致する場合はdot/range系tokenです。
`<.>` のようにdotを含むより長いrunはcustom operator候補のままです。

隣接したoperatorを別々に書く場合は空白で区切ります。たとえば乗算の右operandを負にする式は
`a * -b` と書きます。`a*-b` を `a * -b` へ推測して分割しません。

型構文内の `<` と `>` はdelimiterです。nested generic末尾の `>>` は型parserが二つの
`>` delimiterとして扱えます。`<` と `>` だけからなるcustom symbolを禁止するため、expression
との意味衝突はありません。

## 12.4 header scanとmodule interface

header scanは関数本体や値の式をparseせず、次だけをerror recovery付きで集めます。

- importとre-exportのmodule specifier
- top-level operator宣言のsymbol、fixity、precedence
- 公開operatorの型scheme

private operatorも宣言module自身のfixity tableへ入り、`pub operator`だけがmodule interfaceへ
入ります。operatorのfixityとprecedenceは公開APIの一部です。変更すると、そのoperatorをimport
するmoduleの構文解決をinvalidateします。

operatorはtop-levelでのみ宣言でき、宣言位置より前でも同じ意味で使えます。local scope、条件分岐、
型推論結果によってfixityを変えられません。import cycleは禁止されるため、dependency interfaceは
有限の順序で解決できます。

## 12.5 flat operator chainとfixity resolution

CSTはcustom operatorを見つけても、scannerや初期parse中に左右へ結合しません。次の式は、まず
operandとoperatorをsource順に保つflat chainとして読めます。

```seseragi
a <+> b <*> c
```

module interface解決後、standard operatorとscope内のcustom operatorを一つのfixity tableへ入れ、
precedenceとassociativityから式treeを一意に作ります。shunting-yard、Pratt parser、別のalgorithmの
どれを使っても構いません。

次の場合はコンパイルエラーです。

- custom operator候補がscopeのoperator namespaceに存在しない。
- 同じsymbolが複数importされ、aliasでも解消されていない。
- 非結合operatorを括弧なしで連鎖する。
- 同じprecedenceで異なるassociativityのoperatorを括弧なしに混ぜる。
- operandが不足している。

括弧は常に明示的な結合境界です。fixity resolutionは括弧を越えて式を組み替えません。
operator overloadのinstance選択はこの後の型検査で行い、parse treeを変更しません。

## 12.6 不完全なsource

language server用parseは、次を含んでもfile末尾までCSTを構築します。

- 閉じていない括弧・block・String
- operandが片側だけのoperator
- 書きかけのimport、型parameter、関数signature
- 未解決または書きかけのcustom operator宣言

欠けた箇所にはsource rangeを持つerror nodeまたはmissing tokenを置きます。未知のoperatorも
spellingを失わないnodeとして残します。一つの未知operatorを理由に、後続するtop-level宣言を
同じ式へ吸収してはなりません。

dependency interfaceが一時的に取得できない場合、language serverはoperator chainをflatなまま
保持できます。最後に成功したinterfaceを補助表示へ使っても構いませんが、それを最新sourceの
確定結果として報告してはなりません。compilerの通常buildは未解決interfaceやerror nodeを
受理しません。

## 12.7 language server

language serverはsource fileを実行せず、module interface metadataとsourceから情報を得ます。
operator symbolの定義移動、hover、completion、semantic tokenはoperator namespaceを使います。

local operator header、import一覧、dependency interfaceのいずれかが変わった場合、そのfileと
依存するopen documentのfixity resolutionを更新します。変更のないfunction bodyだけを編集した
場合、dependency全体のheader scanをやり直す必要はありません。

unknown operatorやambiguous fixityがあっても、識別子、型、後続宣言のnavigationを可能な範囲で
提供します。一次diagnosticは原因のoperatorへ置き、同じ原因から生じた型errorをcascadeとして
抑制できます。

## 12.8 formatter

formatterはlossless CSTと解決済みfixityを入力にします。custom infix operatorの両側には空白を
一つ置きます。改行しても結合が変わる場合は括弧を補い、意味を変えてはなりません。

unknown operator、ambiguous fixity、missing operandを含む範囲は、tokenを削除・並べ替えせず、
安全に整形できる周囲だけを整形します。formatter独自のoperator precedenceを持ちません。

## 12.9 最低conformance case

compilerとlanguage serverは、少なくとも次を同じ結合結果として扱います。

- operator宣言より前での利用
- 明示importしたoperatorの利用
- 同じprecedenceの左結合・右結合・非結合operator
- standard operatorとcustom operatorの混在
- nested genericの末尾 `>>`
- unknown operatorを含む不完全なfileの後続宣言
- dependency側のfixity変更後の再解決

## 12.10 syntax highlight

syntax highlighterはproject全体の型検査がなくても、comment、String、number、keyword、delimiter、
固定operator、custom operator候補をlexical tokenから分類できなければなりません。importと
module interfaceが得られる場合、language serverのsemantic tokenで型、constructor、関数、trait、
operator declarationとoperator referenceを追加分類します。

lexical highlightとsemantic highlightでtoken境界を変えてはなりません。unknown operatorも一つの
operator tokenとして表示し、複数の既知tokenへ推測分割しません。不完全なStringやcommentの
色付けが後続file全体へ漏れないrecovery caseをconformance testに含めます。

## 12.11 playground

playgroundは専用の簡易parserやruntime semanticsを持ちません。compiler frontend、formatter、
diagnostic、実行hostを再利用し、CLIと同じsourceへ同じ結果を返します。browserなどhost targetの
差はservice providerで表し、言語構文の分岐にしません。

playground sampleは `examples/spec/` のsourceから生成または直接読み込みます。同じprogramを
playground source内へ手作業で複製してはなりません。exampleの変更を取り込んだ生成物には
source fileと内容hashを記録し、stale sampleを検出できなければなりません。

## 12.12 languageとしての最低surface

仕様example一つを「対応済み」とするには、少なくとも次が同じsourceに対して成立する必要が
あります。

- parseとtype check
- format後の再parseで同じ意味を保つ
- CLIまたは対応hostでの実行
- language serverのdiagnostic、hover、definition、completion
- lexicalおよびsemantic syntax highlight
- playgroundでの読み込みと実行

compilerだけが受理する構文、またはplaygroundだけが変換して受理する構文を言語機能の完成とは
扱いません。
