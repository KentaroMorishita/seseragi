# 7. TypeScript interop

## 7.1 境界の原則

TypeScript interopは、TypeScriptの型システムをSeseragiへ持ち込む機能ではありません。
TypeScript moduleの値を、明示したcalling conventionと変換規則でSeseragiの型付き境界へ
接続する機能です。

interopは次を保証します。

- Seseragi内部の型意味論はTypeScript compilerに依存しない。
- `null`、`undefined`、throw、Promise rejectionは境界で可視化する。
- TypeScriptのoverloadや高度な型を推測で一つのSeseragi型へ潰さない。
- importした関数もSeseragi側ではカリー化関数として見える。

## 7.2 foreign module

TypeScript / JavaScript moduleはforeign blockでbindします。

```seseragi
foreign "typescript" from "calendar-lib" {
  opaque type Date

  pure fn format date: Date -> pattern: String -> String
}
```

foreign blockは既定でmodule-privateです。生成bindingや他module向けのraw interop moduleは
`pub foreign` と書き、block内の名前を公開できます。個々のmemberだけを選択公開する場合は
private foreign blockを手書きの `pub fn` / `pub alias` で包みます。

`from` のspecifierはTypeScript host resolverへ渡します。relative specifierはbinding fileから、
bare specifierは `seseragi.toml` の `[foreign.typescript]` とhost package resolverから解決します。
load identityはspecifier文字列ではなくresolverが返すexact host module identityです。異なるrelative spellingや
re-export pathが同じidentityへ解決される場合も、一runtime内で一moduleとして扱います。

resolved host moduleごとに、全foreign blockを合わせてload modeを決めます。

- pure function / value / constructor / method / propertyが一件でもあれば**pure-load**。
- opaque typeとtask memberだけなら**task-load**。

pure-loadでは、対象moduleとそのtransitive host dependencyの評価が同期的・非throw・決定的で、外部状態を観測も
変更もしないことをbinding authorが保証します。runtimeはSeseragi dependencyの初期化順でhost moduleを一度評価し、
exportをbindしてから、そのmoduleを参照するtop-level pure expressionを評価します。保証違反はbinding defectです。

task-loadでは、Seseragi moduleのimportと初期化だけでhost moduleを評価しません。最初のtask memberを実行した時点で
lazy loaderを開始し、成功後に対象exportを呼びます。同じidentityへの並行した初回callは一つのloadを共有します。
loadのsuccessまたはfailureはruntime中memoizeし、後続callでmodule initializerをretryしません。load failureは
phaseをModuleLoadとして保持する`Js.Error`です。

host module load自体にportableなcancellationはありません。待機中のforeign Taskがcancelされた場合、そのcallerは
cancellationを観測しますが、共有loadは継続して結果をmemoizeできます。load成功後にtask memberを呼び始めている場合の
cancellationは7.11と各bindingの明示contractに従い、推測でhost operationをcancelしません。

別foreign blockのtask memberだけを見てlazy-loadと判断してはなりません。同じresolved identityにpure memberがあれば
module全体がpure-loadです。module初期化にside effectや失敗がある場合、そのidentityへpure memberを宣言できず、
すべてtask memberとしてbindしなければなりません。

foreign blockのlocal名は通常のimportと同じmodule scopeへ入ります。export名が異なる場合は
文字列で指定します。

```seseragi
foreign "typescript" from "legacy-lib" {
  pure fn parseDate text: String -> Js.Nullable<Date> = "parse_date"
  pure value libraryVersion: String = "VERSION"
}
```

## 7.3 foreign call mode

foreign関数は `pure` または `task` のどちらかを必ず指定します。

### pure

```seseragi
pure fn clamp value: Float -> min: Float -> max: Float -> Float
```

binding authorは、対象が同期的・決定的・非throw・外部状態を観測も変更もしないことを
保証します。戻り値がStringやIntなどへ安全に変換できることも保証に含みます。違反は
binding defectです。pure関数がPromiseを返す宣言は不正です。

### task

```seseragi
task fn readFile path: String -> String
task fn fetchJson url: String -> Js.Unknown
```

宣言された最後の型はsuccess値です。Seseragiから見える型は自動的に次へなります。

```text
readFile  : String -> Task<Js.Error, String>
fetchJson : String -> Task<Js.Error, Js.Unknown>
```

adapterは関数呼び出し時の同期throwと、戻り値がPromiseLikeならrejectionを捕捉します。
同期値なら即座に成功したTaskへ変換します。Taskをrunするまで対象関数は呼びません。
task-load moduleでは同じTask error channelがmodule load failureも扱います。`Js.Error` metadataは
ModuleLoad、SynchronousThrow、PromiseRejectionを区別し、diagnosticとcross-language stackでphaseを失いません。

## 7.4 arityとcurry

foreign関数の宣言はSeseragiの通常構文を使いますが、adapterはすべての値引数を集めてから
TypeScript関数を一度だけuncurried callします。

```seseragi
pure fn slice text: String -> start: Float -> end: Float -> String
```

`slice "hello" 1.0 4.0` はhost上で `slice("hello", 1, 4)` を一度呼びます。`slice "hello"`
はhostを呼ばず、残り二引数を待つSeseragi関数です。

## 7.5 境界型

TypeScript backendは次の型をforeign宣言内とinterop moduleで提供します。

```text
Js.Unknown
Js.NullOr<A>
Js.Nullable<A>
Js.UndefinedOr<A>
Js.Promise<A>
Js.Error
Js.Object
Js.Number
Js.String
Js.Null
Js.Undefined
Js.MutableArray<A>
Js.Callback<Args, Result>
```

- `Js.Unknown` はdecoderで検査するまで通常値として操作できない。
- `Js.NullOr<A>` は `A | null` の境界表現。
- `Js.Nullable<A>` は `Js.UndefinedOr<Js.NullOr<A>>` のconvenience alias。
- `Js.UndefinedOr<A>` は `A | undefined` の境界表現。
- `Js.Promise<A>` はraw Promiseで、通常コードではTaskへ変換する。
- `Js.Object` はfieldを直接読めないopaque object。
- `Js.Number` と `Js.String` は検査前のhost primitive。
- `Js.Null` と `Js.Undefined` はそれぞれのhost singleton。
- `Js.MutableArray<A>` はhostとidentityを共有する可変array。
- `Js.Callback` はhost callback ABIを表すopaque値。

境界型から通常型への暗黙変換はありません。

## 7.6 primitive変換

| Seseragi | TypeScript境界 |
| -------- | -------------- |
| `Bool`   | `boolean`      |
| `Char`   | `string`       |
| `String` | `string`       |
| `Float`  | `number`       |
| `Int`    | `bigint`       |
| `Unit`   | `undefined`    |
| `Never`  | `never`        |
| `Bytes`  | `Uint8Array`   |

CharはUnicode scalar一個を表すJavaScript stringへ写像し、入力時にscalar数が一個であることと
unpaired surrogateを含まないことを検査します。TypeScriptの`string`からCharを自動推論しません。
`Int` は64 bit精度を失わないため `bigint` へ写像します。`number` をIntとして受け取るbindingは
`Js.Number` として受け、有限・整数・範囲内であることをdecoderで検査します。
BytesとUint8Arrayは10.8の規則により両方向でcopyし、mutable viewを共有しません。

## 7.7 collectionとrecord

- `Array<A>` は境界で `ReadonlyArray<Ts(A)>` と相互変換する。
- `Bytes` は境界で `Uint8Array` と相互変換し、必ずsnapshot copyする。
- `List<A>` は自動変換しない。`Array` を経由する明示関数を使う。
- tupleは同じ長さのreadonly tupleへ変換する。
- recordはreadonly objectへfieldごとに変換する。
- optional record fieldはTypeScriptのreadonly optional propertyへ変換し、absentとpresentを保持する。
- structとADTはforeign引数へ暗黙変換しない。明示adapterまたは生成ABIを使う。

TypeScript objectは可変でありうるため、foreign objectをSeseragi recordとして参照共有
しません。入力時にsnapshotを作り、出力時に新しいobjectを作ります。

native optional record fieldのabsentはpropertyを生成しません。presentなfieldの値がMaybeや
Js.UndefinedOrであってもpresenceを別に保持します。raw `.d.ts` optional propertyは8.5のとおり既定では
Js.UndefinedOrへ変換し、missingとexplicit undefinedを推測でnative optional fieldへ統合しません。

## 7.8 optional、nullable、rest

raw foreign bindingのoptional parameterは `Js.UndefinedOr<A>` と宣言します。
`Js.Undefined` はargumentの省略ではなく `undefined` 一個へ変換し、host functionのarityを
保ちます。通常のSeseragi APIでは手書きadapterが `Maybe<A>` を受け、raw型へ変換します。
末尾argumentを本当に省略する必要があるAPIには、引数数ごとに別bindingを定義します。

nullable resultは `Js.Nullable<A>` で受け、`Js.nullableToMaybe` を明示的に使います。

rest parameterは次の形で末尾に一つだけ宣言できます。

```seseragi
pure fn join separator: String -> ...parts: Array<String> -> String
```

adapterは `parts` をhost callの末尾へspreadします。

## 7.9 overload

TypeScript overloadは自動dispatchしません。一つのforeign名へ、選択したoverloadごとに
異なるlocal名を付けます。

```seseragi
foreign "typescript" from "formatter" {
  pure fn formatText value: String -> String = "format"
  pure fn formatNumber value: Float -> String = "format"
}
```

複数signatureを一つにunion化したり、呼び出し時にTypeScript checkerへ選択を委譲したり
しません。

## 7.10 class、constructor、method、property

foreign classはopaque typeとしてbindします。

```seseragi
foreign "typescript" from "db-client" {
  opaque type Client
  opaque type Rows

  task constructor fn newClient url: String -> Client = "Client"
  task method fn query self: Client -> sql: String -> Rows = "query"
  pure property fn clientName self: Client -> String = "name"
}
```

- constructorは `new ExportedClass(...args)` を呼ぶ。
- methodは第一引数をreceiverにし、hostの `receiver[name](...args)` を呼ぶ。
- propertyは追加引数を持たず、hostの `receiver[name]` を読む。
- `task` modeはthrow、Promise、getter effectをTaskへ閉じる。
- `pure` method/propertyは7.3のpure保証を満たす場合だけ使える。

opaque foreign objectのidentityはhost object identityです。ただしSeseragiにidentity比較は
公開しません。同じobjectをmethodへ渡すことはできます。

## 7.11 callback

同期pure callbackだけは自動adapterできます。

```seseragi
pure fn mapValues callback: (String -> String) -> values: Array<String>
  -> Array<String>
```

hostがcallbackを保持する、複数回呼ぶ、非同期に呼ぶ、またはcallbackからTaskを実行する場合は
`Js.Callback` とresource APIを使う手書きadapterが必要です。callback lifetime、解除操作、
cancellationをforeign signatureから推測しません。

## 7.12 TypeScriptからSeseragiを呼ぶ

公開Seseragi宣言にはTypeScript向けwrapperと `.d.ts` を生成できます。TypeScript ABIは
次のとおりです。

- 公開関数はuncurried functionとして公開する。
- `Int` は `bigint`、`Float` は `number`。
- recordとtupleはreadonly TypeScript型。
- nominal structはreadonly objectと非公開 `unique symbol` brand。
- ADTは `tag` を持つdiscriminated union。
- `Maybe` と `Either` もADTとして公開し、nullやthrowへ暗黙変換しない。
- `Task<E, A>` は `Promise<Either<E, A>>` として公開する。
- opaque型はconstructorとfieldを公開せず、brandされたopaque型として出力する。

```seseragi
pub fn add x: Int -> y: Int -> Int = x + y
```

は概念上、次のdeclarationを生成します。

```typescript
export declare function add(x: bigint, y: bigint): bigint;
```

wrapperはTypeScript引数を境界検査してからSeseragi関数を呼びます。検査不能な値を
unchecked castしません。

environment requirementが残る `Effect<R, E, A>` はTypeScriptへ直接exportできません。
Seseragi側の公開wrapperでenvironmentをprovideし、Taskへ変換してからexportします。

## 7.13 ABI安定性

生成物内部のclosure、dictionary、ADT layoutはbackend実装詳細です。外部から利用できるのは
生成wrapperと `.d.ts` だけです。同じpublic signatureを保つ限り、内部表現を変更できます。

public signatureの型、parameter順、opaque性、ADT variantを変えることはTypeScript ABIの
breaking changeです。

## 7.14 generic public ABI

rank-1でconstraintを持たないpublic generic functionは、型parameterを同じ順のTypeScript genericとして
exportできます。

```seseragi
pub fn firstOr<A> fallback: A -> values: Array<A> -> A = ...
```

```typescript
export declare function firstOr<A>(fallback: A, values: readonly A[]): A;
```

ABI wrapperはAそのものをopaque host valueとしてpass-throughし、Array、tuple、record、Maybe、Eitherなど既知の
outer shapeだけを検査・copyします。TypeScript type argumentはruntimeに存在しないため、Aの値をinstanceof、tag、
typeofで検査したことにしません。Seseragiのparametricityにより、unconstrained functionはAを型固有に観測できず、
保持、並べ替え、返却、parametric callbackへの受け渡しだけができます。

次はdefaultでTypeScript export errorです。

- trait constraintを持つgeneric function
- higher-kinded、rank-2以上、existentialな型parameter
- Aのruntime codec、foreign opaque layout、mutable host identityを要求するsignature
- public型へ現れない型parameter、またはTypeScriptが推論できない配置だけに現れる型parameter

traitをTypeScript dictionary ABIへ公開する場合だけ、project設定でtraitごとのdictionary interfaceを明示します。
wrapperはdictionaryを先頭の明示parameterとして受け、Seseragiのcoherent impl探索とは別のforeign dictionaryへ
変換します。たとえばEq<A>は `{ readonly equals: (left: A, right: A) => boolean }` のような設定済みshapeにできます。
mappingのないconstraintを名前の似たTypeScript `extends` へ推測しません。dictionary parameterの追加・shape変更は
ABI breaking changeです。

設定は `seseragi.toml` のtool tableへtrait identityをexactに書きます。

```toml
[tool.typescript-abi.traits."std/prelude::Eq"]
name = "EqDictionary"

[tool.typescript-abi.traits."std/prelude::Eq".methods]
eq = "equals"
```

未知trait / method、足りないrequired method、生成後の名前衝突はErrorです。このtableとgenerator versionのdigestを
ABI metadataへ記録し、同じpublic Seseragi interfaceでも設定が変わればABI diff対象にします。

generic struct / ADTはすべてのfieldがexport可能で、type parameterが上のopaque leaf規則だけで使われる場合に
TypeScript generic readonly typeを生成できます。brandはtype constructorごとに一つで、type argumentごとに
新しいruntime brandを作りません。wrapperはouter tag / fieldを検査し、Aのleafは検査不能であることをgenerated
metadataへ記録します。

## 7.15 source mapとcross-language stack

TypeScript backendはECMAScript source map v3を生成し、user declaration・expression・patternのrangeを生成JSへ
対応づけます。`sourcesContent` を含め、source URIはpackage identityを基準にした
`seseragi://<package>/<module>` 形式とし、build machineのabsolute pathをartifactへ埋めません。foreign wrapper、
dictionary passing、do desugarなどcompiler生成frameも元source rangeまたはgenerated markerを持ちます。

runtime defectと未処理Js.Errorのstackはsource mapを適用してから表示します。既定表示はSeseragi user frameを
優先し、compiler-generated frameを畳みますが、foreign境界を消しません。TypeScript / JavaScript frameは
`[typescript]`、Seseragiから生成されたwrapperは `[interop]` と表示し、`--show-internals` で畳んだframeを展開
できます。host packageが有効なsource mapを提供する場合は元TypeScript sourceまでchainし、なければJS locationを
保持します。

task foreign callは同期throw / Promise rejectionを捕捉した時点のhost stackと、Taskを作ったSeseragi call siteを
causal chainとしてJs.Errorへ保持します。awaitやschedulerを越えても「どこで開始し、どこで失敗を観測したか」を
別frame groupで示します。stack text自体はABIではありませんが、frameのlanguage、package、module、source range、
generated flagを持つmachine-readable formをdiagnostic JSONへ出せます。

source mapが欠落・破損していても元errorを隠さず、unmapped frameとsource-map warningを出します。path traversalを
含むsource URI、artifact外のsourcesContent読込、network source map取得は行いません。
host stackにabsolute pathが含まれる場合、known package root内はpackage-relative URIへ変換し、それ以外はbasename
だけを既定表示します。machine path全体をdiagnostic artifactへ保存するには明示debug optionを要求します。
known package root内のhost source URIは `package://<package-name>/<relative-path>`、外部host packageは
`package://<host-package-name>/<package-relative-path>` です。package外でidentityを解決できないframeはuriをnullにし、
absolute pathやbasenameから偽のpackage identityを作りません。

`--diagnostic-format json` はcompile diagnosticだけでなく、未処理typed failureとdefectもstderrへ一行の
RuntimeDiagnostic JSONとして出します。schema 1はkeyを次の順で持ちます。

```text
schema: 1
kind: TypedFailure | Defect | Cancellation
phase: ModuleLoad | SynchronousThrow | PromiseRejection | null
message: user-facing summary
groups: causal orderのarray
  role: Initiated | Thrown | Observed
  frames: outermostからinnermostのarray
    language: seseragi | typescript | interop
    function: source上のnameまたはnull
    uri: package-relative / seseragi URIまたはnull
    range: { start, end } またはnull
    generated: Bool
```

rangeは0-based・end-exclusive UTF-8 byteです。Initiatedはforeign Taskを作ったSeseragi call、Thrownはhostの
throw / rejection origin、Observedはawait / bind後にfailureを受け取ったSeseragi位置です。同じ位置ならInitiatedと
Observedを一groupへ畳んでObservedにします。通常表示で畳むinterop frameもJSONでは保持し、generated=Trueにします。
host stackに位置がない場合はuri / rangeをnullにし、架空のsource位置を生成しません。object keyとframe順は上の順、
末尾newline一つ、ANSIなしです。RuntimeDiagnostic自身のserialization failureで元failureを置き換えてはなりません。
