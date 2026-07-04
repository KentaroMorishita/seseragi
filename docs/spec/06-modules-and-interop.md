# 6. モジュールと外部連携

## 6.1 module

一つの `.ssrg` ファイルが一つの module です。module 名は source root からの相対 path
で決まります。ファイル内で module 名を宣言しません。

top-level には次だけを書けます。

- import
- `let`
- `fn`
- `type`
- `alias`
- `struct`
- inherent `impl`
- trait `impl`
- `trait`
- `foreign` declaration
- `rec` group

## 6.2 visibility

宣言は既定で module-private です。`pub` を付けた `let`、`fn`、`type`、`alias`、
`struct`、`trait` だけを他 module から import できます。`impl` の visibility は対象の
型または trait に従い、`pub impl` とは書きません。

公開関数と公開 let は完全な型注釈を必要とします。公開 ADT の constructor と公開
struct の field は公開されます。

## 6.3 import

```seseragi
import { User, findUser } from "./user"
import { parse as parseJson } from "json"
import * as text from "std/text"
```

import は module の先頭に置きます。同じ scope に同名を二度導入できません。import は
値、型、trait の各 namespace を同時に参照し、曖昧な場合はエラーになります。

相対 path は `./` または `../` で始めます。拡張子は省略し、`.ssrg` を補います。
`std/` は標準ライブラリ、その他の bare specifier は package resolver に渡します。

循環 import はコンパイルエラーです。

## 6.4 namespace

次の namespace は分かれています。

- 型・trait
- 値・関数・constructor
- field・inherent method

同じ namespace 内で同じ名前を重複定義できません。ADT constructor は値 namespace に
入ります。

## 6.5 外部連携の原則

外部連携は Seseragi の型意味論を外部言語へ合わせる機能ではありません。外部値を
安全な Seseragi 値へ変換する、型付きの境界です。

外部 declaration は backend 名を明示します。

```seseragi
foreign "js" from "user-api" {
  fn fetchUser id: String -> Js.Promise<Js.Unknown>
}
```

`foreign` 内の型は backend 固有の境界型を使えます。外部関数の arity、calling
convention、例外、Promise などは backend adapter が扱います。Seseragi 側からは宣言に
書かれたカリー化関数として見えます。

## 6.6 外部型

JavaScript backend は少なくとも次の opaque boundary type を持ちます。

```text
Js.Unknown
Js.Nullable<A>
Js.Promise<A>
Js.Error
```

これらは通常の Seseragi 型へ暗黙変換できません。

- `Js.Unknown` は decoder で検証して `Either<DecodeError, A>` にする。
- `Js.Nullable<A>` は `Maybe<A>` へ明示変換する。
- `Js.Promise<A>` は rejection と同期 throw を捕捉して `Task<Js.Error, A>` にする。
- `Js.Error` は application error へ明示的に写像できる。

`.d.ts` や外部型定義は foreign declaration の生成補助に使えますが、それ自体が
Seseragi の型ではありません。外部の conditional type、mapped type、overload などを
Seseragi の型システムへ持ち込みません。

## 6.7 pure foreign

foreign 関数は既定で effectful boundary value です。同期的な関数でも throw しうる場合は
`Either<Js.Error, A>`、外部状態へ触れる場合は `Task<Js.Error, A>` の wrapper を通します。

binding author は、決定的・非 throw・外部状態を観測しないことを保証できる関数だけを
`foreign pure` と宣言できます。この保証違反は binding の defect です。

## 6.8 export と外部からの呼び出し

外部ランタイムから Seseragi の公開関数を呼ぶ adapter は、curried function、ADT、
Task をそのまま外部 ABI と見なしません。backend ごとの安定した wrapper を生成し、
境界変換を行います。ABI の表現は言語上観測できません。
