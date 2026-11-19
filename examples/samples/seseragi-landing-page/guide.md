このサンプルを選ぶ理由: Seseragi だけでランディングページ全体を組み立て、見た目の区切りごとに分けた複数ファイルの構成とシンタックスハイライト付きコード表示を一緒に読みたいときに選びます。

## この画面で試すこと

- ファーストビューの Playground・Tour・GitHub の各リンクをキーボードやクリックで開く
- `Three ways to see it` の `Readable` / `Composable` / `Alive` タブを切り替える
- タブに応じて見出し・説明文・強調色・Playground と同じ配色のシンタックスハイライト付きコードが `Signal` 経由で差し替わることを確認する
- デスクトップとスマートフォン幅で、ファーストビューからフッターまでの流れを比べる
- Explorer から画面の区切りとソースファイルの対応を追う

## ファイル構成

```text
main.ssrg                    実行の入口（dom.run）
app.ssrg                     URL 準備とページ全体の合成
url.ssrg                     文字列から WebUrl への変換
components/header.ssrg       ロゴと主要リンク
components/hero.ssrg         ファーストビューとコードカード
components/principles.ssrg   3 つの設計方針
components/chapter/          タブ状態・コードのトークン色分け・表示
components/image-statement.ssrg
components/closing.ssrg      締めくくりと行動リンク
components/footer.ssrg       フッター
ui/styles.ssrg               共通の配色と名前付きスタイル
```

`components/chapter/model.ssrg` は選択中タブの `Chapter` 状態だけを `MutableSignal` で持ち、外には `Signal<Html<Task<Unit>>>` だけを返します。`components/chapter/code-panel.ssrg` は Seseragi コードをトークン種別ごとの ADT で表し、Playground のダークテーマと同じ配色で色分け描画します。
