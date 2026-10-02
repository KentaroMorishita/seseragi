# Signalの複数readとsnapshotの監査

2026-10-02。対象はSeseragi main `00c4d6b4015c0d9c99283cdf0f48d27700a23dd8`の
Signal/Effect実装と、Ishikoridome `0a3dc9f4914f0ba7fca8bf7e33280aa5ffada2a2`の
commandです。Ishikoridomeのコードは調査だけに使い、変更していません。

## 実アプリで必要な組

| 参照元（`seseragi/web/src/features/`配下） | 同じ判断へ渡す値 | 必要な扱い |
| --- | --- | --- |
| `design_session_commands.ssrg`のopen/apply | diagramとcontentRevision、編集中のstateとrevision | データと世代番号の組を一度に取得し、適用時のrevision検査も保持する |
| `selection_toolbar.ssrg`のcreateClusterCurrent | diagram、selected、viewport、customGroups、customClusterLimit | command入力のrecordとして一度に取得する。権限や制限の後続検査は別に必要 |
| `scene_camera.ssrg`のrefitForDocument | diagramとcamera state | fit計算へ渡す入力を一度に取得する |
| `design_export_command.ssrg` | 設計stateとdocumentName | export対象と名前を選ぶ時点を明確にする |

同じAction内にreadを並べるだけでは、この取得単位を定義できません。
`action.ssrg`のliftTaskはmapError/provideSomeでTaskを包みますが、snapshotを
固定する機構はありません。各commandで実際に不正な保存やexportが起きたという
本番障害の証拠と、今回のruntimeでの競合再現は区別します。

## 確認した意味

`Signal.read`はcoldなEffectで、実行時に`source.current()`を読みます。
`Effect.flatMap`のinterpreterは各bindの値をawaitしてから続きを実行します。
二つのsourceを0で開始し、最初のread後に両方を1へatomic更新すると、
二回のreadの結果は`{ a: 0, b: 1 }`になりました。更新自体のatomicityと、
複数readのsnapshot consistencyは別の条件です。

同じsourceを`combine`したderived Signalを一度読むと、結果は`{ a: 0, b: 0 }`です。
取得後に更新しても返されたrecordは変わらず、次のreadは`{ a: 1, b: 1 }`です。
subscriber内で次の更新をqueueした場合も、通知は`0/0`、`1/1`、`2/2`の順でした。
この動作をspec 5.14に明記し、実装に対応する回帰テストを追加しました。

derived Signalはsourceへの参照と計算関数を保持します。構成時に購読は始まらず、
未購読のderived Signalはsource更新だけでは再計算されません。
次に読むと最新の公開値で計算します。購読していない一時的なprojectionを捨てる際に、
unsubscribeは不要です。参照保持の量と回収時期はhostのGCにも依存します。

## コストの観測

Bun 1.3.9、darwin-arm64で5つのsourceを4つのcombineでまとめました。
更新のない10万回のreadを、warmup後に5回計測しています。
既存projectionの再利用は1.38〜10.15ms、一回ごとの再構成は164.00〜290.87msでした。
後者は各回に4つのderived nodeを構成します。最終projectionの評価は、
再利用時は各測定で1回、再構成時は10万回でした。両経路のchecksumは10万で一致しました。

これは小さい値を読む局所計測です。GCのheap量、UI latency、全アプリの負荷や
更新を含む性能を証明する数値ではありません。コストを理由にruntime primitiveを
増やす根拠は、この計測からは得られていません。

## 判断と推奨する形

commandが必要とするread-only recordをfeature内のpure関数で組み立て、
`combine`または既存Applicativeから作ったSignalを一度readします。
MutableSignalはfeature内に保持でき、command inputへ公開する必要はありません。
同じ入力集合を頻繁に使うcommandはprojectionを再利用し、単発なら一時構成します。

| 候補 | 判断 |
| --- | --- |
| 既存combine/Applicative + single read | 採用。必要な取得単位を明示でき、購読の常設は不要 |
| feature内のrecord projection helper | 用途に応じて採用。domainのfield名と必要なSignalをそのfeatureが所有する |
| 標準sample/snapshot helper | 今回は追加しない。既存readとの意味の差と、共通化すべき固定引数形が確定していない |
| Effect全体を固定するruntime primitive | 今回は追加しない。非同期処理を含む排他や古い値の扱いは別の意味論になる |

#610は監査として完了可能です。今後、既存の単一readで表せない実アプリの取得条件や、
測定で確認した高コストが見つかった場合に、対象を限定したwork itemへ昇格させます。
今回の競合再現を、Ishikoridome全commandの修正・本番確認済みとは扱いません。

## 再検証

`bun test runtime/ts/tests/signal-snapshot.test.ts`で4 tests / 13 assertionsが成功しました。
複数read間のcommit、単一snapshotの保持、未購読projectionの遅延評価、
nested transactionの通知順を検証します。同期の直接呼出しだけでなく、
実際のEffect.flatMap/runを使ってread間の再開を確認しています。

コストの再計測はこのdirectoryの`cost.ts`をBunで実行します。
閾値を設けた性能gateではなく、同じ処理を再現するための観測用scriptです。
