このsampleを選ぶ理由: interactive stateを入れず、tag、props、children、関数component、link / image、SSRだけを順に確認したいときの出発点です。

関数をcomponentとしてchildrenから呼び、長いutility列は自己完結した`cx` helperとnamed classへ、固有の視覚値はnamed `html.Style`へ切り出します。`header` / `nav` / `article` / link / image / list / text semantics / void tagで文書構造を組み立て、escaped HTMLをSSR previewへ渡します。固定HTTPS画像とrepository linkは`parseWebUrl`でopaqueな`WebUrl`へ変換してから`src` / `href`へ渡します。画像は意味のある`alt`と幅・高さで領域を予約し、PreviewのCSP内で読み込みます。

暖色のdocument surfaceはStatic HTML / SSRの入口であることを示し、headerはmobileで自然にwrapし、desktopではtitleとsource linkを同じ行に保ちます。image、copy、listを一枚のreusable cardへ閉じるので、表示のためのstructureとcomponentの責務を一緒に追えます。

次は`interactive-app`で、同じcomponentの考え方をState / Action / pure `update` / `view`と`dom.app`へ広げます。同じappのSignal生成、query、options、runを自分で制御する最小例は`signal-run-route`、複数featureの合成は`feature-composition`を選びます。
