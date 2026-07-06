# 6. モジュール

## 6.1 moduleの単位とidentity

一つの `.ssrg` fileが一つのmoduleです。通常moduleのidentityはpackage identityと、packageの
source rootからfileまでの正規化された相対pathの組です。source内にmodule名を書きません。

```text
<package identity>::<relative module path>
```

path separatorは意味上 `/` に正規化し、拡張子 `.ssrg` はmodule pathに含めません。
同じfileを異なる相対pathやsymlinkから二重に読み込んでも、一つのcanonical pathへ
解決しなければなりません。

generated rootとtest rootのmoduleは通常sourceとidentityを共有せず、11.3で定めるroot tagを
含みます。

## 6.2 package

packageは、package identity、source root、直接dependency、公開entry pointを
[`seseragi.toml`](./11-packages-and-projects.md)で宣言します。manifest、標準layout、export map、
lockfileの具体的な契約は11章に従います。

- package identityはdependency graph内で一意。
- source、test、generatedの各rootを横断する相対importは禁止。
- dependencyはmanifestに宣言した直接dependencyだけimportできる。
- 同じpackage名の異なるversionは内部identityで区別する。

## 6.3 top-level

top-levelには次だけを書けます。

- importとre-export
- `let`
- `fn`
- `effect fn`
- `type`
- `newtype`
- `alias`
- `struct`
- inherent `impl`
- trait `instance`
- `trait`
- custom `operator`
- `foreign` declaration
- `rec` group

任意の式statementはtop-levelに書けません。top-level `let` の右辺は純粋な式でなければ
ならず、Effectを作ることはできますが実行できません。

## 6.4 visibility

宣言は既定でmodule-privateです。`pub` を付けた `let`、`fn`、`effect fn`、`type`、`newtype`、
`alias`、`struct`、`trait`、custom `operator`、`foreign` blockだけを他moduleから参照できます。

```seseragi
pub fn findUser id: UserId -> Task<FindError, Maybe<User>> = ...
```

公開 `let` と公開関数は完全な型注釈を必要とします。公開型の署名からprivate型を参照
できません。

公開ADTと公開newtypeのconstructor、公開structのfieldは、既定で型と一緒に公開されます。
表現を隠したい場合は `opaque` を付けます。

```seseragi
pub opaque struct UserId {
  value: Int,
}

pub opaque type Token =
  | Token String

pub opaque newtype Email = String
```

opaque型は他moduleから型名だけを参照できます。constructor、field、destructuringは
定義module内だけで使えます。公開された通常関数とmethodを通して操作します。

非opaqueな公開newtypeの右辺型は公開APIに現れるため、private型を参照できません。opaque
newtypeの右辺型はmodule外のsignatureへ現れないため、private型を使えます。

inherent methodも既定でprivateです。他moduleから呼べるmethodには `pub fn` を付けます。
trait instance自体へvisibility modifierは付けません。instanceを宣言したmoduleが現在moduleの
transitive import closureに含まれる場合、instance searchの対象になります。instanceだけを
選択的にimport・非表示にはできません。

## 6.5 import

```seseragi
import { User, findUser } from "./user"
import { parse as parseJson } from "json"
import { operator <+> } from "./semigroup"
import * as text from "std/text"
```

importはmodule先頭に置きます。同じscopeへ同名を二度導入できません。未使用importは
warningであり、意味には影響しません。

importしたmoduleのinstanceが型検査で選択された場合、そのimportは使用済みです。namespace aliasやnamed valueが
sourceに現れなくてもunused warningを出さず、organize-importsで削除してはなりません。instanceを一件も選択せず、
名前・operator・re-exportにも寄与しないimportだけがunusedです。

operator importはsymbolだけでなくfixity、precedence、型schemeをmodule interfaceから取得します。
language serverを含むconsumerはimport先のfunction bodyをparseまたは実行せず、公開interfaceだけで
operator chainを解決できます。

import formの意味は次のとおりです。

- named importは指定した公開名だけをlocal scopeへ導入する。
- alias importはlocal名だけを変更する。
- namespace importは `text.trim` のようなqualified accessだけを許可する。
- wildcardで全名をunqualifiedに導入する構文はない。

namespace aliasは値・関数だけでなく、型、trait、constructor、入れ子の公開namespaceを修飾できます。
foreign block内の`namespace`も同じmodule alias namespaceへ入り、outsideからは通常のnamespace importと同じ
`metrics.count`形式で参照します。host objectのfield accessではなく、7.2で検証されたforeign namespaceです。

```seseragi
import * as users from "acme/users"

fn find id: users.UserId -> Maybe<users.User> = ...
let guest = users.Guest
```

`users.User` は型位置、`users.Guest` は式またはpattern位置のconstructorとして解決します。
custom operatorはqualified infix syntaxを持たないため、従来どおり `import { operator <+> }` で
明示importします。任意の値に対するfield/method accessとnamespace accessは名前解決時に区別し、
runtime objectを調べません。

同じspellingが型namespaceと値namespaceの両方で公開されている場合、named import一つで両方を
導入します。aliasも両方へ同じlocal spellingを与えます。この規則により、非opaqueなnewtypeは
`import { UserId }` だけで型とconstructorを利用できます。参照位置のnamespaceで区別するため、
曖昧にはなりません。

## 6.6 module specifierの解決

specifierは次の順ではなく、先頭形式で分類します。

1. `./` または `../`: 現在moduleからの相対path。
2. `self/`: 現在packageのsource module。
3. `std/`: 標準ライブラリmodule。
4. `gen/`: 現在packageのgenerated binding module。
5. その他のbare specifier: manifestに宣言されたpackageと、その公開subpath。

相対specifier `./user` は正確に `<current-dir>/user.ssrg` へ解決します。directory indexの
暗黙探索、拡張子候補の総当たり、current working directoryへのfallbackはしません。
specifierに `.ssrg` を書いた場合も同じcanonical moduleへ正規化します。

package specifier `acme/http/client` はdependency import root `acme/http` が公開したsubpath
`client` へ解決します。package内部の非公開pathへ直接入れません。dependency alias、package-private
`self/` import、generated rootの詳細は11章に従います。

解決候補が0件または複数件ならコンパイルエラーです。

## 6.7 re-export

```seseragi
pub import { User, findUser } from "./user"
pub import * as validation from "./validation"
```

re-exportはimport先の公開名を現在moduleの公開APIへ加えます。named re-exportは元のidentityを
保ち、別の宣言を複製しません。namespace re-exportは公開namespaceを一つ導入します。

re-export graphも通常のimport graphに含まれます。

## 6.8 namespaceと名前解決

次のnamespaceは分かれています。

- 型・trait・型parameter
- 値・関数・constructor
- field・inherent method
- module alias
- custom operator

同じnamespace内で同じ名前を重複定義できません。ADTとnewtypeのconstructorは値namespaceに
入り、型parameterは宣言scope内の型namespaceへ入ります。

unqualified名はlocal bindingとlocal function、現在moduleの宣言、明示import、preludeの順に検索します。
同じ段階に複数候補があれば曖昧エラーです。後から追加したimportが、既存の曖昧な参照を
勝手に別候補へ変えることはありません。

## 6.9 dependency graphと循環

値、型、traitを問わず、module import graphの循環は禁止します。type-only cycleを特別扱い
しません。compilerはcycleを構成するmodule pathをすべて診断へ出します。

同一module内の相互再帰は `rec` groupで表します。module間相互再帰が必要な設計は、共有型を
第三moduleへ分離します。

## 6.10 初期化と評価

moduleはdependencyのtopological orderで一度だけ初期化します。同一module内のtop-level
`let` はsource順に評価します。関数、type、newtype、alias、struct、traitは評価前から
名前解決できます。

module-level function名が先に名前解決できても、関数body間の循環は2.9の同じ`rec` groupに属する場合だけ
許可します。block内のlocal functionは3.2の宣言位置から始まるscopeを使い、module-levelのhoist規則を
適用しません。

top-level `let` は、それより前の `let` とすべての関数宣言だけを参照できます。後続の
`let` は参照できません。effectはEffect内に閉じるため、module importだけでI/Oは発生しません。

同じmoduleを複数箇所からimportしてもtop-level値は一度だけ作られます。

foreign moduleのruntime codeはparser、type checker、formatter、language server、document generatorが評価しては
なりません。runtime buildでpure-loadと判定されたhost moduleだけは、7.2の保証に基づきSeseragi module初期化中に
一度評価できます。task-load moduleはSeseragi moduleをimportしただけでは評価せず、最初のforeign Task実行まで
遅延します。

## 6.11 entry point

実行可能packageは、manifestで指定したmoduleから公開 `main` を一つ解決します。

```seseragi
pub effect fn main -> Unit
with AppEnv
fails AppError = ...
```

これは匿名Unit parameterを持つ `Unit -> Effect<AppEnv, AppError, Unit>` 型の関数です。
manifestのhost targetはentry pointへ `()` を渡し、返されたEffectへenvironmentを供給して
実行します。`AppEnv` はclosedなstructural recordでなければなりません。hostが提供しない
application固有serviceは、mainが返すEffectを構成する時点で `provide` しなければなりません。
closedの判定は2.6に従い、generic main、free type variable、field集合が未確定なrequirementを
entry pointにできません。host environmentの追加fieldは許されます。

mainのfailure型もconcreteでなければなりません。failure型が `Never` でない場合は、その型に対する
coherentな `Show` instanceをentry moduleのtransitive import closureから一意に解決できなければ
build errorです。hostはtyped failureだけを `show` で表示し、defectやcancellationをapplication errorへ
偽装しません。

mainのsuccess型はUnitだけです。application codeがprocessを直接終了する `exit` primitiveや、任意の
exit codeを返すentry point overloadは提供しません。終了statusはmainの終了理由とprocess signalから
10.14のhost規則で決めます。これによりfinalizerを飛ばす通常codeを作れないようにします。

library packageはentry pointを持たなくて構いません。importされたmoduleの `main` を暗黙実行
しません。
