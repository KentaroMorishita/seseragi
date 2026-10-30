# はじめての Web アプリ

Seseragi で Web アプリのプロジェクトを作り、VS Code で編集して、ブラウザで動作を確かめ、
本番用にビルドするまでの手順です。npm などのパッケージマネージャや、プロジェクトごとの依存関係のインストールは要りません。

## 1. ツールをインストールする

必要なのは Seseragi の CLI と [Bun](https://bun.sh/) の2つです。

- **CLI**: [GitHub Releases](https://github.com/KentaroMorishita/seseragi/releases) から OS と CPU に合ったアーカイブをダウンロードし、
  中の `seseragi` を PATH の通った場所に置きます。
- **Bun**: [公式のインストール手順](https://bun.sh/docs/installation)に従ってインストールします。

Rust は要りません（CLI をソースからビルドする場合だけ必要です）。
インストールできたら、バージョンが表示されることを確認します。

```sh
seseragi --version
bun --version
```

VS Code を使う場合は、同じ GitHub Releases から OS と CPU に合った
`seseragi-v<version>-vscode-<platform>.vsix` をダウンロードし、
コマンドパレットの **Extensions: Install from VSIX...** でインストールします。
LSP は VSIX に入っているので、別途インストールする必要はありません。

## 2. プロジェクトを作る

まだ存在しないディレクトリ名を指定します。
名前には小文字の英字、数字、`-`（先頭と末尾以外）が使えます。

```sh
seseragi new web hello-web
cd hello-web
```

次のファイルが作られます。

```text
hello-web/
├── seseragi.toml   # プロジェクトの設定
├── seseragi.lock   # 依存関係を固定するファイル
└── src/
    ├── app.ssrg    # アプリ本体（状態・更新・画面）
    └── main.ssrg   # ブラウザの DOM につなぐ入口
```

中身は Playground のサンプル `web-starter` と同じです。
既にあるファイルやディレクトリを上書きすることはありません。

### seseragi.lock について

`seseragi.lock` は、依存しているパッケージや使うランタイムを固定するファイルです。
Git にコミットしてください。

`seseragi.toml` や依存関係を変えたときは、次のコマンドで更新します。
自動では更新されません。

```sh
seseragi lock update
```

lock ファイルがない場合や設定と食い違っている場合、`run` / `build` / `dev` は
エラー `SES-K0102` で止まります。そのときは上のコマンドを実行してください。
`dev` の実行中にソースを編集するだけなら、更新は要りません。

### タイトルや favicon を変えたいとき

`seseragi.toml` に次を追加すると、自分で用意した HTML と静的ファイルを使えます。

```toml
[web]
index = "web/index.html"
public = "public"
```

`web/index.html` には、次の2つの目印を1つずつ置きます。

- `<head>` の中に `<!-- seseragi:head -->`（CSS が入ります）
- `<div id="app"></div>` の後に `<!-- seseragi:entry -->`（JavaScript が入ります）

`public/` に置いたファイル（例: `public/images/favicon.svg`）は、同じパスで配信されます。
設定を追加したら `seseragi lock update` を実行してください。
詳しくは [言語仕様の該当節](spec/11-packages-and-projects.md#web-documentとpublic-assets) を参照してください。

## 3. VS Code で開く

```sh
code .
```

`src/app.ssrg` を開くと、補完・ホバー・エラー表示が使えます。
アプリの状態（`StarterState`）、操作（`StarterAction`）、状態を更新する `update`、
画面を作る Signal は `app.ssrg` にあり、`main.ssrg` がそれをブラウザに表示します。

## 4. ブラウザで確認しながら開発する

プロジェクトのディレクトリで開発サーバーを起動します。

```sh
seseragi dev --open
```

ブラウザで `http://127.0.0.1:3000/` が開きます。
`src/` のファイルを保存すると、自動でビルドし直してページを再読み込みします。
コンパイルエラーがある間は最後にビルドできた画面のままで、直すと元に戻ります。

VS Code のコマンド **Seseragi: Start Development Server** からも起動できます。

## 5. 本番用にビルドする

開発サーバーを `Ctrl-C` で止めてから、ビルドします。

```sh
seseragi build .
```

`dist/` に、そのまま公開できる静的サイトが出力されます。
`index.html`、JavaScript、CSS、ソースマップが入っています。
VS Code のコマンド **Seseragi: Build Web App** でも同じことができます。

## 次のステップ

- [サンプル集](../examples/samples/README.md) — もっと大きな Signal アプリの例があります
- [A Tour of Seseragi](https://seseragi.vercel.app/tour/) — 言語の機能を順番に学べます
