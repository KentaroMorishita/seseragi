このsampleを選ぶ理由: 一つのlarge Action unionへ全部を集めず、featureが
自分のstate、Action、update、event adapterを持ったまま、app shellで
一つのapplicationを作る読み方を試したいときに選びます。Previewは
releaseを整える一つのRelease Roomであり、Focus rhythmとStory deckは
ただ隣にあるwidgetではなく、同じheroへ現在の数を報告します。

## まずExplorerで読む順番

1. `app.ssrg` を開きます。ここはshellのvisual state、fixed image / document
   linkのURL検証、二つのfeature生成、`<$>` / `<*>`によるSignal合成だけを持ちます。
2. `focus/model.ssrg` を開きます。Focus rhythmはprivateな`FocusState`と
   `FocusAction`を持ち、view Signalとblock数のsummary Signalを同時に返します。
3. `notes/model.ssrg` を開きます。Story deckはform draft、validation、cardの
   add / edit / remove / clearを自分の`NotesState`へ閉じ込めます。
4. `notes/form.ssrg` と `notes/view.ssrg` を開きます。modelがbrowser eventの
   snapshotをTaskへ変換し、view側はpresentationalなform / card listだけを受け取ります。
5. 最後に `ui/styles.ssrg` と `ui/components.ssrg` を開きます。`cx`、named
   style、metric、headingをshared UIとして一箇所に置いています。

## 画面とmoduleの対応

```text
Release Room page (`app.ssrg`)
├─ app shell hero + derived metrics
│  ├─ `ui/styles.ssrg`       page / hero / accent styles
│  └─ `ui/components.ssrg`   eyebrow / metric / heading
├─ Focus rhythm              `focus/model.ssrg` + `focus/view.ssrg`
│  └─ blocksのSignalとAdd / Remove Action
└─ Story deck                `notes/model.ssrg` + `notes/form.ssrg` + `notes/view.ssrg`
   └─ draft、validation、cardのAdd / Edit / Remove / Clear Action
```

`app.ssrg`はfeature viewとcount Signalを受け、`<$>` / `<*>`でheroと二つの
feature surfaceを一つのpageへ合成します。feature間でstateやActionを渡さないので、
Previewの二つの領域とExplorerのmodule境界がそのまま対応します。

`main.ssrg` はfeatureを知りません。`create ()`で完成済みのcontent Signalを受け、
`dom.query`、`dom.defaultOptions ()`、`dom.run`だけを実行します。このsampleで
`dom.app`を使わないのは、複数のfeature-owned MutableSignal、各featureのeffectful
dispatch、app-levelのApplicative composition、そしてExplorerで追えるmodule境界を
明示する必要があるためです。

## Previewで試すこと

- `Add a focus block` と `Take one away` はFocus rhythmだけを更新します。heroの
  Focus metricも同じSignalから更新されます。
- Story card titleを空のまま送信するとvalidationが出ます。titleを入力して
  `Add a story card`を押すと、heroのStories metricとdeckが同時に変わります。
- card titleはinline editでき、`Remove`と`Clear deck`でempty stateまで到達します。
- `Use day studio` / `Use night studio`はapp shellだけが所有するvisual stateです。
  page background、hero surface、accent、text contrastをまとめて切り替えます。

固定Unsplash imageにはalt、width、height、cropを指定し、外部document linkには
`parseWebUrl`の検証結果だけを渡します。class utilityは長いliteralへ戻さず、
`ui/styles.ssrg`の`cx [...]`とnamed `html.Style`を使います。

pure reducerだけの最小構成は`interactive-app`、同じappの明示的runtime接続は
`signal-run-route`、single-fileでformと多様なeventを確認する完成例は`form-todo`です。
このRelease Roomは、その次にExplorerを開いて「featureの境界が画面の境界とどう対応するか」を
読むためのmulti-module Showcaseです。
