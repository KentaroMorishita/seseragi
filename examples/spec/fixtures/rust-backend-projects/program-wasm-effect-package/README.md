# Effect program WASMとTS生成UI

標準の`web.programs`から計算moduleをprogram WASMへ、画面をTSへ生成する受入fixture。
計算moduleはproviderなしのcold Effectだけを使う。bridgeは生成loaderの型を使い、
foreign taskの中で明示的にrunする。描画とUI stateはSeseragiに置く。

```sh
seseragi build examples/spec/fixtures/rust-backend-projects/program-wasm-effect-package \
  --target web --profile development --out-dir target/effect-program-site
```

| 操作 | 画面で確認する結果 |
| --- | --- |
| planを作る → run／再実行 | 作成時は`running:false`。同じplanを繰り返しrunして`Success`を得る。 |
| 型付き失敗 | `Failure`のStringを表示する。 |
| Defect | `Defect`を表示し、instanceは`alive:true`、`retired:false`を保つ。 |
| finalizerのDefect | `inner released / outer released`の順に表示する。 |
| callbackを受け取る → 呼ぶ | Effectの成功値で受け取った関数が捕捉した文字列を使う。 |
| callbackを解放 → 呼ぶ | detach後の呼出しを`ProgramWasmClosed`として拒否する。 |
| 継続するEffectを開始 → cancel | UIからcancelし、終了後に`Cancelled`を表示する。 |
| 解放を報告するEffectを開始 → instanceを解放 | `await dispose()`後に`cancel released`のDefectと所有数0を表示する。 |
| 再生成 → run | 旧instanceの解放を待ち、新しいgenerationで実行できる。 |

正常なinstance解放では`callbacks:0`、`plans:0`、`running:false`、
`managedLiveAtClose:0`をbridgeが検査する。UI finalizerは保存callbackを使ってから
instanceを閉じる。標準watch reloadはUI finalizerとprogram所有者の終了を待つ。
通常のtab終了でbrowserが非同期cleanupを待つとは保証しない。

計算moduleは既存の`program-wasm-effects.ssrg`と同じ入力を使う。
一般I/O、Ref、provider、WASMへ渡すhost callback、一般の非同期program WASMは含めない。
このfixtureの作成と、正式配布・公開browserでの受入完了は別に記録する。
