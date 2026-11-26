このサンプルを選ぶ理由: 画面全体の状態を 1 つの大きな型へ集約せず、機能ごとに独立した `MutableSignal` を所有して `dom.run` で合成する設計を見たいときに選びます。

公開準備ボードの 3 つの確認カードがそれぞれ自分専用の `MutableSignal` を持ち、親のレイアウト制御からは `Signal<html.Html<Task<Unit>>>` として合成します。

## この画面で試すこと

- 「下書き確認」で「確認を 1 件進める」を何度か押してから「先頭カードを隠す」→「先頭カードを再表示する」を押し、確認数がそのまま残るのを確かめる
- 「カードの順番を入れ替える」を押し、位置が入れ替わっても確認数がカードごとに独立して保たれるのを見る
- 「先頭を最終チェックへ差し替える」で別のカードへ切り替え、さらに戻しても元の進捗が消えないことを確かめる

## コードの読みどころ

1. `createChecklistCard`: 呼び出された時点でカード専用の `MutableSignal<CheckState>` を一度だけ作り、外へは描画結果の `Signal<html.Html<Task<Unit>>>` だけを返します。
2. `LayoutState` と `selectFirstSlot`: 親は表示・順序・差し替えだけを管理し、子の `CheckState` や `CheckAction` には一切触れません。
3. `signals.switchMap`: 表示する分岐を動的に切り替えます。非表示中もカードの Signal を作り直さないため状態が維持されます。
4. `pageView <$> layoutView <*> panel <*> activeFirst <*> second`: 複数の独立した Signal を 1 つの画面 Signal に合成し、`mount` から `dom.run` へ渡します。
