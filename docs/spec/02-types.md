# 2. 型システム

## 2.1 基本性質

Seseragi は静的型付きです。すべての式は実行前に型を持ち、型検査に失敗した
プログラムは実行されません。

型推論は Hindley-Milner 型推論を基礎とし、次を加えます。

- nominal な ADT、struct、newtype
- immutable record の structural typing
- trait constraint
- `Never` からの底型 coercion

暗黙の `Any` はありません。型を確定できない場合は `Unknown` に逃がさず、注釈を
要求します。

## 2.2 組み込み型

| 型       | 意味                          |
| -------- | ----------------------------- |
| `Int`    | 符号付き 64 bit 整数          |
| `Float`  | IEEE 754 binary64             |
| `Bool`   | `True` または `False`         |
| `Char`   | Unicode scalar value 一個     |
| `String` | Unicode scalar value の不変列 |
| `Unit`   | 唯一の値 `()` を持つ型        |
| `Never`  | 値を一つも持たない型          |

`Never` は任意の型が要求される位置で使えます。`Unit` と `Never` は異なります。

## 2.3 型構築子

```text
Maybe<A>          欠如する可能性のある値
Either<E, A>      Eで失敗したかAで成功した同期的な結果
Effect<R, E, A>   environment Rを要求し、Eで失敗しうる遅延計算
Task<E, A>        Effect<{}, E, A>のalias
Array<A>          不変の連続列
List<A>           不変の連結リスト
Range<A>          Aの有限な順序付きrange。range literalはRange<Int>
Signal<A>         読み取り専用の時間変化する値
MutableSignal<A>  更新可能なsource Signal
(A, B, ...)       tuple
{ x: A, y: B }    record
A -> B            関数
```

型引数の区切りは `,` です。関数型は右結合なので `A -> B -> C` は
`A -> (B -> C)` です。

型引数は再帰的に任意の型を取れます。`Array<Array<Int>>`、`Array<Maybe<Int>>`、
`Map<String, Array<Maybe<Int>>>`はいずれも一つの型です。末尾の`>>` / `>>>`は型構文内では
複数の`>` delimiterであり、expressionのoperator tokenとして扱いません。各段階の型構築子kindと
引数個数を独立に検査します。

## 2.4 型注釈と推論

関数 parameter と公開宣言には型注釈が必要です。非公開 `let` の型は推論できます。
関数の戻り型は必須です。これは module API と再帰関数の意味を安定させるためです。

```seseragi
fn add x: Int -> y: Int -> Int = x + y

let count = 3
let label: String = "ready"
```

型 parameter は宣言名の直後に書きます。

```seseragi
fn identity<A> value: A -> A = value
```

## 2.5 多相性

Seseragi は let-polymorphism を持ちます。型parameterを明示した宣言と、`let` で束縛した
非再帰的な値は型schemeとして一般化され、利用するたびに新しい型へinstantiateされます。

```seseragi
let identity = \value -> value

let number = identity 42
let text = identity "hello"
```

`identity` の型schemeは `forall A. A -> A` です。`forall` は仕様上の表記であり、
source syntaxではありません。

Seseragiの値は不変で、外部状態を持つ計算は `Effect` の内側に閉じるため、通常の
`let` にvalue restrictionは設けません。再帰groupだけはgroup内でmonomorphicに検査し、
group全体の検査後に一般化します。

## 2.6 nominal 型と structural 型

ADT、struct、newtypeはnominalです。field構成や内部表現が同じでも別の型は互換ではありません。

record は structural です。record `R1` は、`R2` が要求する全 field を同じ型で持つとき
`R2` の subtype です。field は不変なので、この width subtyping は読み取りに対して
安全です。

```seseragi
fn nameOf value: { name: String } -> String = value.name

let user = { id: 1, name: "Aki" }
let name = nameOf user
```

record field の型に対する暗黙の depth subtyping はありません。

record fieldはrequiredまたはoptionalです。`{ id?: String }` はidが存在すればStringであるrecord型で、field access
`value.id` の型は `Maybe<String>` です。optional fieldは `{ id: Maybe<String> }` と同じ型ではありません。後者は
field自体が必ず存在するため、値がNothingでもoptional accessなら `Just Nothing` と区別できます。

record `R1` が `R2` のsubtypeである条件は次です。

- R2のrequired fieldはR1にもrequiredとして存在し、型が同じ。
- R2のoptional fieldはR1で不在、optional、requiredのいずれでもよく、存在する場合の型が同じ。
- R1だけが持つ追加fieldは無視する。

したがって `{ children: String }` と `{ id: String, children: String }` はどちらも
`{ id?: String, children: String }` のsubtypeです。optionalからrequiredへのsubtyping、field型のdepth
subtypingは行いません。record literalが持つfieldは通常requiredとして推論し、期待型にないoptional fieldを
Nothingで挿入したことにはしません。

### closed structural record

通常のrecord型はstructuralで、width subtypingにより余分なfieldを持つ値も利用できます。
`closed structural record` はこのsubtypingを無効にする型ではなく、host境界でrequirementの全fieldを
有限かつ静的に列挙できるrecord型を意味します。

closedであるためには、alias展開後に次を満たさなければなりません。

- 最上位がrecord型で、free type variableや未解決のrow remainderを持たない。
- field名とfield型がすべて確定し、同名fieldがない。
- Effect / Streamのhost requirementとして使うrecordにはoptional fieldがない。
- 再帰aliasや未解決constraintを通してfield集合が変化しない。

Seseragiはsource syntaxとしてrow variableを持ちませんが、generic aliasやinference途中の型を
entry pointへ残さないことをclosed requirementとして明示します。hostが実際に渡すrecordは
width subtypingにより余分なfieldを持って構いません。余分なfieldはmainのrequirementには含まれず、
mainから参照もできません。

### requirement merge

Effect / Streamをgenericに合成し、元のrequirementへserviceを加えるため、第一型引数の中だけで
`R & S` をrequirement mergeとして使えます。

```seseragi
alias Timed<R, E, A> =
  Effect<R & { clock: Clock }, E, A>
```

operandはrequired fieldだけを持つstructural record型、またはこの位置からrecord requirementと分かる
型parameterでなければなりません。optional field、nominal struct、通常値型はmergeできません。mergeは
両operandのfield和へcompile時に正規化し、同名fieldの型が同じなら一fieldにまとめ、違えば
`SES-E0001`です。`R & {}`はRと同じで、同じfield型に対して結合順序と括弧は結果を変えません。

`&`は一般intersection型を作らず、値constructor、field access、runtime tagを持ちません。Effect / Streamの
第一型引数以外で使うと`SES-T0501`です。genericなRはrow remainderではなく一個の完全なrecord型を表し、
entry pointではmerge正規化後のfield集合がclosedでなければなりません。

## 2.7 関数型と curry

複数 parameter の関数宣言は、単一 parameter 関数の入れ子です。

```seseragi
fn add x: Int -> y: Int -> Int = x + y
```

の型は `Int -> Int -> Int` で、`add 1` の型は `Int -> Int` です。
parameter は左から右へ bind されます。

parameterを一つも書かない宣言は、名前のないUnit parameterを一つ持ちます。

```seseragi
fn clock -> Instant = ...
```

この型は `Unit -> Instant` です。`Unit` は型、`()` はその唯一の値で、呼び出しは `clock ()`
です。parameter名を使わないこと以外は通常の一引数関数と同じです。

## 2.8 型の同一性と coercion

Seseragi に一般的な暗黙変換はありません。

- `Int` と `Float` は別の型。`toFloat` を明示する。
- `A` は自動で `Maybe<A>` や `Either<E, A>` にならない。
- struct は同じ field の record にならない。
- newtypeは内部表現の型へ暗黙にunwrapされず、内部表現から暗黙にwrapされない。
- `Js.Nullable<A>` は `Maybe<A>` にならない。境界関数を使う。

一般の型に許される暗黙coercionは、`Never` から任意型へのcoercionとrecordのwidth
subtypingだけです。EffectとSignalが持つcapability固有のcoercionは2.19と5章で明示します。

## 2.9 再帰

moduleまたはblockの名前付き関数とtype宣言は再帰できます。再帰する関数は戻り型を必ず注釈します。
通常の `let` は自身を参照できません。相互再帰関数は同じ `rec` group に置きます。
直接のself tail callは14.8に従い一定のhost call stackで実行します。相互再帰とnon-tail callには
同じ保証を適用しません。

```seseragi
rec {
  fn even n: Int -> Bool = if n == 0 then True else odd (n - 1)
  fn odd n: Int -> Bool = if n == 0 then False else even (n - 1)
}
```

block内の通常の`fn` / `effect fn`は、自身のbodyと宣言後のblock itemから参照できます。後続のlocal
functionはscopeに入っていないため、相互参照にはlocal `rec` groupが必要です。rec groupの全member名は
group全体とgroup後のblock itemでscopeに入り、group前では参照できません。group内では既存規則どおり
monomorphicに検査し、group全体の検査後に明示型parameterを一般化します。

## 2.10 kindと型構築子

通常の値が持つ型のkindは `Type` です。kindは次で構成します。

```text
Kind = Type | Kind -> Kind
```

型parameterは通常 `Type` kindです。`M<_>` と書いたparameterは `Type -> Type` kindの
型構築子を受け取ります。型構築子parameterは関数、ADT、struct、alias、trait、implで
宣言できます。

```seseragi
alias StateT<S, M<_>, A> = S -> M<(A, S)>
```

kindのhole `_` は型parameter宣言と型構築子の部分適用にだけ使え、値の型には残せません。
kindが一致しない適用はコンパイルエラーです。

型宣言は名前とarityを持つ型構築子を導入します。

```seseragi
type Maybe<A> =
  | Nothing
  | Just A

struct Pair<A, B> {
  first: A,
  second: B,
}
```

このとき `Maybe` のkindは `Type -> Type`、`Pair` のkindは
`Type -> Type -> Type`、`Maybe<Int>` と `Pair<Int, String>` のkindは `Type` です。

通常の型位置では、すべての型引数を与えなければなりません。`Maybe` や
`Pair<Int>` は値の型ではありません。型構築子kindが要求される引数では、`Maybe` および
`Either<E, _>` のような部分適用を許可します。

型引数の個数またはkindが宣言と一致しなければコンパイルエラーです。余分な型引数を
無視したり、不足した引数を `Unknown` で補ったりしません。

## 2.11 型parameterのscope

型parameterのscopeは、宣言の型parameter list直後から宣言末尾までです。内側の宣言で
同名の型parameterをshadowingできません。同じlist内で名前を重複できません。

```seseragi
fn map<A, B> f: (A -> B) -> values: List<A> -> List<B> = ...
```

`A` と `B` はparameter型、戻り型、constraint、関数本体内の明示型注釈で参照できます。
scope内では型parameterが同名のmodule型より優先されます。

型parameter名は大文字から始めます。宣言されていない大文字名を型parameterとして
推測しません。型名としても解決できなければ未定義型エラーです。

## 2.12 generic関数

明示的な型parameterを持つ関数は、そのparameterを量化した型schemeを導入します。

```seseragi
fn const<A, B> first: A -> second: B -> A = first
```

型schemeは `forall A B. A -> B -> A` です。call siteでは次の順にinstantiateします。

1. 各型parameterをfreshな推論変数へ置換する。
2. 値引数の型から制約を集める。
3. 呼び出し式に期待型があれば、戻り型からも制約を集める。
4. trait constraintを解決する。
5. 未解決の型変数が残れば、曖昧な呼び出しとしてエラーにする。

通常は型引数を省略します。必要なときだけ関数名の直後に明示します。

```seseragi
let inferred = identity 42
let explicit = identity<String> "hello"
```

明示型引数は宣言順に対応し、全個数を指定します。部分指定とnamed type argumentは
ありません。値引数または期待型と矛盾する明示型引数はコンパイルエラーです。

型parameterが戻り型にしか現れない関数も宣言できますが、期待型または明示型引数が
なければ呼び出せません。

```seseragi
fn empty<A> -> List<A> = `[]

let names: List<String> = empty ()
let numbers = empty<Int> ()
```

## 2.13 let-polymorphismとrank

非再帰的な `let` の右辺を推論した後、現在の型環境に現れないfree type variableを
すべて量化します。型schemeは識別子を参照するたびにfreshにinstantiateします。

右辺からtrait constraintが生じた場合、量化した型variableに依存するconstraintも型schemeへ
含めます。例えば `\x -> x + x` は概念上
`forall A. Add<A, A, A> => A -> A` です。環境だけに依存するconstraintは一般化しません。

多相性はrank-1です。関数parameter、record field、Array要素などへ多相な値そのものを
渡すhigher-rank polymorphismはありません。parameter型 `(A -> A)` の `A` は外側の
宣言で束縛された一つの型であり、呼び出すたびに変わる型schemeではありません。

## 2.14 generic ADT

generic ADTの各constructorは、ADTの全型parameterを量化した関数または値を導入します。

```seseragi
type Result<E, A> =
  | Failure E
  | Success A
```

導入される型schemeは次のとおりです。

```text
Failure : forall E A. E -> Result<E, A>
Success : forall E A. A -> Result<E, A>
```

constructorを使うたびに全parameterをfreshにinstantiateします。payloadから決まらない
parameterは、期待型、matchのcontext、または明示型引数で決めます。

```seseragi
let failed: Result<String, Int> = Failure "not found"
let empty = Failure<String, Int> "not found"
```

pattern内のconstructorはmatch対象の型引数をfieldへ代入します。
`Success value` を `Result<String, Int>` にmatchした場合、`value` は `Int` です。

再帰ADTでは、再帰位置に型引数を明示します。

```seseragi
type Tree<A> =
  | Leaf A
  | Branch (Tree<A>, Tree<A>)
```

## 2.15 generic struct

generic structの生成時はfieldから型引数を推論します。

```seseragi
struct Box<A> {
  value: A,
}

let inferred = Box { value: 42 }
let explicit = Box<String> { value: "hello" }
```

`inferred` は `Box<Int>`、`explicit` は `Box<String>` です。複数fieldから同じparameterに
異なる型制約が集まった場合はエラーです。field accessではreceiverの型引数を宣言fieldへ
代入するため、`Box<Int>.value` は `Int` です。

constructionの明示型引数はfieldと期待型による推論より優先し、宣言parameterと個数が
一致しなければなりません。nested型引数も通常の型構文として解決します。明示引数に
未解決の型holeが残る場合はbackendへ渡さず型診断で停止します。

struct updateとspreadでは元のstructと完全に同じ型引数を要求します。更新によって
`Box<Int>` を `Box<String>` に変えることはできません。

## 2.16 generic implとmethod

型parameterを含む型へmethodを定義するときは、`impl` 自身でparameterを束縛します。

```seseragi
impl<A> Box<A> {
  fn get self: Box<A> -> A = self.value

  fn map<B> self: Box<A> -> f: (A -> B) -> Box<B> =
    Box { value: f self.value }
}
```

`impl` parameterは対象型に現れなければなりません。method独自のparameterはmethod名の
直後で追加できます。receiver型から `impl` parameterを決定した後、method parameterを
通常のgeneric関数と同じ規則でinstantiateします。

## 2.17 generic alias

```seseragi
pub alias Named<A> = { name: String, value: A }
```

aliasは型へ別名を付けますが、新しいnominal型を作りません。`Named<Int>` は
`{ name: String, value: Int }` と同じ型です。

alias利用時は型引数の個数を検査し、宣言parameterを引数でcapture-avoiding substitution
してから型検査します。直接・間接を問わず、aliasの循環はコンパイルエラーです。
値を区別したい場合はnewtype、struct、ADTのいずれかを使います。

## 2.18 newtype

newtypeは値を一つだけ包むnominal型です。型名と同名のconstructorを自動で導入します。

```seseragi
newtype UserId deriving Eq, Ord, Hash, Show = Int
newtype Tagged<A> deriving Eq, Show = A
```

型schemeは次のとおりです。

```text
UserId : Int -> UserId
Tagged : forall A. A -> Tagged<A>
```

constructor patternは包んだ値を取り出します。newtypeはconstructorが一つだけなので、対象型が
一致するconstructor patternはirrefutableです。

```seseragi
fn raw id: UserId -> Int =
  match id {
    UserId value -> value
  }
```

newtypeと内部表現の間に暗黙coercionはありません。wrapにはconstructor、unwrapにはpatternを
使います。汎用の `coerce`、自動生成field、runtime type reflectionは提供しません。

newtypeは直接・alias経由・別newtype経由のいずれでも再帰できず、右辺はkind `Type` の型一つで
なければなりません。型parameterは右辺に現れなくても構いませんが、通常のinferenceで
決まらない利用箇所では明示型引数が必要です。

backendは、constructor適用、pattern match、trait instance、module visibilityから観測できる意味を
保つ限り、newtypeのruntime wrapperを消去できます。型検査上は常に別のnominal型です。

## 2.19 variance

すべてのuser-defined型構築子と標準collectionはinvariantです。`A` が `B` のsubtypeでも、
`Box<A>` は `Box<B>` のsubtypeではありません。variance annotationと自動variance推論は
ありません。

この規則はrecordのwidth subtypingと独立です。`SmallRecord <: LargeRecord` が成立しても、
`Array<SmallRecord> <: Array<LargeRecord>` は成立しません。必要な変換は `map` で明示します。

言語組み込みのEffect / Stream requirement widening、`Never` failure wideningと、
`MutableSignal<A>` から `Signal<A>` への読み取り権限のforgetful coercionだけは、各章で定義する
例外です。success型 `A` と、それ以外のerror型の組み合わせはinvariantです。

## 2.20 型消去と実行時表現

型parameterと型引数は実行時の値ではありません。`A` によるruntime分岐、型引数の
reflection、型ごとの暗黙specializationはありません。異なる型引数を持つ同じ型構築子の
値は、型検査では区別されますが、backendは観測可能な意味を変えない範囲で型消去または
monomorphizationを選べます。
