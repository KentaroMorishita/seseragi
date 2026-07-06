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
hash_seed = "entropy"
random_seed = "entropy"
```

test commandの既定値はoptionalな`test` tableへ置きます。

```toml
[test]
target = "node"
jobs = 1
timeout_ms = 30000
cleanup_grace_ms = 5000
seed = 0

[benchmark]
target = "node"
warmup = 10
samples = 50
minimum_sample_ms = 100
regression_threshold_percent = 5.0
```

targetはCLI指定、`test.target`、`run.target`の順に選び、すべてなければtest開始前にtarget selection errorです。
jobsとtimeout_msは正の整数、cleanup_grace_msは0以上の整数、seedはsigned 64-bit integerです。省略時は
jobs 1、timeout_ms 30000、cleanup_grace_ms 5000、seed 0です。
CLIの対応optionはmanifest値をそのrunだけ上書きし、manifestやlockfileを書き換えません。

benchmarkのwarmupは0以上、samplesは3以上、minimum_sample_msは正の整数、
regression_threshold_percentは0以上のfinite Floatです。省略時は順に10、50、100、5.0です。benchmark runnerは
各sampleがminimum_sample_ms以上になる反復回数をcalibrateし、同じcaseの全sampleでその回数を使います。
target selectionはCLI指定、`benchmark.target`、`run.target`の順で、すべてなければ開始前にerrorです。

`signal_mode` は `cancel` または `forward` で、省略時は `cancel` です。`cancel` はtermination signalを
root Effectのcancellationへ変換し、`forward` は `std/process.signals` へ渡します。
`shutdown_grace_ms` は0以上の整数で、省略時は10000です。cancel modeでだけ指定でき、0はcancellationを
要求した直後にforced terminationへ移る設定です。

grace periodはhostのmonotonic clockで測り、application environmentのClock serviceを要求・参照しません。
test hostはsignal入力とmonotonic timeを決定的に制御できなければなりません。

`hash_seed` は `"entropy"` またはsigned 64-bit integerで、省略時は `"entropy"` です。entropy modeは
hostのsecure entropyを要求します。integerは同じHash implementationで内部bucket配置を再現するための
test・benchmark用設定で、production packageでの使用はtool warningの対象です。seedはMap / Setの反復順や
serialized dataを変えないため、通常testは固定seedへ依存してはなりません。

signalを持たないtargetで `forward` を選ぶこと、またはsignal / graceful shutdown capabilityを持たない
targetへこれらのkeyを明示することはmanifest errorです。target adapterは対応可否をbuild時に公開し、
runtimeで黙って無視しません。

`random_seed`は`"entropy"`またはsigned 64-bit integerで、省略時は`"entropy"`です。standard Random serviceを
main environmentへ提供するときのseedで、Map / Setのhash_seedとは別です。entropy modeを選んだtargetにsecure
entropyがなければapplication code開始前にtarget errorです。固定integerはsimulationの再現やdevelopment用で、
security用途へ昇格しません。Entropy service自身の出力を固定する設定はありません。

manifestはUTF-8のTOML 1.0です。未知のcore key、同じkeyの重複、型の違う値はerrorにします。
tool固有設定だけは `[tool.<tool-name>]` 以下に置けます。compilerは未知のtool tableを保持して
構いませんが、言語semanticsへ影響させません。

core top-level tableは `package`、`layout`、`exports`、`dependencies`、`foreign`、`run`、`test`、
`benchmark`、`tool`
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
benchmarks = "benchmarks"
generated = ".seseragi/generated"
```

```text
seseragi.toml
seseragi.lock
src/
tests/
benchmarks/
.seseragi/generated/
```

各pathはpackage rootからの相対directoryです。absolute path、`..`、symlinkによるpackage root外への
escapeを禁止します。4 rootは互いに重複できません。

- source root: publish・build対象の通常module。
- test root: test commandだけがrootとして読み込むmodule。package exportにはできない。
- benchmark root: benchmark commandだけがrootとして読み込むmodule。package exportにはできない。
- generated root: toolが再生成するforeign binding。手編集を前提にしない。

source moduleのidentityは `package identity::path`、generated moduleは
`package identity::@generated/path`、test moduleは `package identity::@test/path`、benchmark moduleは
`package identity::@benchmark/path` です。
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

test commandはtest rootの`.ssrg`をcanonical module path順に再帰列挙し、全moduleをcompileしてから一件も
実行せずdiscoveryします。`pub let tests: test.Test`を持つmoduleだけがtest treeを
公開し、持たないmoduleはhelperです。別名export、複数export、型が違うtestsはtestとして推測しません。

caseのfull nameは`module::suite::case`で、nested suiteはsegmentを`::`で追加します。nameとskip reasonは
空String、先頭・末尾のUnicode whitespace、`::`、C0 / DEL controlを含められません。同じfull nameが二件
あればdiscovery errorです。module順と各Test treeのsource順を合わせた順序がcanonical discovery orderです。

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
bindings = "seseragi.bindings.toml"
```

`manifest`、`lockfile`、省略可能な `bindings` はpackage root内のfileでなければなりません。`resolver` はtarget toolchainが
認識するhost resolver IDです。compilerはSeseragi dependencyとforeign host dependencyを混同せず、
foreign blockのbare specifierだけをこのresolverへ渡します。

host lockfileがない、manifestと不一致、同じspecifierが複数の実体へ解決される場合はbuild errorです。
foreign moduleのexact identityとdeclaration digestも `seseragi.lock` へ記録します。これにより
`.d.ts` binding生成時とbuild時が同じhost moduleを参照していることを検証できます。

runtime load cacheのkeyもこのexact identityです。specifierのaliasやre-export pathは別moduleを作りません。
generated binding metadataに記録したmodule evaluation modeと現在のconverter設定が違う場合はstale bindingとして
buildを拒否します。pure-load / task-loadの意味と同一identityを複数blockから参照する規則は7.2に従います。

## 11.10 executable entry

`run.entry` はsource root内のmodule path、`run.target` はhost adapterの識別子です。
entry moduleは6.11の公開 `main` を一つ持たなければなりません。

target IDとforeign resolver IDは `[a-z][a-z0-9-]*` に一致しなければなりません。

libraryとexecutableを兼ねるpackageは `exports` と `run` の両方を持てます。`run` を省略したpackageを
実行しようとするとerrorです。target固有optionは `[tool.<target>]` に置き、Seseragiの型やEffect
semanticsを変更できません。

runnerは6.11のentry point規則に従い、entry moduleの公開 `main` を読み込み、匿名Unit値 `()` を
一度渡してEffect valueを得ます。Effect valueはtarget adapterが用意したroot resource scopeで実行し、
`main` のclosed environment requirementに必要なserviceだけを照合して供給します。actual host
environmentは追加serviceを持てますが、required fieldを欠く場合や型が合わない場合はrun前のhost
configuration errorです。

generated TypeScript moduleの公開 `main` はEffect valueを返すだけで、Promise化、throw化、process
exit、Consoleへの直接writeを行いません。stdout / stderr、exit code、typed failure表示、defect、
cancellation、signal処理はrunnerとtarget adapterの責務であり、10.14と6.11の規則に従います。

conformance runnerはsource path、absolute build path、host module cache pathを比較identityに含めません。
entry、runtime ABI major、required runtime feature、host service trace、stdout / stderr、exit分類を比較します。

## 11.11 lockfile

package managerはpackage rootへ `seseragi.lock` を生成します。lockfileは少なくとも、全packageの
exact identity、content digest、dependency edge、language version、標準ライブラリversion、runtimeが使う
IANA timezone databaseのexact release IDを固定します。

lockfileはUTF-8 / LFのTOML 1.0で、schema 1の形は次です。

```toml
schema = 1
language = "0.1.0"
standard_library = "0.1.0"
unicode = "16.0.0"
timezone_database = "2025b"
root = "acme/app@1.0.0#workspace:."

[[packages]]
id = "acme/app@1.0.0#workspace:."
name = "acme/app"
version = "1.0.0"
source_kind = "workspace"
source = "."
manifest_digest = "sha256:0123456789abcdef..."
content_digest = "sha256:0123456789abcdef..."
dependencies = [
  { import = "http", package = "acme/http@2.1.4#registry:default" },
]

[[packages]]
id = "acme/http@2.1.4#registry:default"
name = "acme/http"
version = "2.1.4"
source_kind = "registry"
source = "default"
manifest_digest = "sha256:..."
content_digest = "sha256:..."
dependencies = []
```

`source_kind` は `workspace | path | registry` のclosed enumです。workspace sourceはrootだけでsource `.`、
path sourceはrootから対象packageへの `/` 区切りrelative path、registry sourceはresolver設定が選んだstableな
registry IDです。absolute path、hostname、取得cache pathをlockfileへ書きません。pathは使用時にcanonicalizeし、
package root外も許しますが、同じrelative spellingとcontentからmachine非依存なidentityを作ります。

idは `<name>@<version>#<source_kind>:<source>` です。nameとversionは対象manifestと一致しなければならず、
rootはpackages内のworkspace entry一件を参照します。dependenciesのimportはroot manifestで宣言したdependency key、
packageはpackages内のidです。同じimport、id、package identityの重複はlockfile errorです。packagesはidのUTF-8 byte順、
dependenciesはimport順、TOML keyは上の順で生成します。

manifest_digestは対象 `seseragi.toml` のraw bytes、content_digestはmanifest、通常source、公開generated interface、
converter metadataをpath順にlength-prefixしてSHA-256したlowercase hexです。lockfile自身、test / benchmark root、build
artifact、absolute path、mtimeはcontent digestへ含めません。foreign moduleはpackage entryのcontent digestにbinding
metadataとdeclaration digestを含め、host lockfile全体のraw pathを含めません。digest prefixは必ず `sha256:` です。

writerは上記canonical順と末尾newline一つで生成し、readerはTOML上のkey順を要求しません。未知keyは同じschema major
では無視しますが、未知schema major、必須key欠落、型違い、非canonical ID / digest、参照先のないedgeを拒否します。
manifest、source content、converter metadata、toolchain version databaseのいずれかがlockと違えば `SES-K0102` で、
通常build中に書き換えません。

application buildとtestはlockfileを必須とし、manifestと不一致ならerrorにします。library publish時の
依存契約はmanifestのrangeですが、library自身のtestにもlockfileを使います。lockfile更新は明示commandで
行い、通常buildが勝手にversionを選び直しません。

## 11.12 workspace discovery

compiler、formatter、language server、playground toolは、対象fileから親directoryへ探索して最初の
`seseragi.toml` をpackage rootとします。そこで探索を止め、さらに外側のmanifestをmergeしません。

一つのeditor workspaceに複数package rootがある場合は、それぞれ独立したresolver graphを持ちます。
fileがどのrootにも属さない場合はstandalone syntax modeとしてparseとformatだけを行い、package import、
generated binding、型検査は未解決診断にします。
