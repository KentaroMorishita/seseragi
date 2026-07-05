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

namespace aliasのmember completionはmodule interfaceの型・値・constructor・trait・公開namespaceを
contextに応じて提示します。同じspellingが型namespaceと値namespaceにある場合もsymbol identityを
混同せず、definition、rename、reference検索は参照位置のnamespaceに属するsymbolだけを更新します。
local functionは親functionのdocument symbolへnestし、definition、reference、renameは3.2のlexical scopeだけを
対象にします。rec groupのmemberは互いのreferenceを解決し、group外の同名bindingと混同しません。

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

canonical formatは次で固定します。projectごとのstyle optionで出力を分岐させません。

- UTF-8、LF、file末尾newline一つ。trailing whitespaceと末尾の余分な空行を除く。
- indentは2 spaces、tabをindentへ出力しない。目標line widthは100 Unicode scalarで、token、URL、長い
  Stringを壊してまで強制しない。
- `{}` は空block / recordだけに使う。一行へ収まる単純なrecordとtupleは一行、複数行ではitemごとに
  改行しtrailing commaを付ける。`{ value, }` のcommaはrecord / block判別に必要なので削除しない。
- function signature、constraint、effectのwith / failsは100 columnsを超える場合、parameterとclauseの
  意味境界で改行する。
- pipelineは一行へ収まれば一行、複数行ではsourceを先頭に置き、各 <code>&#124;&gt;</code> を同じindentで
  新しい行の先頭に置く。`$` は右辺がif / match / do / lambdaまたは長い式の場合だけ直後で改行できる。
- import、declaration、match arm、record fieldのsource順を変更しない。unused importを削除しない。
- top-level declaration間は空行一つ。連続したline commentとattached doc commentを対象から離さない。
- optional record field / query markerはfield名へ空白なしで付け、`id?: String`、`{ id? }` と出力する。
- String、number、custom operatorのspellingは、構文上必要なescape修復を除いて保持する。

formatを二回適用したbytesは一回目と同じでなければなりません。format後に再parse・fixity resolutionしたtreeは、
source rangeとtriviaを除いてformat前と同じです。range formatはrangeと交差する最小の完全CST nodeだけを対象にし、
declaration途中を独自grammarで整形しません。error node内はindent修復だけを許し、tokenの追加・削除・移動を
行いません。

## 12.9 最低conformance case

compilerとlanguage serverは、少なくとも次を同じ結合結果として扱います。

- operator宣言より前での利用
- 明示importしたoperatorの利用
- 同じprecedenceの左結合・右結合・非結合operator
- standard operatorとcustom operatorの混在
- nested genericの末尾 `>>`
- unknown operatorを含む不完全なfileの後続宣言
- dependency側のfixity変更後の再解決
- grammarで使うkeywordと1.9のreserved/contextual keyword分類の一致
- `{}`、`{ value }`、`{ value, }`、`{ field: value }` のrecord/block判別
- record/struct valueとpatternのfield shorthand、およびvalue側だけのspread
- block内のlocal function、self recursion、local rec group、forward reference rejection
- Effect / Stream requirement内の単独`&`と、値位置でのinvalidな単独`&`

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

playground sampleは `examples/spec/lessons/` のsourceから生成または直接読み込みます。同じprogramを
playground source内へ手作業で複製してはなりません。lessonの変更を取り込んだ生成物には
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

`examples/spec/COVERAGE.md` は規範的な意味を追加する文書ではなく、各仕様機能を上の検証surfaceへ
結びつけるindexです。学習上複雑なnegative caseやmulti-file caseは `examples/spec/fixtures/` に置き、
lessonへ網羅性の都合だけで無関係な構文を詰め込んではなりません。

## 12.13 diagnostic contract

compiler、formatter、package resolver、binding generator、language serverは共通のDiagnostic data modelを
使います。一件のdiagnosticは少なくとも次を持ちます。

```text
code: stable diagnostic code
severity: Error | Warning | Information | Hint
message: human-readable summary
primary: source URI and UTF-8 byte range
labels: zero or more related source ranges with messages
notes: zero or more source-independent explanations
helps: zero or more concrete next actions
fixes: zero or more versioned text edit groups
```

codeは `SES-P####`（parse / fixity）、`SES-N####`（name / module）、`SES-T####`（type / trait）、
`SES-E####`（Effect / resource）、`SES-K####`（package / manifest）、`SES-F####`（foreign / binding）、
`SES-L####`（lint）、`SES-I####`（compiler invariant）のいずれかです。番号は4桁で、一度公開したcodeを別の
原因へ再利用しません。messageの改善や翻訳は互換性対象ではなく、toolはcodeで判定します。codeの削除・分割は
release noteへ記録し、同じlanguage major内では旧codeから新codeへのalias metadataを提供します。

初期core registryは次です。より詳細な原因を追加するときも既存codeの意味を広げません。

| code      | severity | 意味                                        |
| --------- | -------- | ------------------------------------------- |
| SES-P0001 | Error    | token / grammarを回復不能な位置で拒否       |
| SES-P0101 | Error    | scopeに存在しないcustom operator            |
| SES-P0102 | Error    | operator fixityが曖昧または両立しない       |
| SES-P0201 | Error    | literalに未定義または不正なescapeがある     |
| SES-P0202 | Error    | Char literalがUnicode scalar一個でない      |
| SES-P0203 | Error    | 数値literalの綴りまたは値域が不正           |
| SES-N0001 | Error    | nameを解決できない                          |
| SES-N0101 | Error    | import / re-exportが曖昧                    |
| SES-T0001 | Error    | standard `todo` placeholderが残っている     |
| SES-T0101 | Error    | expected typeとactual typeが一致しない      |
| SES-T0201 | Error    | required trait instanceが存在しない         |
| SES-T0202 | Error    | trait instanceがambiguous                   |
| SES-T0301 | Error    | matchがnon-exhaustive                       |
| SES-T0302 | Error    | 到達不能なmatch arm                         |
| SES-T0401 | Error    | derivingとexplicit instanceが重複           |
| SES-T0501 | Error    | requirement mergeを許可されない型位置で使用 |
| SES-T0502 | Error    | trait instanceを`impl`で宣言                |
| SES-E0001 | Error    | Effect requirementまたはfailure型が不一致   |
| SES-K0001 | Error    | manifest / lockfileが不正または不一致       |
| SES-F0001 | Error    | foreign ABIへ安全に変換できない             |
| SES-L0101 | Warning  | standard HTML props literalの未知field      |
| SES-L0201 | Warning  | deprecated symbolを参照                     |

tableのcodeが十分具体的でない場合はlabels / notesで詳細を示し、message差だけのためにcodeを増やしません。

source rangeは0-based・end-exclusive UTF-8 byte offsetです。CLIのline / column表示は1-based Unicode scalar、
LSP境界ではclientがnegotiationしたUTF-8 / UTF-16 / UTF-32 position encodingへ変換します。変換不能なmid-scalar
positionを受け取った場合は直前のscalar boundaryへ丸めずrequest errorにします。generated sourceのdiagnosticは
source mapで元のSeseragiまたはforeign declarationへ戻し、生成fileだけをprimaryにしません。

Errorはbuildまたは要求されたcode generationを失敗させます。Warning、Information、Hintは既定では成功statusを
変えません。`--deny-warnings` はWarningをprocess failureへ昇格しますが、diagnostic自身のseverityとcodeは
書き換えません。syntax / name / type errorをlintへ降格できません。formatterの安全に整形できない範囲はError、
未整形のまま保持できる範囲はWarningです。

同一runのdiagnostic順はcanonical source URI、primary start byte、end byte、severity、codeの順です。同じroot
causeから生じる型errorはprimary diagnosticのrelated noteへ畳み、error node一つから後続declarationすべてへ
cascadeさせません。最大件数で打ち切る場合は `SES-I0001` ではなく専用のInformation diagnosticを最後に一件出し、
元error件数を隠しません。

machine-readable出力はschema version、tool version、language version、Unicode version、diagnostic配列を持つJSON
です。未知fieldをreaderは無視し、schema majorが違う場合は拒否します。text edit fixは対象document version、
UTF-8 range、replacementを持ち、version不一致ならLSP / CLIとも適用しません。一つのfix groupはall-or-nothingで、
overlapするeditを含めません。

## 12.14 type inference explanation

通常のtype diagnosticは「期待型」「実型」「両者を要求した最小のsource span」を先に表示します。内部の型変数ID、
dictionary名、unification stackをそのままuserへ出しません。generic constraint failureは、要求されたtrait、具体型、
constraintの導入元、検討したが不適合だったvisible implを区別します。

`--explain <diagnostic-id>` とLSPのexplain commandは、型推論を再実行せず保存済みconstraint graphから次をsource順に
返します。

1. expressionへfresh型を割り当てた箇所
2. application、annotation、operator、patternから生じたconstraint
3. generalize / instantiateしたlet binding
4. trait candidateの探索とreject理由
5. 最初に両立しなかったconstraint pair

trace nodeはstableなrun-local ID、source range、rule name、input / output typeを持ちます。同じsource・dependency
interface・compiler versionでは同じ順序です。型はuser syntaxでpretty-printし、alpha-equivalentな型変数名を
出現順 `a`, `b`, `c` へ正規化します。error recovery由来の仮型は `<unknown>` と表示し、確定型に見せません。

## 12.15 exhaustive match fix

non-exhaustive matchはErrorで、専用codeと不足constructorの一覧を持ちます。LSP code actionは型宣言順に不足armを
追加し、各bodyをcompile errorになるplaceholder `todo "handle Constructor"` にします。ADT constructorはqualified
name、BoolはFalse / True、Maybe / Eitherはstandard constructor、有限tuple / record patternは既存armから必要な
深さだけ展開します。Int、Stringなど無限domainにconstructor列挙fixを出しません。

既存のcatch-all `_` または変数patternがある場合はnon-exhaustive errorではありません。guard付きarmだけでは
constructorをcoverしたとみなさず、fixはunguarded armを追加します。`_ -> todo ...` 一件を追加する別fixを提供して
もよいですが、安全性を隠すためpreferred fixにはしません。edit位置は最後のarmの後、closing braceの前で、既存
commentとformatterのtrailing delimiter規則を保ちます。

## 12.16 document commentとAPI document

`///` は直後のdeclarationへ付くdocument comment、`//!` はmodule document commentです。どちらもlexicallyは
line commentで、raw scannerがordinary `//` と区別してtextとrangeを保持します。module commentはshebangを
使わないsourceの先頭、importより前にだけ置けます。declarationとの間にblank lineまたは別declarationがある
`///` はattachedせずordinary commentとして警告します。連続行は一つのMarkdown documentへ結合します。

document bodyはCommonMarkを基準にheading、paragraph、list、table、link、inline code、fenced code blockを
扱います。raw HTMLとscriptはescapeし、生成documentで実行しません。`[User]`、`[module.Name]`、
`[(<+>)]` のようなcode linkはdeclaration位置のnamespaceで解決し、曖昧・非公開・不存在ならWarningです。
public declarationからprivate symbolへのlinkは公開documentで解決成功にしません。

API document generatorはmodule docs、public declarationのsignature、constraint、deriving / instance / impl、source link、
document commentをmodule interface metadataから生成します。型やdefaultをcommentから再parseしません。同じ
interface、comment、tool versionから生成するHTMLとJSON search indexはbyte単位で決定的です。private APIは
明示option時だけ含め、public documentのlink identityへ混ぜません。

fenced blockのinfo stringは次を予約します。

- `seseragi`: parseとtype checkする。実行しない。
- `seseragi run`: deterministic test serviceで実行し、block内の `// Expected stdout:` とbyte単位比較する。
- `seseragi compile_fail SES-T####` のようにcategoryを含むcodeを書くと、compileが失敗し、指定codeを
  少なくとも一件含むことを要求する。
- `seseragi no_test`: syntax highlightだけ行い、doc test対象外であることを明示する。

doc testはdocumented moduleと同じpublic namesをunqualifiedに参照できるsynthetic moduleとしてcompileしますが、
private nameへの参照は拒否します。各blockは独立し、前blockのbindingを引き継ぎません。run blockはnetwork、real
filesystem、real clock、global randomを既定で持たず、必要serviceをfixtureで明示提供します。doc test failureは
元comment rangeをprimaryとするdiagnosticになり、generated wrapper pathをuserへ要求しません。

## 12.17 test runner

`seseragi test`は11.7のtest treeを実行します。filterは正規表現ではなくfull nameに対するcase-sensitiveな
Unicode scalar substringです。`--filter TEXT`は一致するcase、`--exact FULL_NAME`は完全一致一件を選び、
両方は同時指定できません。選択結果が0件なら成功扱いにせずexit code 2です。

`--jobs N`は正の並列case数で、既定値はmanifestに従います。caseは独立したenvironmentとroot resource scopeで
実行します。完了順にかかわらずreportはcanonical discovery orderで出し、caseのConsole / Logger captureを
他caseへ混ぜません。runner cancellationでは実行中caseをcancelしてfinalizerを待ち、未開始caseを開始しません。
timeout後はcancellationを要求し、`cleanup_grace_ms`までfinalizerとchild Fiberを待ちます。超過時はcaseをleak
failureとして記録してisolated test runtimeを破棄します。このforced teardownだけは未完了finalizerを保証せず、
通常programのresource semanticsとして利用できません。

既定text reporterはcolorを除いた次のstable lineをstdoutへ出し、durationを含めません。

```text
PASS module::suite::pass-case
PASS module::suite::other-case
SKIP module::suite::skip-case -- reason
FAIL module::suite::fail-case
2 passed; 1 failed; 1 skipped
```

failure detail、assertion diff、captured Console / Logger、defect stackは対応するFAILの後にstderrへ出します。
parallel実行でもstderr blockはdiscovery orderです。TTY colorは許しますが、redirect時とsnapshot reporterでは
ANSI sequenceを出しません。

test commandのprocess-capable targetにおけるexit codeは次です。

| 結果                                                      | code |
| --------------------------------------------------------- | ---: |
| 選択した全caseがpassまたはskip                            |    0 |
| assertion、typed failure、timeout、leak、defectが一件以上 |    1 |
| manifest / compile / discovery error、選択case 0件        |    2 |
| runner自身のcompiler invariant違反                        |   70 |
| host cancellation                                         |  130 |

case failureがあっても残りcaseを既定で実行します。test caseはprocessを直接終了できず、runnerは各caseの
finalizer完了後にだけsummaryとexit codeを確定します。signalを持たないtargetは同じ分類をhost resultとして返します。

## 12.18 deprecation metadata

公開APIの移行案内はdocument comment内の慣習や汎用annotationではなく、閉じたdeprecation clauseで表します。

```seseragi
/// 古いwire formatを読む互換入口です。
pub deprecated "Use decodeUser instead" since "0.3.0"
fn decodeLegacy text: String -> Either<DecodeError, User> =
  decodeUser text
```

clauseは`pub`の後、declaration keywordの前に置きます。top-levelのnamed declaration、trait method、inherent
impl member、foreign memberに付けられます。instance member、local declaration、import、re-export、field、
ADT constructor、rec group全体には付けられません。messageは空でないString、optionalなsinceはSemVerの
canonical versionでなければcompile errorです。metadataは型、名前解決、visibility、ABI、評価を変えません。
汎用 `@annotation` / `#[attribute]` syntaxやmacro expansionをこのclauseから導入しません。

deprecated symbolを参照すると、declaration自身のbody内を除き `SES-L0201` Warningを参照位置へ出します。
message、since、declarationへのrelated locationを添え、同じsource expressionから同じsymbolへの参照には一件だけ
出します。importだけでは未使用symbolのWarningを出さず、re-exportはそのpublic APIを新しく公開する参照なので
Warningを出します。deprecated trait methodのinstance実装自体には出さず、method callに出します。
`--deny-warnings` の扱いは12.13と同じです。

module interfaceはdeprecationのmessage、since、source locationを保持します。LSPはcompletionとdocument symbolへ
deprecated tag、semantic tokenへdeprecated modifier、hoverへmessageとsince、reference diagnosticへ
DiagnosticTag.Deprecatedを付けます。rename、definition、completion filteringからsymbolを消しません。API documentの
HTMLはsignature直前へdeprecation noticeを表示し、JSON search indexは `deprecated: { message, since? }` を持ちます。
同じmetadataをgenerated bindingとsource mapでも保存します。

`.d.ts` converterはdeclarationへ直接付いたJSDoc `@deprecated` のtextをmessageとしてこのclauseへ変換できます。
JSDocだけからsinceを推測せず、converter設定に明示mappingがない限り省略します。foreign hostのdeprecated metadataが
取得できない場合に名前やversionから推測してはなりません。

## 12.19 stable tool optionとtarget capability query

CLI optionはcommandごとの文字列慣習だけでなく、machine-readableなschemaを持ちます。
`seseragi options --json` はUTF-8 JSON一件をstdoutへ出し、少なくとも次を含みます。

```text
schema: option schema major
toolVersion: compiler distribution version
languageVersion: accepted language version
unicodeVersion: shared Unicode data version
commands: command name順のarray
  name: stable command id
  options: option id順のarray
    id: stable kebab-case id
    flags: accepted CLI spellings
    value: Boolean | Integer | Float | String | Enum | Path
    enumValues: Enumの場合のclosed values
    repeatable: occurrenceを複数許すか
    default: target / manifest選択前のdefaultまたはnull
    manifestKey:対応するcore manifest keyまたはnull
    stability: Stable | Experimental
```

schema 1でstableなcompiler共通optionは `target`、`profile` (`development | release`)、
`diagnostic-format` (`text | json`)、`deny-warnings`、`locked` です。buildはさらに `emit`
(`javascript | declarations | interface | none`) を持ちます。formatは `check`、`stdin-file`、
`range-start-byte`、`range-end-byte`、`diagnostic-format` を持ちます。rangeは両端を同時に指定し、
0-based end-exclusive UTF-8 byteです。formatterのindent、line width、quote、trailing commaを変更する
style optionはなく、12.8のcanonical formatだけを生成します。

Stable optionのid、値型、意味を同じlanguage major内で別用途へ再利用しません。flag spellingを置き換える場合は
一つのmajorの間deprecated aliasとしてschemaへ残します。Experimental optionは先頭を `experimental-` とし、
generated artifact metadataへ値を記録します。未知option、値型不一致、closed enum外はcommand開始前のexit code 2
です。environment variableやtarget adapterが同名optionを暗黙注入してschemaから隠してはなりません。

`seseragi target capabilities TARGET --json` はapplicationをcompile・実行せず、target adapterのbuild-time
metadataを返します。schema 1は次を持ちます。

```text
schema: 1
target: { id, adapterVersion, runtimeFamily, runtimeVersion }
services: canonical service name順のarray
  { name, availability: Required | Optional | Unavailable, details }
features: stable feature idからBoolまたはclosed Stringへのobject
```

標準service nameは少なくとも `clock`、`console`、`entropy`、`fileSystem`、`httpClient`、`logger`、
`process`、`random`、`stdin`、`webDom` です。detailsはserviceごとのJSON objectで、processならsignalsと
gracefulShutdown、fileSystemならatomicReplaceとsymlink、httpClientならstreaming、webDomならhydration、
runtimeならthreadsを表せます。標準feature idは `foreign.pure-load`、`foreign.task-load`、
`source-map.cross-language`、`runtime.threads`、`runtime.wasm` です。未知fieldは無視し、schema major不一致は
拒否します。array順とobject key順はUTF-8 byteの辞書順で、同じadapter metadataからbyte単位で同じJSONを返します。

未知target、adapter load failure、schema不一致はdiagnostic JSONをstderrへ出してexit code 2、成功は0です。
capability queryはhostの現在権限やnetwork到達性をprobeせず、「build artifactが要求可能なservice」を答えます。
compilerはentry pointのclosed requirementをこのmetadataと照合し、Requiredはhostが必ず提供、Optionalは
adapter configurationで提供可否を確定、Unavailableはbuild errorにします。source codeからcapability queryを
呼んでtargetごとに型や意味を分岐するcompile-time reflectionは提供しません。差異はmanifest target、module境界、
Effect service providerで明示します。
