関数をcomponentとしてchildrenから呼び、`html.Style`をtop-level値へ切り出して再利用します。`header` / `nav` / `article` / link / image / list / text semantics / void tagで文書構造を組み立て、escaped HTMLをSSR previewへ渡します。
