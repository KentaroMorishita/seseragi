このsampleを選ぶ理由: interactive stateを入れず、tag、props、children、関数component、link / image、SSRだけを順に確認したいときの出発点です。

関数をcomponentとしてchildrenから呼び、長いutility列は自己完結した`cx` helperとnamed classへ、固有の視覚値はnamed `html.Style`へ切り出します。`header` / `nav` / `article` / link / image / list / text semantics / void tagで文書構造を組み立て、escaped HTMLをSSR previewへ渡します。固定HTTPS画像とrepository linkは`parseWebUrl`でopaqueな`WebUrl`へ変換してから`src` / `href`へ渡します。画像は意味のある`alt`と幅・高さで領域を予約し、PreviewのCSP内で読み込みます。

次は`interactive-app`で、同じcomponentの考え方をState / Action / pure `update` / `view`と`dom.app`へ広げます。Signalの生成、custom options、複数featureの合成を自分で制御したい場合は`feature-composition`の明示的`dom.run`を選びます。
