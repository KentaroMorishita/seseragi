このsampleは、interactive Web Appを七つのfileへ分割したbrowser projectです。

- `main.ssrg`: projectのentry
- `styles.ssrg`: `cx`を公開する共通style helper
- `app.ssrg`: DOM runtimeと各featureを合成するapp shell
- `counter.ssrg`: 独立したSignalを所有するcounter feature
- `todo/feature.ssrg`: Todoのstateとupdate
- `todo/form.ssrg` / `todo/list.ssrg`: Todo viewの部品

各view moduleは長いutility列を役割名を持つ縦の`cx [...]`へ分け、Explorerから`styles.ssrg`の実装へ辿れます。Explorerでfeatureを編集してからRunすると、workspace全体がcompileされます。diagnosticを選ぶと、問題のあるfileと範囲へ移動できます。
