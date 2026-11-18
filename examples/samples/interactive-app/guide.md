このsampleを選ぶ理由: 一つのStateとpure reducerで完結する小さなTrail plannerを、Signal作成やDOM runtime接続を手書きせず`dom.app`で動かしたいときに選びます。

三つのTrail state、typed Action、pure `update`、HTML `view`を一画面へ置きます。Actionを選ぶと見出し、説明、所要時間、accent、progressが同時に変わります。長いutility列はsource内の`cx`と役割名を持つclass valueへ分け、動的な色と進捗幅はnamed `html.Style`へ置いています。

森林のfixed Unsplash imageはroute選択の文脈を作り、mobileでは三つのactionが一列、desktopでは三列へ切り替わります。

`dom.app`は内部で`MutableSignal`作成、`signals.map view`、target query、default options、`dom.run`、Actionからpure `update`へのdispatchを所有します。

pure reducerだけで足りる場合はこの版を選びます。明示的な`signals.make`と`dom.run`の最小構成を見たい場合は`web-starter`、複数のfeature-owned Signalやcustom実行境界を扱う場合は`feature-composition`、formと複数eventの完成例は`form-todo`を確認します。
