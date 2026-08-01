このsampleを選ぶ理由: pure reducerで完結する小さなapplicationを、Signalの生成やmount lifecycleを手書きせず最短の`dom.app` contractで動かしたいときに選びます。

pure reducer、typed Action、Signal、HTML view、`dom.app`を一つの小さなFlow UIへ統合します。長いutility列はsource内の`cx`と役割名を持つclass valueへ分け、動的な色・進捗幅はnamed `html.Style`、画面構造はprogress / action group / card componentとして追える順序に置きます。Previewのbuttonを押すとstateとDOMが更新されます。

先に静的なprops / children / componentを確認する場合は`html-components`へ戻ります。effectful handler、custom `dom.defaultOptions`、複数のfeature-owned Signalが必要なら`feature-composition`の明示的`signals.make` / `dom.run`へ進みます。formと複数eventを統合した完成例は`form-todo`です。
