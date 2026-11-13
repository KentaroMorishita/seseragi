# はじめての CLI ツール

Seseragi でコマンドラインツールのプロジェクトを作り、実行し、テストし、
単体で動く実行ファイルにするまでの手順です。
ツールのインストールは [はじめての Web アプリ](./GETTING_STARTED.md#1-ツールをインストールする) と同じです。

## 1. プロジェクトを作る

まだ存在しないディレクトリ名を指定します。
名前には小文字の英字、数字、`-`（先頭と末尾以外）が使えます。

```sh
seseragi new cli hello
cd hello
```

次のファイルが作られます。

```text
hello/
├── seseragi.toml   # プロジェクトの設定（実行先は process）
├── seseragi.lock   # 依存関係を固定するファイル
├── src/
│   └── main.ssrg   # プログラム本体
└── tests/
    └── main.ssrg   # あいさつの文を確かめるテスト
```

中身は `examples/templates/cli` と同じです。
既にあるファイルやディレクトリを上書きすることはありません。

`src/main.ssrg` は、コマンドライン引数で受け取った名前にあいさつします。

- 引数がなければ `World` にあいさつします。
- 空の名前を渡すと、エラーを表示して終了コード 1 で終わります。

## 2. 実行する

```sh
seseragi run .
```

```text
Hello, World!
```

## 3. テストを実行する

```sh
seseragi test
```

```text
PASS main::greeting::greets by name
PASS main::greeting::rejects an empty name
2 passed; 0 failed; 0 skipped
```

`--target native` を付けると、同じテストをネイティブの実行ファイルで動かします。

## 4. 実行ファイルを作る

```sh
seseragi build . --target native
./dist/program Alice
```

```text
Hello, Alice!
```

`dist/program`（Windows では `program.exe`）は、Bun などを入れていない環境でも単体で動きます。
最初のビルドでは、ネイティブのビルドに使うツールを自動でダウンロードするため、時間がかかります。

## 5. 整形する

```sh
seseragi format
```

引数なしで、いまいるパッケージのソースをすべて整形します。
CI では `seseragi format --check` で、整形が必要なファイルがないかを確かめられます。

## 次のステップ

- `seseragi help` — コマンドの一覧と、それぞれの使い方を表示します
- [A Tour of Seseragi](https://seseragi.vercel.app/tour/) — 言語の機能を順番に学べます
