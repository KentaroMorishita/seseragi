# 11. packageとproject layout

## 11.1 manifest

Seseragi packageのrootには `seseragi.toml` を一つ置きます。別名、`package.json` からの推測、
親directoryにある複数manifestのmergeは行いません。

最小のlibrary packageは次の形です。

```toml
[package]
name = "acme/validation"
version = "1.2.0"
language = ">=0.1.0 <0.2.0"

[exports]
"." = "lib"
"email" = "email"
```

実行可能packageは `run` を加えます。

```toml
[run]
entry = "main"
target = "node"
signal_mode = "cancel"
shutdown_grace_ms = 10000
hash_seed = "random"
```

`signal_mode` は `cancel` または `forward` で、省略時は `cancel` です。`cancel` はtermination signalを
root Effectのcancellationへ変換し、`forward` は `std/process.signals` へ渡します。
`shutdown_grace_ms` は0以上の整数で、省略時は10000です。cancel modeでだけ指定でき、0はcancellationを
要求した直後にforced terminationへ移る設定です。

grace periodはhostのmonotonic clockで測り、application environmentのClock serviceを要求・参照しません。
test hostはsignal入力とmonotonic timeを決定的に制御できなければなりません。

`hash_seed` は `"random"` またはsigned 64-bit integerで、省略時は `"random"` です。random modeは
hostのsecure entropyを要求します。integerは同じHash implementationで内部bucket配置を再現するための
test・benchmark用設定で、production packageでの使用はtool warningの対象です。seedはMap / Setの反復順や
serialized dataを変えないため、通常testは固定seedへ依存してはなりません。

signalを持たないtargetで `forward` を選ぶこと、またはsignal / graceful shutdown capabilityを持たない
targetへこれらのkeyを明示することはmanifest errorです。target adapterは対応可否をbuild時に公開し、
runtimeで黙って無視しません。

manifestはUTF-8のTOML 1.0です。未知のcore key、同じkeyの重複、型の違う値はerrorにします。
tool固有設定だけは `[tool.<tool-name>]` 以下に置けます。compilerは未知のtool tableを保持して
構いませんが、言語semanticsへ影響させません。

core top-level tableは `package`、`layout`、`exports`、`dependencies`、`foreign`、`run`、`tool`
です。`package.name`、`package.version`、`package.language` は必須で、それ以外は省略できます。

## 11.2 package identityとversion

`package.name` はASCII lowercaseのsegmentを `/` でつないだ名前です。各segmentは
`[a-z][a-z0-9-]*` に一致し、空segment、`.`、`..`、先頭・末尾の `-` を許しません。
先頭segmentの `std`、`self`、`gen` は予約済みで、package名とdependency keyに使えません。

`package.version` はSemVer 2.0.0のversion、`package.language` はこの言語versionに対する
SemVer rangeです。compilerのlanguage versionがrangeを満たさなければ、sourceを型検査する前に
errorにします。

rangeはexact version、`>`、`>=`、`<`、`<=`、`^`、`~` comparatorを持ち、空白はintersection、
`||` はunionです。wildcardとhyphen rangeは使いません。caretとtildeの上限は次で固定します。

```text
^1.2.3   = >=1.2.3 <2.0.0
^0.2.3   = >=0.2.3 <0.3.0
^0.0.3   = >=0.0.3 <0.0.4
~1.2.3   = >=1.2.3 <1.3.0
```

prerelease versionは、同じmajor/minor/patchのprereleaseをrange内で明示した場合だけ候補にします。

dependency graph内のpackage identityは次の組です。

```text
(package name, exact version, source identity)
```

source identityはregistry artifactのcontent digest、またはpath dependencyのcanonical pathです。
同名同versionでもsource identityが違うpackageを一つのgraphへ混在させた場合はdependency
confusion errorとし、自動で片方を選びません。

## 11.3 標準layout

layoutを省略した場合は次を使います。

```toml
[layout]
source = "src"
tests = "tests"
generated = ".seseragi/generated"
```

```text
seseragi.toml
seseragi.lock
src/
tests/
.seseragi/generated/
```

各pathはpackage rootからの相対directoryです。absolute path、`..`、symlinkによるpackage root外への
escapeを禁止します。3 rootは互いに重複できません。

- source root: publish・build対象の通常module。
- test root: test commandだけがrootとして読み込むmodule。package exportにはできない。
- generated root: toolが再生成するforeign binding。手編集を前提にしない。

source moduleのidentityは `package identity::path`、generated moduleは
`package identity::@generated/path`、test moduleは `package identity::@test/path` です。
rootが違う同名fileを同一moduleとして扱いません。

relative importは現在moduleと同じroot内だけを移動でき、別rootへ `..` で横断できません。

## 11.4 module path

manifest内のmodule pathはrootからの相対pathを `/` 区切り・拡張子なしで書きます。

```text
main
http/client
bindings/date
```

空path、先頭 `/`、`.` / `..` segment、空segment、backslash、`.ssrg` suffixをmanifestでは
許しません。filesystemでは対応する `<path>.ssrg` 一件だけを解決し、directory indexや
拡張子候補を探索しません。

module pathはUnicode NFCへ正規化した形をidentityに使います。NFC正規化後に同じになる二つのfileを
同じrootへ置くことは禁止します。

case-insensitive filesystemでもmodule pathのcaseはsource spellingと一致しなければなりません。
caseだけ違う二つのmoduleを同じpackageへ置くことは禁止します。

## 11.5 export map

`exports` は公開specifier subpathからsource module pathへのmapです。

```toml
[exports]
"." = "lib"
"client" = "http/client"
"model/user" = "model/user"
```

package名が `acme/http` なら、上のmoduleはそれぞれ次でimportします。

```seseragi
import { request } from "acme/http"
import { Client } from "acme/http/client"
import { User } from "acme/http/model/user"
```

export key `.` はpackage root、それ以外はmodule pathと同じ正規化規則を使います。export targetは
source root内のmoduleだけに限ります。directory、test module、generated bindingを直接export
できません。generated bindingを公開APIに使う場合は、source rootに手書きadapterを置きます。

export mapにないpackage内部pathは、fileが存在しても外部packageからimportできません。
同じtargetを複数keyで公開することはできますが、module identityは一つです。

既存export keyの削除、別moduleへのremap、公開interfaceの非互換変更はpackageのbreaking change
です。新しいexport keyの追加は、それだけではbreaking changeではありません。

## 11.6 dependency

dependency keyはsource codeで使うbare import rootです。短縮形ではpackage名とimport rootが
同じです。

```toml
[dependencies]
"acme/http" = "^2.1.0"
"acme/json" = "~1.4.2"
```

別名または同じpackageの別versionを直接使う場合はlong formにします。

```toml
[dependencies]
"http" = { package = "acme/http", version = "^2.1.0" }
"http-v1" = { package = "acme/http", version = "^1.9.0" }
```

この場合は `http/client` と `http-v1/client` が別のpackage identityへ解決されます。resolverは
specifierのsegment prefixと一致する宣言済みdependency keyのうち最長のものをimport rootとし、
残りをexport subpathとして扱います。dependency keyを現在package名と同じにすることはできません。
宣言していないtransitive dependencyへ直接importできません。

開発中のlocal packageにはpath formを使えます。

```toml
[dependencies]
"http-local" = { package = "acme/http", path = "../http" }
```

pathは現在package rootを基準にcanonicalizeし、対象 `seseragi.toml` のpackage名と一致しなければ
なりません。対象versionと言語rangeも通常dependencyと同じように検査します。publishされる
manifestにpath dependencyを残すことはできません。

dependency versionはSemVer rangeです。range解決結果はlockfileのexact identityと一致しなければ
なりません。registry dependencyとpath dependencyは、解決後のidentity、公開interface、module
semanticsに同じ規則を使います。git URLやbranchをcore manifestのdependency sourceには使いません。

## 11.7 package内importとtest

package内から自身のpackage名とexport mapを使う公開self importを許可します。これは現在build中の
package identityへ解決し、dependency entryを要求しません。

package-privateなsource moduleは `self/` specifierで参照します。

```seseragi
import { parseHeader } from "self/internal/parser"
```

これはsource rootの `internal/parser.ssrg` へ解決します。source moduleとtest moduleだけが使え、
package外からは参照できません。export mapを通らないため、`self/` importによって名前が公開APIに
なることもありません。

test moduleはpackage名による公開self importと、`self/` によるprivate source importの両方を
使えます。test root間はrelative importできますが、通常sourceからtest rootをimportすることは
禁止します。

testからdependency、standard library、generated bindingをimportする規則は通常sourceと同じです。

## 11.8 generated binding

generated rootのmoduleは `gen/` で始まるspecifierからだけ参照します。

```seseragi
import * as rawDate from "gen/date-fns"
```

これはgenerated rootの `date-fns.ssrg` へ正確に解決します。`gen` はmanifest dependency名に
使えない予約import rootです。sourceとtestはgenerated moduleをimportできます。generated moduleは
同じgenerated root内のrelative import、standard library、manifest dependencyをimportできますが、
`self/` とpackage名によるself importは使えません。生成層から手書きadapterへの逆dependencyを
防ぐためです。

`.d.ts` converterは出力module path、入力digest、設定digest、generator identityとversionをmetadataへ
記録します。同じgenerator、入力、設定の生成結果はbyte単位で決定的でなければなりません。
staleまたは欠落した生成物はbuild errorとし、build中にnetwork取得や暗黙再生成を行いません。
再生成は明示commandです。

## 11.9 foreign host input

TypeScript / JavaScript foreign moduleを使うpackageは、host resolverの入力をmanifestへ明記します。

```toml
[foreign.typescript]
resolver = "node"
manifest = "package.json"
lockfile = "bun.lock"
```

`manifest` と `lockfile` はpackage root内のfileでなければなりません。`resolver` はtarget toolchainが
認識するhost resolver IDです。compilerはSeseragi dependencyとforeign host dependencyを混同せず、
foreign blockのbare specifierだけをこのresolverへ渡します。

host lockfileがない、manifestと不一致、同じspecifierが複数の実体へ解決される場合はbuild errorです。
foreign moduleのexact identityとdeclaration digestも `seseragi.lock` へ記録します。これにより
`.d.ts` binding生成時とbuild時が同じhost moduleを参照していることを検証できます。

## 11.10 executable entry

`run.entry` はsource root内のmodule path、`run.target` はhost adapterの識別子です。
entry moduleは6.11の公開 `main` を一つ持たなければなりません。

target IDとforeign resolver IDは `[a-z][a-z0-9-]*` に一致しなければなりません。

libraryとexecutableを兼ねるpackageは `exports` と `run` の両方を持てます。`run` を省略したpackageを
実行しようとするとerrorです。target固有optionは `[tool.<target>]` に置き、Seseragiの型やEffect
semanticsを変更できません。

## 11.11 lockfile

package managerはpackage rootへ `seseragi.lock` を生成します。lockfileは少なくとも、全packageの
exact identity、content digest、dependency edge、language version、標準ライブラリversionを固定します。

application buildとtestはlockfileを必須とし、manifestと不一致ならerrorにします。library publish時の
依存契約はmanifestのrangeですが、library自身のtestにもlockfileを使います。lockfile更新は明示commandで
行い、通常buildが勝手にversionを選び直しません。

## 11.12 workspace discovery

compiler、formatter、language server、playground toolは、対象fileから親directoryへ探索して最初の
`seseragi.toml` をpackage rootとします。そこで探索を止め、さらに外側のmanifestをmergeしません。

一つのeditor workspaceに複数package rootがある場合は、それぞれ独立したresolver graphを持ちます。
fileがどのrootにも属さない場合はstandalone syntax modeとしてparseとformatだけを行い、package import、
generated binding、型検査は未解決診断にします。
