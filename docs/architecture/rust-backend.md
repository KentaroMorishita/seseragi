# Rustバックエンドの設計と対応範囲

[寿命管理とEffectの研究](../experiments/native-lifetimes.md)から、同期Effect / Ref / scopeと
循環回収をexperimental CLIへ統合する実装を進めている。現在の範囲と検証は
[native runtime統合](../experiments/native-runtime-integration.md)、全体の進捗は
[実用化Issue](https://github.com/KentaroMorishita/seseragi/issues/809)で追跡する。

## 判断と実装境界

Rustソース生成を第二のbackendとして段階的に実装する。Seseragiの型検査、instance選択、
評価順序の authority は既存frontendとCoreIrに置く。ユーザーにRustの所有権注釈を要求しない。
生成されたプログラムはRust標準ライブラリのみを使い、実行時にBun、Node、TS runtimeを必要としない。

experimentalなpure / Effect、structured Fiber / Deferredと検証済みの複数module・path packageを扱う。既存`process` / `web` targetの意味は
変えず、CLIでは`experimental`配下から明示的に選ぶ。純粋entryは結果をinspection表示し、
Effect entryはnativeの実行loopとresource scopeで実行する。

```text
Source → shared frontend → TypedHir → CoreCompilation / CoreIr
                                        ├→ TypeScriptIr → TS runtime
                                        └→ seseragi-backend-rust → Rust source + metadata
                                                                         → rustc → native
```

`seseragi-driver::compile_core_module`はTS lowering / emissionを呼ばない。
既存TSの`compile_module`とprojectのchecked-module loweringも同じCoreCompilation経路を使う。
`CompiledModule`の既存TS APIは互換性を維持する。backend間で一つの巨大な生成物structを共有せず、
Rust側は`RustArtifact`、`RustMetadata`、`RustBackendError`を返す。

## 値、closure、メモリ

pure ABI名は`rust-erased-pure-v1`、同期Effectは`rust-erased-sync-v2`、
Fiber / Deferredを使う生成物は`rust-erased-cooperative-v3`。型消去した共通値`V`を使い、generic / HKTをRustのgenericsで
再現する必要をなくす。Unit / Int / Boolは即値、文字列・collection・ADT・関数・dictionaryは
immutableな共有ノードを使う。Intは既存言語のsafe integer範囲を維持し、算術で検査する。

closureは`captures: Vec<V>`と`fn(&[V], V) -> V`の組である。非捕捉のRust関数pointerを使うので、
値への参照がopaqueな`dyn Fn`の中に隠れない。CoreIrのfree-variable分析で必要なterm nameと
hidden dictionaryだけを捕捉し、不明な式はscope全体へ保守的に戻す。

共有cloneは内容を複製しない。最後の参照を破棄するときはthread内の作業キューへノードを入れ、
親の破棄で発生する子の破棄もキューへ積む。これにより深いADT・collection・captureの破棄が
Rustのcall stackを再帰的に消費しない。inspection表示も明示的なstackで走査する。

pure programは通常のRCを使う。Ref / Fiber / Deferredを使うmoduleでは実行前にweak registryを有効にし、
全container、closure、dictionary、Effect captureのedgeをtraceする。外部rootは強参照数から
内部edgeとcollectorのsnapshotを差し引いて求める。到達不能なRef、完了したFiber / Deferredのpayload edgeを切り、反復Dropで回収する。

mutableなedgeを持つRef / Fiber / Deferredは、borrowのないcheckpointでtraceと切断を行う。local recursive closureや
opaque foreign capture、Signalはまだ未対応。callback captureをWeakへ変えない。
collectorはlanguage finalizerを実行せず、finalizerはscopeの所有物として実行loopが管理する。
full scanを512 checkpointごととroot終了時に行う。停止時間の上限やpending host I/Oの接続は未実装。

## CoreIrで共有する分析

`seseragi-lowering`へtargetに依存しない三つの分析を置いた。CoreIr自体は書き換えない。

- `known_pure_call_shape`: 宣言のparameter数から部分適用・完全適用・返却関数への適用を区別する。
  return typeの矢印数をsource arityとして数えない。lexical shadowingとdeferred evidenceは
  backend側で除外する。
- `is_closed_local_evidence`: parameter evidenceを含まないlocal dictionary依存木を判定する。
  imported / standard / derived / supertrait / method constraintsはここで共有可能と断定しない。
  この判定だけで初期化移動を認めず、backend側でfactoryがmethod closureのみを構築することを確認する。
- `core_free_variables`: term nameとhidden evidence parameterを別に扱い、lambda、順次let、
  decision branchのscopeを解釈する。未対応shapeは`None`を返し、勝手にcaptureを落とさない。

Rustの既知関数の完全適用はdirect helperへ落とす。部分適用・alias・返却関数はcurried ABIを保つ。
通常のcallは各stageを適用してから次の引数を評価する。たとえば`returned a b`では、`returned a`の
bodyが実行されてから`b`を評価する。選択済み辞書で証明できるlocal callable以外の、未対応stagingは診断する。

閉じた単相のmethod参照は、選択済みprimary辞書から得た関数値をそのまま保持する。
保存した`add`や`eq`へ左引数を渡すと、そのsource stageを実行し、返った関数を保存する。
nativeは辞書のowner・型引数・constructorのkindと、実装のsource parameter / bodyの返り値型を照合する。
letで汎化するaliasやmethod固有制約の辞書導入は別に扱う。これらのmethod参照と、source引数を並べ替える
標準演算子のstrict評価は、[部分適用のIssue](https://github.com/KentaroMorishita/seseragi/issues/887)に残る。

純粋関数の直接の自己末尾callは、if / decision / pure sequenceの末尾を辿ってloopへ落とす。
次のdictionaryと全引数を左から右へ評価した後、freshなloop storageへ代入する。各周回のsource
parameterはimmutable snapshotなので、closureが前の値を保持でき、letによるshadowingもstorageへ
干渉しない。部分適用と返却関数のABIは維持する。返却lambda内のcall、alias経由、相互再帰、
非末尾callをこの変換の対象にしない。Effectは別の明示的frameでstackを管理する。

dictionaryはthread内cacheで遅延共有する。static local factoryも入れ子のclosed factoryも共有し、
parameter-dependent factoryは共有しない。cache keyはinstance identity、type arguments、依存evidenceを
含めた選択済みCore evidenceに基づく。factory evaluation中はcacheをborrowしない。harness終了時に
明示的にcacheを空にする。TLS破棄時のcache teardownも検証する。

[TS実測](../experiments/core-ir-performance.md)で改善が見えた二つの処理をこの分析へ戻した。
TS backendはfree-variable分析を非再帰letのinitializer scope判定へ使う。同名の外側の値を
参照するinitializerは新しいbindingのscope外で評価し、JSのTDZに捕捉される不具合を防ぐ（#820）。
direct helperとdictionary factoryの共有はRust backendの実装であり、TSへ同じ変換を導入したとは扱わない。
TS / nativeのscalar self-tail loop認定とscalar global readの共通改善は#822 / #843で検証した。
過去の測定用TSコピーの改善率を製品性能として扱わず、条件と適用範囲を#816で追跡する。

## 対応範囲と診断

対応するのは、pure / 同期Effect function、返却closure、部分適用、generic / HKTの型消去、
選択済みのuser instanceとnested dictionary、Int / Bool / String、基本Char / Float / List、
Tuple / Array / field-only Record、local ADT、decision、let / effect sequence。
Show / Debugは対応済みの標準辞書と選択済みchild methodを使う。
Arrayの`values[index]`はcanonical Array / Int / Maybeと要素型を検証し、
receiver、indexをこの順で一度ずつ評価する。負数・範囲外・空配列ではNothingを返し、
有効な位置では要素の共有edgeをJustへ保持する。取り出した関数・Ref・cold Effectは
配列本体を破棄しても使える。固定長Arrayパターンと`std/array.get`は未対応。
標準MaybeのShow / Debugも選択済みchild methodを使い、Nothingでは呼び出さない。
検証済みのglobal / local / method部分適用では、渡した引数を作成時に評価する。
Maybe表示も既存のclosed dictionaryと残るsource parameterの検査を通る範囲で扱う。
未証明の型・辞書stagingは生成前に拒否する。部分適用した関数値をArrayへ保存・取得する経路も扱う。
Effectはsucceed / fail / defer / flatMap / recover / mapError、Refはmake / get / set / update / modify、
resourceはscoped / acquireRelease、Consoleはprint / printlnとqualified std/consoleの出力を扱う。
`std/path`のcurrent / parse / child / join / render / normalize / isAbsolute / parent /
fileName / extensionと、`std/fs`のreadTextUtf8 / writeTextUtf8を同期native hostで扱う。
write modeはReplace / CreateNew / Append。Pathはcanonical ownerを持つopaque値として保持し、
String aliasへ置き換えない。必要なPath・filesystem・UTF8エラーの選択済みShow / Debugと、
`std/effect.fromEither`のfailure / success型を照合する。
小さいファイルを全量で読み書きし、handleはoperationが所有する。成功・失敗とも一度closeし、
read / write / flushの失敗をcloseの失敗で上書きしない。成功後のclose失敗は型付き失敗になる。
Effectの作成はI/Oせず、同じEffectを実行するたびにI/Oを行う。
この範囲はFileSystemの標準host実装に限る。packageの明示filesystem provider選択やprovider artifactを含む構成は出力前に拒否し、
JavaScript providerを黙ってnative hostへ差し替えない。全FileSystem API、async I/O・キャンセル、
低水準handle公開や一般のprovider ABIの対応を意味しない。

Effect machineはstate、継続frame、resource scopeを保持して中断・再開する。単一runtime threadの
FIFO queueでcheckpointごとにrunnable Fiberを進め、managedな`V`をhost threadへ渡さない。
`fork` / `await` / `poll` / `join` / `interrupt` / `yieldNow`と、`std/deferred`の
make / await / poll / complete / succeed / failを扱う。source引数、環境row、failure / success、
opaque型のcanonical ownerと各slotのvalue kindを、callと保存した関数値の両方で照合する。

scopeは未完了childへcancellationを要求し、cleanupを待つ。待機者だけの取消はtarget Fiberや
Deferredを完了させない。Deferredは最初の完了だけを受け入れ、waiter登録順で再開する。
resource取得成功から登録までを保ち、finalizer中は取消をmaskする。finalizerがDeferredで
待機しても継続を保持し、他のFiberを動かせる。collectorは待機中のframeとscopeをrootとして扱う。
完了したFiberのtyped failureはjoinで伝播し、awaitではFiberExitになる。Defectはfailureへ変換しない。

これは非同期I/Oを載せるための中断・再開機構である。現在のFileSystem hostは同期のままであり、
pending host operation、外部process shutdownのcooperative処理、timer、race / parallel API、
Queue / Stream等まで対応したという意味ではない。compiler WASMの生成TSで動くことと、一般program
WASMの対応も分ける。

検証済みのsource module / path packageをlinkし、単相globalの初期化順も保持する。
全source moduleを検査し、未使用の未対応declarationも黙って消さない。

CoreIrはgeneralizeしたlet lambdaと名前付きlocal functionのbinding scopeを区別する。
let lambdaは前のlexical scopeを捕捉し、名前付き関数のbodyは自身の名前を参照する。
TSはlet lambdaを外側のscopeで生成してからbindingし、nativeは未対応のlocal self参照を
同名の外側の値へ解決せず、生成前に拒否する。

非再帰の制約付きlocal functionとgeneralizeしたlambdaは、呼出しごとの辞書を受け取る。
内部の辞書factoryとsource parameterの段階を分け、選択済みevidenceを辞書frame内のindexへ対応づける。
外側から継承した制約も明示的に渡す。辞書とterm captureは既存のruntime graphで追跡し、
保存関数・部分適用・返却closureが使う値を保持する。部分適用を作るときはcalleeと渡したsource式を
左から右へ一度だけ評価し、未使用でもDefectを省略しない。再利用時は評価済みの値を使う。
未証明のconstructor parameterやstagingは生成前に拒否する。

instance method固有の`where`制約も、instance factoryの制約と分けて扱う。
factoryの辞書を受け取ってmethod closureを作り、source parameterの後にmethod用辞書を渡す。
method内のevidence indexにはfactory側の制約数を加える。返された関数への追加引数は、
method用辞書を渡してbodyを実行した後に評価する。

この呼び出しでは、確定したsource parameter型とprimary / method制約をTyped HIRからCoreIrへ保持する。
nativeはcanonical owner・型引数・辞書の数を検査する。TSも同じsource arityを使う。
TSは渡されたsource引数を先に適用して残る関数値を保存し、未適用のsource引数とその後の辞書だけを
closureへ残す。nativeのmethodも同じ位置まで適用して保存し、method固有の辞書は残りのsource引数の後へ置く。
保存・返却closureの辞書は既存のcapture graphで追跡する。推論された標準Refは`std/ref::Ref`として比較し、
同名のユーザー型のownerと混同しない。

この修復はsource定義の関数と固有制約付きmethodの経路を対象にする。比較演算子のsectionや
標準関数専用の生成経路には、部分適用のstrict評価の調査・修復が残る（#887）。

現在の未対応範囲：async Effect、残るstdlib API、一般のpolymorphic global、再帰local function、
record spreadと未対応pattern、derived instance、supertrait、一般のMonad辞書を使うdo、
一般のdeferred callable evidence、foreign / provider ABI、並列runtime、標準のprogram WASM target、
source map、一般のDefect provenance。Floatに標準Eq / Ord / Hashは導入しない。
直接の純粋自己末尾call以外はRust stackを使う。一般の再帰・相互再帰のconstant-stack保証はない。
深い値の破棄と深い関数callは別の境界である。

未対応CoreIrは`SES-RUST-0001`とsource byte rangeを持つtarget errorへする。frontendの型エラーは
既存診断を維持する。target検査に失敗した場合はRust sourceやnative outputを作らない。
pure inspectionのInt defectはRust panicとなる。Effect実行中のpanicはDefectへ移し、scopeを閉じて
から報告する。typed failureのpayloadは任意の`V`をそのまま保持し、recover / mapErrorへ渡す。
entryで未処理の一般のuser-defined errorに選択済みShowを適用することとsource frameの報告は残る。

## CLIと再現

Rust toolchainがあれば次を使える。`emit-rust`はrustcもBunも起動しない。
`build-native`はPATH上のrustcでhost native executableを作る。cross compilationはまだ公開しない。

```sh
cargo run --locked -p seseragi-cli -- experimental emit-rust \
  examples/spec/fixtures/rust-backend/ownership.ssrg \
  --entry probe --output /tmp/seseragi-ownership.rs

cargo run --locked -p seseragi-cli -- experimental build-native \
  examples/spec/fixtures/rust-backend/ownership.ssrg \
  --entry probe --out-dir target/rust-native-ownership

env PATH="" target/rust-native-ownership/program
# (42, 42, 11, 12)
```

`build-native`は`program.rs`、`rust-backend.json`、`program`（Windowsは`.exe`）を保存する。
metadataは生成sourceのbackend、experimental status、ABI、logical module、entry、edition、
生成code内のdirect call / self tail call / closed dictionary site数を記録する。
速度やallocation削減の証明ではない。
このmetadataは既存production artifact-manifest契約へまだ統合しない。

## 検証と次のslice

新backendのunit/runtime testsとnative execution fixturesをRust workspace gateへ組み込んだ。
CLI testはpure 10 fixtureとEffect 14 fixtureをTSとnativeで実行し、stdout / exitとDefect以外の
stderrを比較する。native実行のPATHは空にする。所有権 / dictionary fixtureは1万回反復し、
cacheがwarmした後のlive node数が一定で、明示的なcache teardown後に0となることを確認する。
自己末尾再帰は100万段のsum、再利用した部分適用と各周回のclosure snapshot、generic辞書を
入れ替える再帰を256 KiB stackで検証する。counterは管理ノードのみを数え、RSSではない。

```sh
cargo test --locked -p seseragi-backend-rust
cargo test --locked -p seseragi-cli --test rust_backend
bun run check:rust -- -p seseragi-driver -p seseragi-lowering -p seseragi-backend-rust
bun run check
```

shared driver / compiler基盤とCLIを変更するため、scoped testsの後にrepository-wide gateを実行する。
WASMの正式生成物はLinux x64で再生成して確認する。macOS生成物をcanonicalとして上書きしない。
実際のgate結果と未検証境界は作業の検証記録に残す。

次の範囲と受入条件は[#809](https://github.com/KentaroMorishita/seseragi/issues/809)に集約する。
基本言語機能、stdlib / package、async provider、foreign ABI、program WASM、実測とCoreIr改善、
標準の開発経路と配布を別Issueで追跡する。全言語対応やstable化を現在のsliceの完了と混同しない。
