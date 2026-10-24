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

## pure program WASMとtypedな返却関数

`experimental build-wasm`はcompiler WASMとは別の、利用者のprogramを生成する。
公開する関数を明示して、checked Coreの完全な関数型と宣言のsource引数を照合する。
単相で制約のないpure関数が対象で、Unit／Bool／Int／String、2〜32要素のTuple、
標準Maybe／Either、返却する関数を扱う。配列・record・任意ADT・generic公開scheme、
入力側の関数、Effect／Ref／providerを含むmoduleは出力前に位置付きで拒否する。

```sh
rustup target add wasm32-unknown-unknown
seseragi experimental build-wasm \
  examples/spec/fixtures/rust-backend/program-wasm.ssrg \
  --export label --export validate --export staged --out-dir target/program
```

source fileとlock済みpackageを受け取り、`program.wasm`、loaderの`program.mjs`、
型宣言`program.d.mts`、生成Rustと`program-wasm.json`を出す。manifestには実際のrustc、
sourceと生成fileのSHA、canonical公開型、source arityとcodecを記録する。失敗したbuildは
前の完成したartifactを保存し、利用者が編集した出力や無関係なdirectoryは置き換えない。

```typescript
import { loadProgram } from "./program/program.mjs"
const program = await loadProgram(wasmBytes)
const prefix = program.exports.label("合計: ")
const callback = prefix(40)
prefix.dispose()
console.log(callback(2)) // 合計: 42
callback.dispose()
program.dispose()
```

JSの一回の呼出しは一つのsource値を適用する。Unitも一段階であり、辞書の導入や
返り値関数の引数と混ぜない。返却callbackはstrongな外部rootで、detachするまで捕捉を保つ。
tokenは再利用せず、detach／instance dispose後の呼出しはWASMへ入らず拒否する。
通常disposeはglobal・辞書・callbackのrootを解放し、managed-live0を検査する。

このcheckedな完全関数型は共有TS loweringでも使う。pureで単相・制約なしの自己参照関数は、
primitive／Tuple／Maybe／Eitherと関数で閉じる型をconst注釈へ残し、TSの再帰推論による
TS7023を防ぐ。unknown、nominal参照、generic、辞書やasyncの署名へ型を補う処理は加えない。

loaderはIntのsafe範囲、UTF-8／surrogate、Tupleの要素数とown data field、Maybe／Eitherの
variantを検査する。Unitは`undefined`、Tupleはreadonly array、Maybe／Eitherは
`{tag, value}`（Nothingは`{tag}`）で、境界は16MiBまで。loaderとWASMの契約を初期化前に
照合する。この照合は任意の悪意あるWASMを安全に実行する仕組みではない。

整数Defectもengineのstack trapも`ProgramWasmDefect`となり、instanceをretireして
host所有参照を閉じる。originは呼出した公開関数を指し、内部の演算位置までは保証しない。
RustのunwindやEffect finalizerの完了をtrap後に主張せず、`managedLiveAtClose`はnullのままにする。
WASM physical heapの回収時刻も保証しない。新しいinstanceで実行を再開できる。

```sh
python3 scripts/build-program-wasm-demo.py --cli seseragi --out target/program-demo
python3 -m http.server 8080 --directory target/program-demo/site
```

このsampleは、別のSeseragi sourceを既存web targetでTSへ生成し、task foreign bridgeから
program WASMを呼ぶ。画面で成功値・Eitherのdomain failure・callback detach・instance dispose・
Defect後の再生成を確認できる。assetの配置はsample scriptで明示している。標準package buildへの統合は次の経路を使う。UI stateを保持するHMRと一般の同期host callbackは残る。providerless Effectと型付きのhost Effect callbackは次の別ABIで扱う。
これだけで#814／#815や実用化全体を完了扱いにしない。

## providerless Effectを実行・取消する

閉じた公開Effectを選ぶと、`seseragi-program-wasm-effect-v1`を生成する。
pure ABIとは別のloaderとresumable runtimeを使い、従来のpure出力を保つ。
source引数を適用するとcoldな`OwnedEffect<E, A>`を返し、`run()`で実行を開始する。
辞書factory、source引数、返り値関数とEffectの実行を一段階へまとめない。

```typescript
const plan = program.exports.value("雪")
const execution = plan.run()
const outcome = await execution.result
switch (outcome.tag) {
  case "Success": console.log(outcome.value); break
  case "Failure": console.log(outcome.error); break
  case "Defect": console.error(outcome.messages); break
  case "Cancelled": break
}
plan.dispose()
await program.dispose()
```

`Success`、型付き`Failure`、明示的な言語`Defect`、`Cancelled`を区別する。
`std/effect.die`はStringを受け取るcoldなDefectで、`recover`や`mapError`の対象にはならない。
scopeの全finalizerをLIFO順で試み、cleanup Defectを失わず結果へ渡す。
明示Defectの後もinstanceは使える。予期しないWASM trapやprotocol違反はinstanceをretireし、
その場合のRust unwindやfinalizer完了を保証しない。

一つのinstanceで実行できるroot Effectは一つである。boundedなmachine turnの間でJSのtaskへ
制御を返し、`execution.cancel()`は取消を要求する。開始済みのacquisitionと登録済みfinalizerを
保持し、masked cleanupを終えてからterminal outcomeを公開する。planのdetachは実行を取り消さず、
実行側がそのplanを保持する。返却callbackも独立したrootであり、planのdetach後に利用できる。
`program.dispose(): Promise<void>`は新しい呼出しを止め、実行中のrootを取消し、cleanup完了後に
全rootを解放してmanaged-live0を確認する。dispose中のcleanup Defectは解放後にrejectする。

このABIはproviderを持たない閉じた型と、生成前に証明した式だけを扱う。Intはsafe範囲の
値の受渡しに限り、演算は含めない。String連結、Bool分岐、Tuple／Maybe／Either、source関数の
保存・部分適用・返却closureと、選択した標準Effect操作を扱う。標準Effect関数自体の保存・部分適用、
Ref、Fiber／Deferred、host入力callback、open provider row、generic公開scheme、未証明の式や
同期依存cycleは、未使用のbodyにあっても位置付きで拒否する。取消はcooperativeで、
host I/O providerやOS resourceのWASM ABIを実装した意味にはならない。

Preludeの`succeed ()`と標準moduleを明示した`effects.succeed ()`は匿名Unitを渡せる。
同名のnominal Unitとはcanonicalな型identityを区別し、暗黙の変換を加えない。

## 型付きhost Effect callbackを渡す

公開関数の引数に、閉じた`A -> Effect<{}, E, B>`を持つ関数を渡せる。
この場合は`seseragi-program-wasm-host-effect-v1`を生成し、pure／providerlessの出力を保つ。
引数・成功値・失敗値には既存の閉じた値codecを使い、成功／失敗の`Never`は値を持たない。
同期host関数、複数のsource引数を取るhost関数、open row、generic、Ref／Fiber／Deferred、
foreign handleや一般provider ABIは生成前に拒否する。

```seseragi
pub effect fn transfer
  request: (String -> Effect<{}, String, String>) ->
  release: (String -> Effect<{}, Never, Unit>) ->
  input: String -> String fails String = effects.scoped $ do {
    resource <- effects.acquireRelease (request ("open:" + input)) release
    result <- request ("read:" + resource)
    effects.succeed ("processed:" + result)
  }
```

生成TSの`HostEffect<A, E, B>`は`(argument, {signal})`を受け取り、
`HostResult<E, B>`またはそのPromiseLikeを返す。結果は`Success`の`value`、`Failure`の`error`、
`Defect`の`messages`で、hostから任意の`Cancelled`を返すことはできない。
AbortSignalはtransportの情報であり、Seseragiのsource arityには加えない。
host関数の適用だけではI/Oせず、cold planの`run()`がその関数を一度呼び、応答を待って再開する。
hostのthrow／rejectや不正な応答は言語Defectへ変換し、登録済みfinalizerを試みる。

取消はunmaskedなhost operationのsignalへ通知し、operationの応答を待つ。acquisitionとfinalizerは
maskを保つ。取消中にacquisitionが成功した場合もreleaseを登録し、releaseの応答後に結果を公開する。
async disposeもこの終了を待ち、正常終了でcallback・plan・host登録とmanaged rootを解放する。
host登録は捕捉している最後のlanguage値が解放された時にも回収する。planのdetachだけで実行は取消さない。
同じinstanceへのsource呼出しはhost応答の待機中に拒否し、別instanceは独立して利用できる。

非協調的なPromiseの強制終了や、host関数が自分自身のinstanceのdispose完了を待つ循環は扱わない。
hostに渡したresourceの意味と実際のcloseはhost側が所有する。受入fixtureは実際の非同期fetchと
JSのresource記録を使い、OS file descriptorのWASM ABIを実証したとは扱わない。
予期しないengine trap／protocol違反ではinstanceをretireし、Rust unwindやfinalizer完了、
managed root0を主張しない。一般interopや#814／#815全体の完了とは分ける。

## 同じpackageのWASM計算とTS UIをbuildする

rootの`seseragi.toml`で計算moduleと公開関数を選ぶ。

```toml
[web.programs.calculation]
module = "calculation"
exports = ["label", "validate", "staged"]
```

```sh
seseragi build examples/spec/fixtures/rust-backend-projects/program-wasm-package \
  --target web --profile development --out-dir target/program-site
```

このpackageにはSeseragiの`src/calculation.ssrg`と`src/main.ssrg`がある。標準build一回で計算を
program WASMへ、UIをTSへ生成し、手動copyなしで配布できる。`host/bridge.ts`は生成virtual
packageの`@seseragi/programs/calculation`から`loadProgram`、所有callbackの型と
`programWasmPath`をimportする。WASMは`new URL(programWasmPath, document.baseURI)`から取得する。
外部TSの通常のmanifest／lockfile入力も必要である。

選択moduleはsource rootに対するcanonical module pathで、UIのrun entryを変更しない。
共有loaderでそのimport closureを読み、path dependencyの可視性・ownerと初期化順を保つ。
計算moduleから未使用のEffectやforeignを削って認定することはしない。root Web設定が選んだ
pure、providerless Effectまたは型付きhost Effect callbackのABIを生成し、dependency manifestのprogram設定を自動実行しない。実際に生成した
moduleのidentityとloader digestを渡してUIのbrowser importを検査する。

`programs/calculation/`にWASM・loader・型宣言・生成Rust・receiptを配置する。通常の
`artifact-manifest.json`の`programs`へcanonical module、ABI、host importと各assetの相対pathを、
`files`へ実際のbytesとSHAを含める。development／releaseのUIを同じ経路でbundle化する。
型・capability・rustc・bundleの失敗やpublic assetの衝突、検査済みsourceの変更があれば
途中のsiteを公開せず、前の完成artifactを保持する。source／manifestのreceiptを通常の出力と分けない。

`seseragi dev`も同じproducer・UI compile・一括公開を使い、計算module・path dependencyとUIの
変更を監視する。失敗時はそのsessionの最後の完成siteとversionを保持する。reload clientは
HTMLをserveした時のversionを使い、初回fetchで新しいbuildを見落とさない。

生成UIのloaderから作るinstanceはbrowser applicationのlifetimeへ登録する。watchのfull-page
reloadではTS UIをcancelし、finalizer・DOM・providerの終了を待ってから残るinstanceを閉じる。
UIのfinalizerが旧callbackを使える順序を保ち、開始済みloadも回収してからnavigateする。保存callbackは閉じた
ownerを拒否する。trapは従来どおりretireし、Rust unwindやWASM physical heapの即時回収は保証しない。
手動disposeとstandaloneのraw program ABIは維持する。UI stateを保持するHMR、WASM source mapと
host providerを持つ一般Effect ABIは未対応で、到達点3や#814／#815の全体完了とは分ける。Effect instanceの
非同期disposeも完了を待つ。通常のtab終了ではbrowserが
非同期処理の完了を待つ保証はなく、制御したwatch reloadと同じ終了保証を主張しない。

## 共有frontend

`seseragi-driver::compile_core_module`はTS lowering / emissionを呼ばない。
既存TSの`compile_module`とprojectのchecked-module loweringも同じCoreCompilation経路を使う。
`CompiledModule`の既存TS APIは互換性を維持する。backend間で一つの巨大な生成物structを共有せず、
Rust側は`RustArtifact`、`RustMetadata`、`RustBackendError`を返す。

## 値、closure、メモリ

pure ABI名は`rust-erased-pure-v1`、同期Effectは`rust-erased-sync-v2`、
Fiber / Deferredを使う生成物は`rust-erased-cooperative-v3`、FileSystemを使う生成物は
`rust-erased-async-v4`。型消去した共通値`V`を使い、generic / HKTをRustのgenericsで
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
full scanを512 checkpointごととroot終了時に行う。pending file I/Oのcaptureはruntime thread側のtaskがrootとして保持する。停止時間の上限は未実装。

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
nativeのInt除算はcanonicalな`std/int::Div`の型・evidence・source段階を照合する。
商は0方向へ切り捨て、保存したmethodや再利用する部分適用でも同じsafe integer範囲を保つ。
0除算はDefectとなり、Effectのresource scopeを閉じてからprocessを終了する。
Rem／Powやprogram WASMのInt算術は、この対応には含まない。
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
fileName / extensionと、`std/fs`のreadTextUtf8 / writeTextUtf8をnative file workerで扱う。
write modeはReplace / CreateNew / Append。Pathはcanonical ownerを持つopaque値として保持し、
String aliasへ置き換えない。必要なPath・filesystem・UTF8エラーの選択済みShow / Debugと、
`std/effect.fromEither`のfailure / success型を照合する。
小さいファイルを全量で読み書きし、handleはoperationが所有する。成功・失敗とも一度closeし、
read / write / flushの失敗をcloseの失敗で上書きしない。成功後のclose失敗は型付き失敗になる。
Effectの作成はI/Oせず、同じEffectを実行するたびにI/Oを行う。
この範囲はFileSystemの標準host実装に限る。packageの明示filesystem provider選択やprovider artifactを含む構成は出力前に拒否し、
JavaScript providerを黙ってnative hostへ差し替えない。全FileSystem API、
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

readTextUtf8 / writeTextUtf8はownedなrequestとresultだけをworkerへ渡し、runtime threadのtaskを中断する。
他のFiberは進行を続け、hostの完了通知後にworkerをjoinして同じtaskを再開する。handleはworkerが
成功・typed failure・Defect・取消のいずれでも一度closeする。open / read / write / flushに存在しない
abortを作らず、実行中のOS呼出しを待ち、chunk間で取消を確認する。取消後の遅い結果を成功・失敗として
公開せず、closeのDefectはcleanupとして保持する。scopeとinterruptはこのdrain・close・joinと
利用者のmasked finalizerが完了するまで待つ。

外部process shutdownのcooperative処理、強制OS abort、timer、race / parallel API、Queue / Stream等は
未対応。compiler WASMの生成TSで動くことと、一般program WASMの対応も分ける。

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

現在の未対応範囲：残るasync API、残るstdlib API、一般のpolymorphic global、再帰local function、
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
