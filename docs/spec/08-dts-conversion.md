# 8. `.d.ts` からのbinding生成

## 8.1 目的

`.d.ts` converterは、TypeScript declarationをSeseragiの型そのものへ翻訳するものでは
ありません。TypeScript moduleを呼ぶためのforeign binding候補と、変換できなかった理由を
生成するtooling contractです。

変換の原則は次のとおりです。

1. 意味を保てる型だけを自動変換する。
2. 表現できない型を `Any` や通常のSeseragi型へ黙って弱めない。
3. TypeScript固有の値は `Js.*` boundary typeまたはforeign opaque typeに残す。
4. purity、throw、side effectを `.d.ts` から推測しない。
5. 生成結果と未変換項目を機械可読reportへ必ず出す。

## 8.2 入力とsymbol解決

converterは、entry `.d.ts`、compiler options、package resolverを入力としてTypeScriptの
programとsymbol tableを構築します。import、re-export、declaration merging、module
augmentationをTypeScriptの名前解決後のsymbolとして読みます。

生成対象はentry moduleがexportするsymbolだけです。transitive dependencyの内部symbolは、
公開signatureから参照される場合だけopaque foreign typeとして取り込みます。

ambient global declarationは既定で無視します。global bindingを生成するには対象symbolを
明示選択しなければなりません。

## 8.3 出力

一回の変換は次を生成します。

- `.ssrg` foreign binding module
- 元symbol、元source location、選択した変換ruleを記録するmetadata
- warningとerrorを含むconversion report
- 手作業が必要なoverload、unsupported type、effect classificationの一覧

生成fileは入力 `.d.ts` と設定が同じなら決定的でなければなりません。symbolはexport名、
source module、signature順で安定sortします。

生成fileを直接編集する運用は前提にしません。手書きadapterは別の `.ssrg` moduleに置き、
生成bindingをimportします。

生成binding blockは `pub foreign` とし、raw boundary型を利用する手書きadapterからimport
できるようにします。applicationの公開APIはraw bindingをre-exportせず、adapterを公開します。

## 8.4 primitive typeの変換

| TypeScript  | 生成するSeseragi型 | 備考                      |
| ----------- | ------------------ | ------------------------- |
| `boolean`   | `Bool`             |                           |
| `string`    | `String`           | 境界でUnicode妥当性を検査 |
| `number`    | `Float`            | NaNとinfinityを含む       |
| `bigint`    | `Int`              | signed 64 bit範囲を検査   |
| `never`     | `Never`            |                           |
| `unknown`   | `Js.Unknown`       | decoder必須               |
| `object`    | `Js.Object`        | field access不可          |
| `null`      | `Js.Null`          | boundary限定              |
| `undefined` | `Js.Undefined`     | boundary限定              |

`void` は関数の戻り値なら `Unit`、それ以外では `Js.Undefined` にします。`any` は既定で
変換errorです。明示的なunsafe optionを有効にした場合だけ `Js.Unknown` へ変換し、symbolごとに
warningを残します。

## 8.5 nullabilityとoptional

nullish unionだけは特別に変換します。

```text
T | null                -> Js.NullOr<T>
T | undefined           -> Js.UndefinedOr<T>
T | null | undefined    -> Js.UndefinedOr<Js.NullOr<T>>
```

optional parameterとoptional propertyは `Js.UndefinedOr<T>` です。通常のSeseragi APIへ公開する
手書きadapterで `Maybe<T>` へ変換します。converterが `Maybe` へ直接変換しないのは、
`null` と「propertyが存在しない」をraw boundaryで区別できるようにするためです。

## 8.6 collection、tuple、object

```text
readonly T[] / ReadonlyArray<T> -> Array<T>
readonly [A, B]                 -> (A, B)
T[] / Array<T>                  -> Js.MutableArray<T>
```

mutable arrayを不変Arrayとして参照共有しません。`Js.MutableArray<T>` からsnapshotを取る
明示関数は `Task<Js.Error, Array<T>>` を返します。

readonly object typeとreadonly interfaceは、すべてのpropertyが変換可能でcall/construct/index
signatureを持たない場合、設定によりSeseragi recordへsnapshot変換できます。既定では
foreign opaque typeとして生成します。

mutable property、setter、index signatureを持つobjectはopaqueです。

## 8.7 function

TypeScript function signatureは、値parameter順を保ったSeseragiのcurried foreign function
候補へ変換します。

```typescript
export declare function format(value: number, digits: number): string;
```

```seseragi
pub foreign "typescript" from "number-format" {
  task fn format value: Float -> digits: Float -> String
}
```

通常のTypeScript functionは既定で `task fn` にします。`.d.ts` はpurity、同期throw、I/Oを
表現しないためです。userがsymbol単位でpure保証を承認した場合だけ `pure fn` を生成します。

戻り型が `Promise<T>` または `PromiseLike<T>` なら必ずtask modeとし、success型を `T` に
します。同期throwとrejectionはどちらも `Js.Error` です。

function型のparameterは既定で `Js.Callback<Args, Result>` にします。`.d.ts` だけではcallbackを
同期的に一度だけ呼ぶのか、保持・再入・非同期呼び出しするのか判別できないためです。
userがcallback contractを承認したsymbolだけ、通常のSeseragi関数型へ変換できます。

## 8.8 overload

overload setを一つのunion関数へ変換しません。各signatureを独立候補としてreportへ出し、
user設定でlocal名と採用signatureを選びます。

```text
parse(text: string): Node
parse(bytes: Uint8Array): Node
```

は例えば `parseText` と `parseBytes` へbindできます。選択がないoverload setはbindingを
生成せずerrorにします。implementation signatureが `.d.ts` に存在しても公開overloadで
なければ候補にしません。

## 8.9 generic declaration

TypeScriptのfirst-order generic function、type alias、interfaceは、すべての型parameterが
`Type` kindとして扱え、bodyとconstraintが対応subset内ならSeseragiの型parameterへ変換します。

```typescript
export declare function first<T>(values: readonly T[]): T | undefined;
```

```seseragi
task fn first<A> values: Array<A> -> Js.UndefinedOr<A>
```

parameter順、arity、出現位置を保ちます。default type argumentは自動適用せず、defaultを
展開したbindingか全引数を持つbindingのどちらを生成したかmetadataへ記録します。

`extends` constraintは、対応するSeseragi traitへ明示mappingが設定されている場合だけ
`where` constraintへ変換します。構造的constraintを名前が似たtraitへ推測変換しません。

## 8.10 classとenum

classはforeign opaque typeを生成し、公開constructor、method、propertyを7.10のbinding候補へ
変換します。private/protected memberは生成しません。generic classはfirst-order型parameterを
持つopaque typeにできます。

TypeScript enumはforeign opaque typeとして生成し、各memberをforeign valueとしてbindします。
数値enumをInt、文字列enumをStringへ弱めません。

## 8.11 unionとintersection

自動変換するunionはnullish unionだけです。それ以外は次の扱いです。

- string/number literal union: opaque foreign typeと、存在するexported constantを生成する。
- discriminated object union: opt-in時だけdecoder付きSeseragi ADT adapterを生成する。
- その他のunion: `Js.Unknown` への明示fallbackを要求し、既定ではerror。

intersectionは、readonly object intersectionを正確にfield mergeでき、同名field型が一致する
場合だけrecord snapshot候補へ変換できます。それ以外はopaqueまたはerrorです。

union/intersectionのmemberを捨てたり、最初のmemberを採用したりしません。

## 8.12 対応しないTypeScript型

次は自動変換しません。

- conditional type
- mapped type
- `infer`
- `keyof` とindexed access type
- template literal type
- recursive conditional type
- polymorphic `this`
- variadic tuple type
- higher-rank generic callback
- unique symbolを使う公開演算
- call signatureとmutable propertyが混在するobject

converterはunsupported型を含む最小source span、symbol path、型の表示、fallback候補を診断へ
出します。`Js.Unknown` fallbackは設定でsymbolごとに承認しなければ生成しません。

## 8.13 discriminated unionのopt-in変換

すべてのmemberが同じliteral discriminator fieldを持ち、残りが変換可能なreadonly fieldなら、
converterはSeseragi ADTとruntime decoder/encoderを生成できます。

```typescript
type Event =
  | { readonly type: "created"; readonly id: string }
  | { readonly type: "deleted"; readonly id: string };
```

概念上、次へ変換します。

```seseragi
type Event =
  | Created { id: String }
  | Deleted { id: String }
```

これはzero-cost type aliasではなく境界変換です。decoderは未知のdiscriminator、欠落field、
不正field型を `DecodeError` として返します。

## 8.14 declarationの更新

再生成時はsymbol metadataで対応を追跡します。

- 互換な追加は生成bindingを更新する。
- export削除、parameter変更、型変更はbreaking diagnostic。
- 手書きadapterが参照する生成名を黙ってrenameしない。
- unsupportedへ変化したsymbolは古いbindingを残さず、生成失敗として扱う。

conversion reportは前回との差分を `added`、`changed`、`removed`、`unsupported` へ分類します。
