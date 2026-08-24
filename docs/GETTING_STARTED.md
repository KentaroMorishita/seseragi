# Getting Started: local Web app

この手順は、SeseragiのWeb projectを作成し、VS Codeで編集し、browserで確認して、
production buildを作るまでの最短経路です。project固有のpackage managerや依存installは
必要ありません。

## 1. toolchainをinstallする

hostにはRustとBunが必要です。CLIをinstallし、versionを確認します。

```sh
cargo install \
  --git https://github.com/KentaroMorishita/seseragi \
  --locked \
  seseragi-cli

seseragi --version
```

VS Codeを使う場合は、GitHub ReleaseからOS / CPUに合う
`seseragi-v<version>-vscode-<platform>.vsix`を取得し、
**Extensions: Install from VSIX...** でinstallします。VSIXには対応するnative LSPが
同梱されるため、別のlanguage server installは不要です。

## 2. Web projectを作る

存在しないdestinationを指定します。package名には小文字ASCII、数字、途中の`-`が使えます。

```sh
seseragi new web hello-web
cd hello-web
```

生成されるのは通常のfilesystem packageです。

```text
hello-web/
├── seseragi.toml
├── seseragi.lock
└── src/
    ├── app.ssrg
    └── main.ssrg
```

`seseragi.toml`は通常のproject manifestで、`run.target = "web"`を持ちます。
sourceはPlaygroundのcanonical `web-starter` sampleと同一で、scaffold専用の別実装では
ありません。既存のfileやdirectoryは上書きしません。

`seseragi.lock`はpackage graph、toolchain database、Provider選択をexactに固定します。
manifestやpath dependencyを変更したときだけ、明示的に更新してcommitします。

```sh
seseragi lock update
```

`run` / `build` / `dev`はlockfileがない、またはmanifestとずれていると
`SES-K0102`で停止し、自動更新しません。`dev`中の通常のsource編集は許可しますが、
dependency graphやProvider要件が変わった場合は再度`seseragi lock update`が必要です。

## 3. VS Codeで開く

```sh
code .
```

`src/app.ssrg`を開くと、installed VSIXのLSPがmanifestと二つのmoduleを同じprojectとして
解析します。`StarterState`、`StarterAction`、`update`、Signal viewがapp側にあり、
`main.ssrg`はDOM targetへ接続します。

## 4. browserで開発する

package rootでdevelopment serverを起動します。

```sh
seseragi dev --open
```

既定では`http://127.0.0.1:3000/`を開きます。`src/**/*.ssrg`を保存するとrebuild後に
browserがreloadされます。compile error中は最後に成功した画面を維持し、修正すると自動で
復旧します。VS Codeからは **Seseragi: Start Development Server** でも同じCLIを起動できます。

## 5. production buildを作る

development serverを`Ctrl-C`で停止し、同じpackageをbuildします。

```sh
seseragi build .
```

自己完結した静的siteが`dist/`へ生成されます。`index.html`、bundled JavaScript、CSS、
source mapを含み、repositoryやPlaygroundを参照しません。VS Codeの
**Seseragi: Build Web App** も同じbuild commandを使います。

次は、[Runnable samples](../examples/samples/README.md)でより大きなSignal appを開くか、
[A Tour of Seseragi](https://seseragi.vercel.app/tour/)で言語機能を順に確認できます。
