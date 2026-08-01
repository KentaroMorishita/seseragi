このsampleを選ぶ理由: 一つのStateとpure reducerで完結する小さなTrail plannerを、Signal作成やDOM runtime接続を手書きせず`dom.app`で動かしたいときに選びます。

三つのTrail state、typed Action、pure `update`、HTML `view`を一画面へ置きます。Actionを選ぶと見出し、説明、所要時間、accent、progressが同時に変わります。長いutility列はsource内の`cx`と役割名を持つclass valueへ分け、動的な色と進捗幅はnamed `html.Style`へ置いています。

`dom.app`は内部で`MutableSignal`作成、`signals.map view`、target query、default options、`dom.run`、Actionからpure `update`へのdispatchを所有します。sourceの`// Runtime boundary`より前は比較先の`signal-run-route`と同一で、初期HTMLと全Action後HTMLも一致します。

pure reducerだけで足りる場合はこの版を選びます。effectful handler、custom options、mount lifecycle、複数のfeature-owned Signalが必要なら、比較先の`signal-run-route`で同じappの明示的`signals.make` / `dom.query` / `dom.run`を確認します。複数featureの合成は`feature-composition`、formと複数eventの完成例は`form-todo`です。
