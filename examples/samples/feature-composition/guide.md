このサンプルを選ぶ理由: 画面の状態を一つの大きな型に集めず、機能ごとに自分の Signal を持たせて一つの画面に組み合わせる形を見たいときに選びます。

トマトとバジルのスパゲッティを作る台所です。分量・手順・タイマーの 3 つの機能が、それぞれ自分の状態を持っています。手順の「タイマーをかける」を押すと、タイマーが時間を刻み始めます。

## この画面で試すこと

- 材料の「+」を押して 3 人分にし、量だけが変わるのを見る
- 手順 4 の「8 分のタイマーをかける」を押し、タイマーが 8:00 から減っていくのを見る。そのあいだに人数を変えたり手順を済みにしたりしても、タイマーは止まりません
- 「一時停止」「+1 分」「リセット」を押し、タイマーの状態だけが変わるのを見る

## コードの読みどころ

1. `app.ssrg` の `create`: 3 つの機能を作り、描画の Signal を `<$>` と `<*>` で一つの画面にまとめます。組み立て側は、どの機能の状態にも触れません。
2. `features/servings.ssrg`: 人数を `MutableSignal<Int>` に持ち、外へは描画の `Signal` だけを返します。
3. `features/steps.ssrg`: 済んだ手順を `Set<Int>` で持ちます。タイマーの状態は知らず、引数で受け取った「タイマーをかける」操作だけを使います。
4. `features/timer.ssrg` の `create` と `step`: 残り時間の Signal と、それを進める Fiber を `effects.fork` で起こします。`dom.run` は操作を一つずつ処理するので、時間の経過は操作ではなく別の Fiber で進めます。Fiber は 250 ミリ秒ごとに `clock.now` を読み、前に読んだ時刻との差を `time.durationBetween` で測って、その分だけ残り時間を減らします。一時停止や再開などの操作も、押した時刻までの経過を反映してから状態を変えます。ブラウザーが裏のタブで刻みを間引いても、タイマーは遅れません。
5. `kitchen/countdown.ssrg` と `kitchen/recipe.ssrg`: タイマーの状態遷移と分量の計算は、画面と切り離した純粋な関数です。`seseragi test examples/samples/feature-composition` で、画面なしに確かめられます。

## ファイル構成

```text
main.ssrg               #app を探して dom.run に渡す入口
app.ssrg                3 つの機能を作って並べる
features/servings.ssrg  分量の換算（人数の Signal）
features/steps.ssrg     手順のチェック（済んだ番号の Signal）
features/timer.ssrg     タイマー（残り時間の Signal と刻みの Fiber）
kitchen/recipe.ssrg     材料と手順のデータ、分量の計算
kitchen/countdown.ssrg  タイマーの状態と遷移
ui/styles.ssrg          台所の色の名前付き style
tests/kitchen.ssrg      分量とタイマーの単体テスト
```

一つの状態を Signal に持ついちばん小さな形は `interactive-app`、複数の画面を持つアプリは `project-flow-app` を見てください。
