# Either / sum 作業の停止時点

ユーザーの「切りいいところで作業とめて。スケジュールも一旦停止で」により、
2026-10-30に実装を中断した。`seseragi-rust`は`PAUSED`を永続設定で確認済み。
再開依頼が来るまで実装を進めない。実装の再開とスケジュールの再開は別に扱う。

## 保存先と完了した範囲

- branch: `codex/native-sum-traversal-20261030` / Draft PR #1149。
- Maybe #1045と共有compiler修復 #1146/#1147はcommit
  `2be8b601fe5a1a588e81404c5a6d3f4497c226e1`で実装・検証・push済み。3 Issueを閉鎖した。
- 親126の正規SDK採用とmainのregular mergeを
  `8f293765146b6913d20debb4004a08cfa6e90d3b`まで取り込んだ。
- 以下のEitherと共有修復は未完了の作業保存。#1046/#1150/#1151/#1152を閉じていない。
  SDK・全体gate・配布は #1148で継続する。

## Eitherの現在地

`mapLeft/mapRight/bimap/fold/swap/sequence/traverse`のnative emitter、runtime、
型・owner・kind・辞書・source stageの拒否検査を追加した。標準Array/Listと
独自・輸入・parameter Traversableを再利用する。最初のLeftを保ち、TS同様に
後続callbackも評価する。NonEmptyList providerは #1048/#1101まで拒否する。

sourceを固定して見つかった既存compiler不具合:

- #1150: 未推論parameterを表示型のHoleへmaskした後も古いsemantic keyが残る。
  canonical ownerを先に投影し、変更した期待型のkeyも合わせる修復を実装した。
  元のHKT最小sourceはTS両profile・fresh Bun/Node/nativeで42/雪を出力済み。
- #1151: Eitherのunion/overloadに対するhost推論がerrorと成功値を混同する。
  標準runtime callへのcheckedな型引数を実装したが、通常の制約付きgeneric関数の
  第1callbackと保存partialの修復が残る。
- #1152: sourceの`Error`が生成guardの`new Error`を隠す。
  guardを`globalThis.Error`へ変更し、sourceの`globalThis` bindingには衝突しない
  backend名を使う修復を実装した。元Error最小sourceとglobalThis/record fieldの対照は
  両profile・fresh Bun/Node/nativeで42を出力済み。generated artifact同期は残る。

1150/1151/1152はすべて配布118でも再現した既存不具合。`runtime/ts/src/sum.ts`は
配布118からbyte不変であり、runtimeの型定義やassertionを緩めてはいない。

## 検証結果

| 確認 | 停止時点の結果 |
| --- | --- |
| runtime `either_operation_tests` | 4件成功、5.92秒。callback順序、後続Defect、第一Left、選択edge、捕捉Ref循環、10万要素List/root0 |
| driver | 24件成功。`either_runtime_types`、`hkt_partial_sum_context`、alias/HKT/generic/strict partial、同名nominal拒否 |
| backend | Either4＋HKT2＋Maybe5の11件成功。Either偽造128件・unused program WASM7件を拒否 |
| CLI `basic_either_operations_` | **4成功・3失敗**、107.90秒。完了扱いにしない |
| 所有process | 上記の全runで`remainingPids: []`を確認。local verificationは停止済み |

失敗しているCLI3件:

1. `basic_either_operations_keep_original_selected_branches_traversals_and_returned_functions`
   の元raw TS strict。通常の`transform choose ...`でcallbackのerror StringがIntと混同される。
2. `basic_either_operations_path_package_preserves_opaque_error_element_hkt_alias_and_dictionaries`
   の元raw TS strict。保存した`values.transform values.increment`でErrorとValueが混同される。
3. `basic_either_operations_source_callback_and_returned_stage_defects_release_every_root`
   の派生入力。検証用の未使用`Right [1]`のerror型が未確定でnative kind guardに拒否される。
   派生式へ具体的な`Either<String,Array<Int>>`型を与える必要がある。
   元fixtureと期待出力は変えず、kind guardを緩めない。

再開時の切り口: `typescript/expr.rs`の`check_late_callback`は`skip(1)`で第1callbackを
除外し、deferred partialも除外する。`lower_constrained_call`のpartial initializerを
source checkerが確定したcallable型で検査する必要がある。
辞書は元のsource arityの後、返る関数の引数の前に置く。calleeと供給済み引数を
partial作成時に一度評価する契約を保つ。

## 元入力の固定

以下のsource/期待値は停止時までhash不変。opaque2moduleの元sourceとlockも保存する。

- `basic-either-operations.ssrg/.stdout`（29行出力）。
- `hkt-partial-sum-context.ssrg`: `bbfdc9685d195266852146bcc60ddb9996d65a90a57561f2ccd521d8ff9cef8e`
- `either-saved-runtime-types.ssrg`: `9d46295308094d603815e37159cb3e78ed5a1827681135299416e8603e5e9e18`
- `host-error-shadow.ssrg`: `1c67cbd254e86c9d1888ca816c3a72f6817c9483770dab85087dd06cc065dd19`
- `host-global-this-binding.ssrg`: `0f0b4b955c2407e8f4b89dd1f9ccc3539c91f3607823f6bed9f45976780a11e3`

host guardの修復に伴い、committed artifactの同期も必要。停止時点では変更していない。
49個の`main.ts`に旧`new Error`がある。対応するcanonical writerを使い、generated
TS/source mapと必要なTypeScriptIrだけを同期する。入力SSRG・実行期待値・診断・型の
assertionを緩めず、他stageのhash/差分も確認する。

## 待機中の統合・配布

- 124: PR #1128はmain `87d5ad93e78e1d2d50694fcdfb035cbee51c8b14`へregular merge済み。
  全6統合gate成功。正規Release [41038774664](https://github.com/KentaroMorishita/seseragi/actions/runs/41038774664)
  は10:52 UTC時点でrelease source gate実行中。#1129の公開受入/dogfood/実配布検査は未完了。
- 125: head `5712becd9de71ee858014e59d4a68178553df781`、PR #1130のbaseはmainへ更新済み。
  [41023378375](https://github.com/KentaroMorishita/seseragi/actions/runs/41023378375)のsource gateが
  10:52 UTC時点で実行中、他5 gateは成功。#1131を継続。
- 126: head `396aa16af48802e4e12ba6ef585abef3a7a8483d` / PR #1143。
  producer41033043807/source1bd9b34f0の全3 job・全6 fileを照合してSDK採用済み。
  fresh Bun/Node各positive103/negative40/struct8+14/raw strict156/Analysis56と
  release:checkが成功。WASM SHA256は
  `6de49b2c1775141e3c415b0fe9d5c710d109b1f9c6c07f80c9c84c317c31a3a7`。
  [41041225786](https://github.com/KentaroMorishita/seseragi/actions/runs/41041225786)を一度dispatch済み。
  10:52 UTC時点でcanonical-wasm成功、他は実行中/queued。#1144を継続。

必要な上記CI/正規ReleaseはGitHub側で継続し、古いchild PRの自動CIは停止のため整理する。
同じheadを再dispatchしない。再開時は必ずlive状態・最新headの既存review/inline/securityを
確認する。securityのHTTP404/no analysis found/不足scopeをclean scanと数えない。
#934に従い新規hosted scan/reviewを依頼しない。

checkoutは1個、stashなし。Cargoは`target/sequence118`を再利用しincremental/debugを
無効化している。完了済みbinary約185MiBと古いworkspace library約2.74GiBは回収済み。
停止前の計測はprofile約4.5GiB、Data空き約165GiB。現行cacheと配布124の必要な受入harnessは
再開・配布検査に必要なので保持する。外部archiveや追加worktreeは作っていない。
