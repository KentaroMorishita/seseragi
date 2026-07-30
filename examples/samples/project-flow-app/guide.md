このsampleは、interactive Web Appを六つのfileへ分割したbrowser projectです。

- `main.ssrg`: projectのentry
- `app.ssrg`: DOM runtimeと各featureを合成するapp shell
- `counter.ssrg`: 独立したSignalを所有するcounter feature
- `todo/feature.ssrg`: Todoのstateとupdate
- `todo/form.ssrg` / `todo/list.ssrg`: Todo viewの部品

Explorerでfeatureを編集してからRunすると、workspace全体がcompileされます。diagnosticを選ぶと、問題のあるfileと範囲へ移動できます。
