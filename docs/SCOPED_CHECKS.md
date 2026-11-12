# 変更に合わせて実行するチェック

コンパイラーやツールを変更して、push する前にどのチェックを実行すればよいかを決めるための文書です。
変更した範囲に合うチェックだけを先に実行します。
リポジトリ全体のチェック（`bun run check`）は、毎回の安全策として実行するものではありません。統合の前など、決まった場面でだけ実行します。

## 変更の種類 → 実行するチェック

| 変えたもの | 実行するチェック |
| --- | --- |
| `docs/` の文書だけ（`docs/spec/` を除く） | `git diff --check`。リンク先があることを確かめる |
| `README.md`、`README.ja.md` | `bun run check:readme` |
| サンプルのソースや情報（`examples/samples/`） | `bun run check:sample` |
| Playground の画面や Tour | `bun run check:playground` |
| Web UI の見た目 | `cd apps/playground && bun run test:visual`（[Web UI の見た目の確認](#web-ui-の見た目の確認)） |
| Rust のコード（コンパイラー、CLI、LSP） | `bun run check:rust`。1つのクレートだけなら `bun run check:rust -- -p <crate>` |
| コンパイラーの中間成果物の形 | 下の[各チェックの中身](#各チェックの中身)の「コンパイラーの成果物」 |
| fixture（`examples/spec/fixtures/`） | `bun run check:conformance`。対象のフォルダーを引数で絞れる |
| コンパイラー、実行時の約束、WASM のアダプター | `bun run build:playground:wasm` で WASM を作り直し、`bun run check:wasm` |
| `.d.ts` の変換 | `bun run check:dts:wasm` |
| native の標準 API | [native の標準 API のテスト](#native-の標準-api-のテスト)の該当する行 |
| プログラムの WASM | `cargo test -p seseragi-cli --test program_wasm` |
| VS Code の拡張機能 | `bun run check:extension` |
| 本番用のビルドの成果物 | `bun run check:production`。ブラウザーで動かすなら `bun run check:production:browser` |
| 版、配布物の名前、リリースの手順 | `bun run check:release` |
| 利用者に見える変更（`crates/`、`runtime/ts/`、`apps/playground/`、`examples/spec/`、`docs/spec/`、拡張機能） | 版を上げて CHANGELOG を書き、`bun run release:check`（[RELEASE.md](./RELEASE.md)） |

チェックが足りない変更は、必要なコマンドを個別に実行し、結果を PR に書きます。足りないチェックを加える作業は、別の Issue にします。

## リポジトリ全体のチェックを実行する場面

次のどれかに当てはまるときだけ、上のチェックに加えて `bun run check` を実行します。

- コンパイラー、実行時のライブラリ、WASM の土台を変更した
- 複数の領域にまたがる変更をした
- リリースの前に、全体がそろって動くかを確かめる
- #201 の作業の列を `main` へまとめて取り込む
- Issue の本文が、全体のチェックを求めている

実行したときは、必要だった理由と結果を PR に書きます。
長い間なにも出力しないときは、どの段階で止まっているか、プロセスが動いているかを確かめ、放っておきません。

## 準備

### 依存パッケージ

チェックは依存パッケージを自動でインストールしません。ロックファイルで決まった手元のプログラムだけを使います。
足りないときは、インストールのコマンドを表示して失敗します。
新しい worktree や CI のマシンでは、チェックの前に一度インストールします。

```sh
bun install --frozen-lockfile
cd apps/playground && bun install --frozen-lockfile
cd ../../extensions/seseragi && bun install --frozen-lockfile
```

- `check:sample` と `check:playground` は、コミットした WASM と Playground の TypeScript のツールを使うので、Playground のインストールが必要です。
- `check:conformance` は、実行時のパッケージを試す処理がルートの Node の型を使うので、ルートと Playground の両方が必要です。
- ロックファイルや依存を変えたときも、そのフォルダーで `--frozen-lockfile` のインストールを明示的に行います。
- 全体のチェックは、ルートと Playground を先にインストールし、拡張機能のパッケージを作るときにもインストールします。

### Bun の版

CI と手元で、Bun を **1.4.4** にそろえます。基準はルートの `.bun-version` で、GitHub Actions も同じファイルを読みます。
手元では、`bun --version` と `cat .bun-version` が同じことを確かめてから実行します。

- 1.3.9 では、Linux x64 の WASM で JavaScriptCore のクラッシュ（IPInt）が起きたため、[WebKit の更新 #26922](https://github.com/oven-sh/bun/pull/26922) を含む 1.3.11 にそろえていました。
- 1.4.4 で Brotli の展開の脆弱性（CVE-2026-111497）が直ったので、1.4.4 に上げました。
- WASM の実行やテストの確認を省いたり、JavaScriptCore の実行の段階を止めたりしません。
- 過去の文書にある古い版の表記は、そのときの記録として残します。

### Playground のテストで使うブラウザー

Playground の Bun のテストは、既定でロックファイルの Playwright に合う Chromium を使います。
インストール済みの Google Chrome を使うときは、その実行ファイルの絶対パスを `SESERAGI_TEST_BROWSER_PATH` に設定します。

```sh
cd apps/playground
SESERAGI_TEST_BROWSER_PATH=/path/to/google/chrome bun test tests
```

- 起動したブラウザーのパスと版をログに残します。指定したパスで起動できなければ失敗します。
- この設定は Bun のテストだけのものです。`test:visual` や本番用のブラウザーのチェックには効きません。

## 各チェックの中身

すべてのチェックは `scripts/check-scoped.sh` を共通の入口として使います。

| チェック | コマンド | 確かめること |
| --- | --- | --- |
| サンプル | `bun run check:sample` | native の CLI でサンプルを実行する。すべてのサンプルを WASM でコンパイルし、整形する。Playground のサンプル一覧が最新か |
| Playground | `bun run check:playground` | サンプルのチェック、Playground とカタログの lint、Tour の一覧、Playground のテスト、TypeScript の型検査、Vite のビルド |
| Web UI の見た目 | `cd apps/playground && bun run test:visual` | Chromium で、すべての HTML サンプルの画面の大きさごとの表示、操作、画像が読めないときの代わりの表示、Code と Explorer を確かめる。確認用の PNG と HTML の報告を出す |
| Rust | `bun run check:rust` | Rust の整形と、ワークスペースのテスト |
| 本番用ビルドをブラウザーで | `bun run check:production:browser` | CLI で Web のひな形と Flow を開発用とリリース用にビルドし、Chromium で操作して、表示と入力の値、ソースマップの既定、JavaScript の例外を確かめる。Playground の依存と Chromium が必要。全体のチェックとリリースのチェックにも含まれる |
| 本番用の成果物 | `bun run check:production` | 6つの fixture のリリース用ビルド、成果物の情報の形、必要な実行時のライブラリ、大きさの上限、ハッシュと同一性、ビルドし直したときの一致、プロセスとしての実行。全体のチェックとリリースのチェックにも含まれる |
| fixture | `bun run check:conformance` | 適合を確かめる fixture を実行する。対象のフォルダーを引数で絞れる |
| コンパイラーの成果物（PR の CI） | `cargo run -p seseragi-conformance -- . --only=tokenStream,losslessCst,surfaceAst,diagnostics,semanticDiagnostics,analysis,moduleInterface,resolvedAst,typedHir,typedInterface,coreIr,typescriptIr` | `examples/spec/artifacts` のトークン、構文木、診断、モジュールの公開情報、型つきの中間表現、Core IR、TypeScript IR を、今のコンパイラーの出力と比べる。tsc、Node、provider は使わない。出力を変えたのに成果物を作り直していないと、`conformance-artifacts.yml` で失敗する。作り直しは `cargo run -p seseragi-conformance --bin write_schema1_artifact -- <case>` |
| `.d.ts` の変換（WASM） | `bun run check:dts:wasm` | ファイル操作を外した変換器を `wasm32-unknown-unknown` 向けにビルドし、ホストの機能を使わずに変換、情報、作り直しの報告、診断を実行する。WASI SDK 25.0 と Rust の WASM のターゲットが必要 |
| コミットした WASM | `bun run check:wasm` | Playground の WASM を作り直し、差分がないことを確かめる。WASI SDK 25.0 が必要 |
| 純粋なプログラムの WASM | `cargo test -p seseragi-cli --test program_wasm` | プログラムの WASM を Bun と Node で実行し、TypeScript（両方のビルド）と native の結果と比べる。生成した型宣言の strict な型検査、標準パッケージのモジュールと素材のビルドと失敗したときに前の出力を残すこと、生成した仮のパッケージの持ち主、パスの依存と初期化、コールバックとインスタンスの寿命、trap の後に使えなくすること。`rustup target add wasm32-unknown-unknown` が必要。全体のチェックでも省かない |
| provider を使わない Effect の WASM | `cargo test -p seseragi-backend-rust --test program_wasm_effect --test program_wasm_effect_admission --test program_wasm_private_ref` と上の CLI のテスト | 型、ソースの段階、受け付ける式を確かめる。WASM で、実行するまで動かない Effect、内部の Ref、型のついた失敗、明示的な Defect、取消、取消を止めた後片付け、非同期の破棄、返したコールバックを、新しく起動した Bun と Node で実行する。純粋なプログラムとは別の ABI で、ホストの provider や trap の後の巻き戻しは保証しない |
| ホストの Effect を使う WASM | `cargo test -p seseragi-backend-rust --test program_wasm_host_effect --test program_wasm_host_ref` と `cargo test -p seseragi-cli --test program_wasm_host_effect` | 引数1つのホストのコールバック、内部の Ref の取り込み、通常の TypeScript と native との本体の比較、WASM の Failure と Defect、遅い応答の後の取消、取消を止めて後に開いたものから返すこと、破棄とホストの貸し出しの解放、標準パッケージの型つきの橋渡し、Console の provider を使う Effect。Console 以外の provider や trap の後の巻き戻しは含めない |
| WASM のパッケージの監視 | `cargo test -p seseragi-cli --test dev` と `bun test ./runtime/ts/tests/program-lifecycle.test.ts` | 計算のモジュールの外側のパスの依存と、UI のビルドし直し、失敗したときに完成したサイトと版を残すこと、開発サーバーの寿命と権限の確認。保存したコールバックの拒否、古い root がゼロになること、遅い読み込みの回収を、新しく起動した Bun と Node でも実行する |
| VS Code の拡張機能 | `bun run check:extension` | 正式な ID と古い ID の移行用の拡張機能の区別、lint とテスト、手元の環境向けの VSIX のアーカイブの形式、展開、`--version-json` の確認。LSP を起動しない移行用の VSIX のパッケージと検証 |
| 手元の Web 製品の E2E | native のアーカイブと VSIX を作った後に `bun run test:local-web-e2e` | 展開した CLI とインストールした VSIX だけを使い、複数モジュールの Web プロジェクトで、LSP、開発サーバー、ブラウザー、編集、診断、回復、本番用ビルドを同じソースで確かめる。報告と PNG を `target/local-web-product-e2e` に残す |
| 版と native の配布物 | `bun run check:release` | 版の元の値、Cargo・JS・WASM の版がそろっていること、CHANGELOG、手元の環境向けの CLI と LSP のアーカイブとチェックサム、展開し直して実行できること |
| タグのリリースの前のチェック | `bun run check:release-gate` | 全体のチェックと同じ内容。ただし native と VSIX のパッケージの確認は、同じコミットを使うリリースの各ジョブに任せ、成果物を二重に作らない |
| リポジトリ全体 | `bun run check` または `bun run check:full` | 整形、lint、Rust のワークスペース、すべての fixture、native のサンプル、WASM、Playground、拡張機能 |

### チェックの決まり

- Playground のテスト、型検査、ビルドは、カタログと Tour の一覧の確認を1度だけ済ませてから実行します。同じ確認を3回繰り返しません。
- `check:sample` は、CLI で実行するサンプルだけでなく、ブラウザーで操作するサンプルも、コミットした WASM でコンパイルします。すべてのソースがフォーマッターの出力と一致することも確かめます。一覧やハッシュだけを更新して、壊れたソースが通ることはありません。
- サンプルと Tour のコマンドは、プロセスの終了と、標準出力・標準エラーの終わりの両方を確かめます。
  - 時間切れは、それだけで失敗として返します。後片付けで標準入出力を無期限に待ちません。
  - POSIX では、親が先に終わっても、持っているプロセスのグループを止めます。
  - 確かめる方法: `scripts/sample-cli-process.test.ts`（親が終わった後の出力、0でない終了、Unicode、時間切れと後片付け）
- 配布するアーカイブを確かめるときは、展開した CLI と LSP とアーカイブのコマンドについて、プロセスの終了と標準入出力が閉じたことを確かめてから、一時フォルダーを消します。
  - 一時的な `EBUSY` などで消せないときは、最初を含めて最大6回まで、待ち時間は合わせて 1500ms まで試します。
  - 確認と後片付けの両方が失敗したら、両方のエラーを残します。確認が成功しても、後片付けが失敗したら失敗にします。後片付けが失敗したときは、実行したコマンドの PID、終了コード、exit と close を観測したかを残します。
  - 確かめる方法: `bun test scripts/native-release.test.ts`
  - Windows に固有のハンドルの競合は、Windows の CI と正式なリリースのアーカイブの確認でも確かめる必要があります。
- PostgreSQL の provider をまとめたファイルが最新かを確かめるときは、実際に解決した `node_modules` の節のコメントだけを正規化します。`bun test scripts/postgres-provider-bundle.test.ts` で、通常のインストールと、深い worktree のシンボリックリンクのインストールで同じバイトになること、実行するコードが変わったら拒否することを確かめます。
- Playground が実行時のライブラリを探す表は、`runtime/ts/package.json` の `exports` と、`runtime/providers/package.json` の `runtime-browser/*` から作ります。
  - 作り直しは `bun run --cwd apps/playground runtime:generate`。`runtime:check` と Playground のテストが、生成物、コンパイラーの Runtime ABI のモジュールと export、ブラウザーの provider の宣言がそろっていることを確かめます。
  - ホストのアダプターは実行時のパッケージの `browser` 条件で、native 専用のモジュールは `browser: null` で宣言します。Playground に別のモジュールの一覧を足しません。
  - provider の `browser/` のソースから `runtime-browser/` のパッケージへの写しは、Rust のパッケージの準備のテストでも同じ中身かを確かめます。

### 標準インスタンスの確認表

- prelude の確認表で「仕様にあるが実装がない」（`specified-but-implementation-missing`）行は、登録表の `tracking_issue` に正の Issue 番号を必ず持ちます。
- 一覧を作るときに確かめるので、単体テストだけでなく、fixture の成果物の照合も、追跡していない行を拒否します。GitHub への問い合わせはしません。
- 実装が終わったら、その行を消します。実装済み、構造から決まるもの、わざと提供しないもの、の行に古い追跡の情報を残しません。
- `prelude::surface` の仮の「実装がない」テストは、今の一覧が空でも、この約束と JSON の `trackingIssue` を確かめます。
- 確認表だけを変えたときは、この意味のテストと、fixture の `stdlib_surface` のテストを先に実行できます。

## Web UI の見た目の確認

`apps/playground/tests/fixtures/web-ui-regression.json` が、すべての HTML サンプル、画面の大きさ（320px、iPhone、Android、横向き、デスクトップ）、確かめる部分、操作の状態を決めています。
ふだんの `check:playground` は、この表とソースの読みやすさを Bun のテストで確かめ、Chromium を勝手にダウンロードしません。
実際のブラウザーで見るときだけ、次を実行します。

```sh
cd apps/playground
bun run test:visual:install
bun run test:visual
```

- Playwright は、決まった Unsplash の URL を手元の SVG に置き換え、配置、はみ出し、コントラスト、キーボードで届く操作、画像が読めないときの代わりの表示を確かめます。
- 代表の状態は、確認済みの画面の画像（基準の画像）とも比べます。失敗したら、`test-results/web-ui-review/` に期待した画像、実際の画像、差分、trace、HTML の報告を残します。
- OS ごとの文字の描き方の違いを配置の違いと取り違えないように、基準の画像は OS ごとに持ちます。手元でも GitHub Actions でも、確かめるコマンドは `bun run test:visual` です。

**許す差分**

- 画面全体の基準の画像は、Playground の枠と Preview、Editor、Explorer の位置の関係を確かめるために残します。
- 許す差分は、画像全体の割合ではなく、それぞれの基準の画像が見ている部分（Preview の本文、Editor の枠、Workspace の枠）の面積の1%を画素の数に直して決めます。
- 色の差のしきい値 0.25 で、OS ごとの文字の描き方の違いを吸収します。部分の余白、文字、そろえ方の違いが、大きな余白に薄まって見逃されないようにするためです。
- 感度のテストで、3種類の意図した部分の差分が、この上限を超えて失敗になることを確かめます。

**基準の画像を更新する**

意図して UI を変えたときは、先に [`SHOWCASE_QUALITY.md`](./SHOWCASE_QUALITY.md) の人による確認を済ませてから、理由を必ず書いて次を実行します。

```sh
bun run test:visual:update -- "変更理由"
```

- PNG と、承認済みの `showcase-review.json` の SHA-256 と理由が、`e2e/visual-baselines.review.json` に記録されます。PNG だけ、または承認の記録だけを変えると、確認の前に失敗します。
- 更新のコマンドは、基準の画像を持つ Web UI のテストだけを実行し直します。ふだんの `bun run test:visual` は、ガイドを含むすべての Playwright のテストを実行します。
- Linux の基準の画像も同じコマンドで作ります。`Web UI visual regression` のワークフローを手動で実行し、`update_reason` を渡します。

## CI と手元の分担

- 手元では、変えた範囲に合うチェックを実行します。
- `.github/workflows/vscode-extension.yml` は、拡張機能のパスが変わったときに、OS ごとの VSIX のパッケージ、アーカイブの形式、展開した native の LSP の `--version-json` の確認を受け持ちます。手元の `check:extension` は、手元の環境だけの短い確認です。タグのリリースでも、アップロードする前の VSIX に同じ確認をもう1度実行します。
- `.github/workflows/release-contract.yml` は、版の元の値や配布物の名前が変わったときに、`check:release` と、版と配布の約束の cargo のテストだけを実行します。
  - 対象は `seseragi-release` 全体、`seseragi-cli --test build reports_version_commit_and_channel`、`seseragi-lsp --test stdio binary_reports_its_distribution_contract`、`seseragi-cli --test run single_file_and_package_compile_diagnostics_share_the_json_envelope`。どれも `--locked` を付け、個別のテストは `-- --exact` を付けます。
  - CLI と LSP のテスト全体は、それぞれのワークフローが受け持ちます。Linux の native のアーカイブを実際に動かす確認は、`linux-native.yml` とタグのリリースが受け持ちます。
  - タグのリリースは `v<version>` だけを受け付けます。タグのコミットが `main` に含まれ、`check:release-gate` が成功したときだけ、同じコミットからすべての環境の成果物を作り、アップロードの前と、ダウンロードし直した後に確かめます。
- リポジトリ全体のチェックは、コンパイラー・実行時のライブラリ・WASM の変更、リリース、作業の列の統合など、決まった場面で明示的に実行します。CI で、サンプルを変えるたびに同じ長いチェックを重ねません。

### CI で使う OS

- `pull_request` では Linux だけを実行します。
  - 対象: `rust-backend.yml` の `file-transform`、`dts-portable.yml` の `convert`（PR では `cross-host-results` も省く）、`vscode-extension.yml` の `package`（macOS と Windows だけなので、PR ではジョブごと省く。Linux の VSIX は `linux-native` が受け持つ）
  - matrix の要素には `if` を書けないので、OS の並びを `fromJSON(github.event_name == 'pull_request' && '[...Linux]' || '[...全OS]')` で切り替えます。
- macOS と Windows は、`main` への push と手動の実行（`workflow_dispatch`）で実行します。そのため `dts-portable.yml` にも `main` への push を加えています。毎晩の定期実行は置きません。`release.yml` の macOS と Windows の確認は変えません。
- macOS と Windows だけの失敗は、マージした後に初めて分かります。OS によって変わる変更（パス、プロセス、ファイルのハンドル、改行、WASM の作り直しなど）は、PR の前に手元の Mac で下の手順を実行します。
- PR から動くワークフローは `concurrency`（ワークフロー、イベント、PR 番号ごと。PR だけ前の実行を取り消す）を持ちます。
- Ubuntu の Rust のジョブは `Swatinem/rust-cache@v2` をジョブごとのキーで使い、保存は `main` への push のときだけにします（PR は読み出すだけ）。`CARGO_INCREMENTAL=0` は保ちます。

### 手元の Mac で macOS の CI を再現する

固定したツール（`rust-toolchain.toml` の 1.97.1 と `wasm32-unknown-unknown`）、`.bun-version` の Bun、Node 24.22.1 を用意し、リポジトリのルートで実行します。CI と同じ環境変数を使います。

CI は `file-transform` の CLI のテストを複数のジョブに分けて実行します（`scripts/cargo-test-shard.ts`）。手元では分けずに、下の `cargo test` をまとめて実行してかまいません。
`CARGO_TARGET_DIR` を設定すると、同じテストどうしが native の実行時のライブラリのビルドを共有するので速くなります（CI も設定しています）。

```sh
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$PWD/target"
bun install --frozen-lockfile

# rust-backend.yml file-transform (macos-15)
cargo test --locked -p seseragi-backend-rust --lib runtime::host_tests
cargo test --locked -p seseragi-runtime --test provider_catalog_identity
cargo test --locked -p seseragi-backend-rust --lib -- \
  runtime::text_tests runtime::math_tests runtime::bytes_tests \
  runtime::array_operation_tests runtime::sequence_tests runtime::big_int_tests \
  runtime::decimal_tests runtime::json_tests runtime::json_codec_tests \
  runtime::json_decoder_tests runtime::json_container_codec_tests \
  runtime::json_derived_adt_tests runtime::hashing_tests runtime::map_tests runtime::set_tests
cargo test --locked -p seseragi-cli \
  --test filesystem --test fibers --test async_filesystem --test inherited_equality \
  --test string_equality --test text_processing --test int_parse --test float_parse \
  --test float_conversion --test int_remainder --test native_power --test basic_standard \
  --test process_entry_strict --test public_aliases --test namespace_results \
  --test computed_callees --test nominal_lambda_identity --test captured_primitive_identity \
  --test program_wasm --test program_wasm_host_effect --test alias_constructor_sections
cargo test --locked -p seseragi-cli --test rust_backend --test dev -- \
  list_prefix_ closed_display_ \
  watches_program_wasm_path_dependencies_keeps_failed_builds_and_closes_real_instances \
  watches_host_effect_programs_and_drains_the_old_application_before_reload \
  initial_program_
bun test ./runtime/ts/tests/program-lifecycle.test.ts ./runtime/ts/tests/program-lifecycle-async.test.ts
bunx tsc --strict --noEmit --skipLibCheck false --target ES2022 --module ESNext \
  --moduleResolution Bundler --lib ES2022,DOM runtime/ts/tests/program-lifecycle-types.ts

# dts-portable.yml convert (macos-15)。WASI SDK 25.0 を展開して WASI_SDK_PATH を設定し、
# wasm-pack v0.15.0 と `cargo install wasm-bindgen-cli --version 0.2.126 --locked` を用意する。
cargo fmt --all -- --check
cargo test --locked -p seseragi-dts
cargo test --locked -p seseragi-dts --no-default-features
cargo test --locked -p seseragi-cli --test dts
cargo test --locked -p seseragi-wasm
cargo test --locked -p seseragi-conformance --bin seseragi-conformance runtime_package::http_server::tests
bun run check:dts:wasm
builds="$(mktemp -d)"
SESERAGI_WASM_BUILD_EVIDENCE="$builds/evidence-first" CARGO_TARGET_DIR="$builds/target-first" \
  ./scripts/build-playground-wasm.sh "$builds/pkg-first"
SESERAGI_WASM_BUILD_EVIDENCE="$builds/evidence-second" CARGO_TARGET_DIR="$builds/target-second" \
  bun run build:playground:wasm
diff -r "$builds/pkg-first" apps/playground/src/wasm/pkg
(cd apps/playground && bun install --frozen-lockfile \
  && bun test tests/binding-wasm.test.ts tests/workspace-*.test.ts && bunx tsc --noEmit \
  && bunx playwright install chromium && bunx playwright test e2e/workspace-bindings.spec.ts)
git checkout -- apps/playground/src/wasm/pkg  # macOS で作ったものはコミットしない

# vscode-extension.yml package (macos-15 / darwin-arm64。Intel Mac は darwin-x64)
(cd extensions/seseragi && bun install --frozen-lockfile \
  && SESERAGI_EXTENSION_TARGET=darwin-arm64 bun scripts/package-extension.ts)
```

Windows に固有の確認（`bun test scripts/native-release.test.ts` など）は手元で代わりに確かめられません。必要なときは、rust-backend と vscode-extension のワークフローを手動で実行します。

ファイルや FIFO の非同期の入出力だけを OS ごとに切り分けたいときは、`rust-backend.yml` を手動で実行し、`async_io_only=true` を渡します。
Linux、macOS、Windows で `cargo test --locked -p seseragi-cli --test async_filesystem` を実行し、15秒の期限、ハンドルを閉じること、ワーカーの終了待ち、後片付け、出力を守ることを確かめます。
`integration=true` はこの指定より優先し、全体のチェックを実行します。

## WASM を作り直すとき

- [WASI SDK 25.0](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-25) の、手元の OS と CPU 向けのアーカイブを展開し、`WASI_SDK_PATH` にそのフォルダーを設定します。C で書いた構文解析器のコンパイラーも固定し、Rust と wasm-pack だけをそろえても生じる差分を防ぎます。
- WASM のつなぎのコードには、`cargo install wasm-bindgen-cli --version 0.2.126 --locked` で `Cargo.lock` と同じ CLI を用意します。`wasm-pack` の自動インストールは CLI の中の依存の版を変えるので、ビルドは `--mode no-install` で既存の CLI を使います。
- 配布する WASM を作る正式な環境は Linux x64 です。`check:wasm` は生成物全体がバイト単位で一致することを確かめ、PR とリリースの Linux の確認で必須にしています。
- Cargo は、ホストで動くビルドスクリプトや proc macro の情報を依存先に伝えるので、macOS で作ったものはコードやデータに差分が出ることがあります。
  - macOS の CI では、同じ固定のツールと、2つの独立した新しい Cargo のターゲットで作り直します。1回目の新しい出力と、2回目のリポジトリへの出力で、すべてのファイルの集合とバイトが一致することを確かめ、変換、ワークスペース、ブラウザーの回帰を確かめます。
  - さらに、両方の OS の変換の JSON の出力を CI で完全に比べます。macOS で作ったもので配布物を置き換えません。

## リリース用ビルドの最適化の確認

`cargo test -p seseragi-cli --test profiles` は、性能の fixture を開発用とリリース用の両方でコンパイルし、それぞれを新しいプロセスで実行します。
さらに、出力したリリース用のモジュールに対して `scripts/release-shapes.ts` を実行し、fixture の条件を確かめます。先に `bun install --frozen-lockfile` でルートの依存を入れます。

- チェッカーは、生成した確認用の入口からモジュールの宣言を探します。最適化が成功したかどうかを、成果物の情報で報告することはありません。
- 構文木による確認は控えめです。対応していない包みや取り出しの形は、推測せずに失敗にします。
- 確認用の入口は、リリース用の出力に残します。まとめて不要なコードを消す処理は、別の境目です。

`bun test scripts/release-shapes.test.ts runtime/ts/tests/effect-stack.test.ts` は、形の拒否と、純粋な末尾呼び出しの最適化とは別に、10万段の Effect の連鎖を確かめます。
fixture のチェック（`check:conformance`）にも含まれます。ビルドの種類、コンパイラー、実行時のライブラリを合わせて変えたときは、全体のチェックと WASM の作り直しも必要です。

## native の標準 API のテスト

native（Rust）で実行できる標準 API を加えたり直したりしたときに、実行するテストの一覧です。

### 共通で確かめること

どの API も、次をテストで確かめます。

- 元の生成した TypeScript を、開発用とリリース用の両方で strict（`skipLibCheck: false`）に型検査する
- 新しく起動した Bun、Node、native で、同じ結果になる
- 保存した関数、部分適用、ジェネリクスの呼び出し、中身を隠した型を持つ2つのモジュールのパッケージでも動く
- 実行するまで動かない Effect をもう1度実行できる。型のついた Failure と Defect の後に、後片付けが動き、管理している root がゼロに戻る。Ref の循環も回収する
- 偽の Core の入力（型の持ち主、種、引数、結果、辞書、ソースの段階）を、元の UTF-8 の位置を示して拒否する
- 新しい API を、プログラムの WASM では受け付けない。使っていない保存した関数でも、位置を示して拒否する
- 入力と、失敗したときの前の出力を変えない

共有のコンパイラーや実行時のライブラリを変えたときは、統合の前に全体のチェックと、コンパイラーの WASM の作り直しも必要です。

### テストの書き方

| 表の書き方 | 実行するコマンド |
| --- | --- |
| backend `--test x` | `cargo test -p seseragi-backend-rust --test x` |
| runtime `x` | `cargo test -p seseragi-backend-rust --lib x` |
| CLI `--test x` | `cargo test -p seseragi-cli --test x` |
| CLI `x_` | `cargo test -p seseragi-cli x_`（名前で絞り込む。分かるときは `--test basic_standard` などを付ける） |
| driver `--test x` | `cargo test -p seseragi-driver --test x` |
| backend `--lib x` | `cargo test -p seseragi-backend-rust --lib x` |
| semantics、lowering、syntax | `cargo test -p seseragi-semantics`、`seseragi-lowering`、`seseragi-syntax` に同じ引数を付ける |

### 数値

| 対象 | テスト | 補足 |
| --- | --- | --- |
| Int の十進の読み取り | backend `--test int_parse`、runtime `int_parse_tests`、CLI `--test int_parse` | 標準の `IntParseError` の持ち主、種、引数の数を確かめる。`IntParseError` の Show は backend と driver の `--test int_parse_error_show` と CLI `--test int_parse standard_parse_error_show` |
| Int の基数の変換、書き出し、上限と下限 | backend `--test int_text --test int_parse`、runtime `int_parse_tests`、CLI `basic_int_radix_` | 2〜36進、符号、先頭の0、大文字と小文字、UTF-8 のバイト位置と最初の失敗を TypeScript に合わせる。返す関数の型は driver `--test strict_partials --test generic_call_instantiation --test hkt_erasure` |
| Int のあふれを調べる計算と上限で止める計算 | backend `--test int_checked --test int_text --test int_parse`、runtime `int_checked_tests`、CLI `basic_int_checked_` | 右の引数を先に評価する。大きな指数でも計算が終わる累乗 |
| Int の `>` と `>=` | backend `--test comparison_partials`、CLI `basic_array_infix_descending_` | |
| Int の余り | backend `--test int_remainder --test int_division --test operator_partials`、CLI `--test int_remainder` | 0で割ると Defect |
| Int と Float の累乗 | backend `--lib power`、`--test native_power`、CLI `--test native_power` | Int の安全な範囲、負の指数、あふれの Defect。Float の NaN、無限大、符号つきの0、指数の偶奇 |
| Float の読み取り | backend `--lib parse_tests`、`--test float_parse --test int_parse --test int_parse_error_show`、CLI `--test float_parse --test int_parse` | 十進と指数の構文、UTF-8 のバイト位置、構文の失敗とあふれの順、非正規化数、アンダーフロー、符号つきの0、明示的な NaN と無限大 |
| Float の丸めと Int への変換 | backend `--lib float_conversion_tests`、`--test float_conversion`、CLI `--test float_conversion` | 6つの丸め方の中間の値と隣の値、符号つきの0、安全な Int の境、有限でない値の失敗 |
| Math の定数、単項の関数、`atan2`、`hypot` | backend `--test math_remaining`、runtime `math_tests`、CLI `basic_math_remaining_` | `atan2` は y が先。`hypot` は途中の2乗であふれない |
| Math の `sqrt`、`sin`、`cos`、`log` | backend と CLI `--test basic_standard`、runtime `math_tests` | 特別な値と、仕様の許容誤差 |
| Decimal の読み取り、`fromInt`、Show | backend `--test basic_decimal`、runtime `decimal_tests`、CLI `basic_decimal_` | 係数と小数点の位置を任意精度で持ち、指数を Float や固定幅の整数に丸めない。負の0と末尾の0を正規化する |
| Decimal の Eq と Hash | backend `--test decimal_keys --test basic_decimal`、runtime `decimal_tests`、CLI `basic_decimal_keys_` | TypeScript と同じ、符号つきの32ビットの区切りのハッシュ |
| Decimal の Ord と比較 | backend `--test decimal_ord --test decimal_keys --test basic_decimal --test method_references`、runtime `decimal_tests`、CLI `basic_decimal_ord_` | 1000桁の小数点の位置を展開しない |
| TypeScript の Decimal の0の比較 | runtime/ts の `decimal.test.ts`、`decimal-zero-order.test.ts`、CLI `--test decimal_zero_order` | |
| Decimal の足し算、引き算、掛け算、Zero、One、Debug | backend `--test decimal_arithmetic`、runtime `decimal_arithmetic_tests`、CLI `basic_decimal_arithmetic_` | 独立した Python の Decimal の期待値と比べる |
| Decimal の割り算と `quantize` | backend `--test decimal_division --test decimal_context --test decimal_conversion`、runtime `decimal_division_tests`、CLI `basic_decimal_division_` | Python の Fraction と Decimal の期待値と比べる。長い do の native の生成は CLI `basic_long_effect_sequence_` |
| Decimal の変換（`toIntExact`、`fromFloat`、`toFloat`） | backend `--test decimal_conversion`、runtime `decimal_conversion_tests`、CLI `basic_decimal_conversion_` | Float のビットから正確な係数を作ってから丸める |
| DecimalContext | backend `--test decimal_context`、runtime `decimal_context_tests`、CLI `basic_decimal_context_` | `std/number` の RoundingMode を共有する |
| BigInt の読み取り、書き出し、`fromInt` | backend `--test basic_big_int`、runtime `big_int_tests`、CLI `basic_big_int_` | 任意の桁数を丸めない |
| BigInt の基数の変換と `toInt` | backend `--test big_radix --test basic_big_int`、runtime `big_int_tests`、CLI `basic_big_radix_` | 10万桁の十進の入力 |
| BigInt の Eq、Ord、Hash、Show、Debug | backend `--test big_int_keys --test big_int_ord --test big_int_display`、runtime `big_int_dictionary_tests`、`big_int_tests`、CLI `basic_big_keys_` | Eq、Ord、Hash は O(n)。読み取りと書き出しは仕様の O(d²) 以内 |
| BigInt の計算、Zero、One、失敗を返す API | backend `--test big_arithmetic`、runtime `big_int_arithmetic_tests`、CLI `basic_big_arithmetic_` | 0の方向への割り算、割られる数と同じ符号の余り |
| Hash の標準の辞書（Int、Bool、Char、String、Unit） | backend `--test basic_hash`、runtime `hashing_tests`、CLI `basic_hash_` | 公開のハッシュに種を混ぜない |

### コレクション

| 対象 | テスト | 補足 |
| --- | --- | --- |
| Array の基本（`filter`、`find`、`get`、`length`、`take`、`drop`） | backend `--test basic_array`、runtime `array_operation_tests`、CLI `basic_array_` | 既存の Array の添字の ABI を変えない |
| Array の残り（`empty`、`append`、`reverse`、`head`、`tail`、`filterMap`、`flatMap` など） | backend `--test array_rest --test basic_array`、runtime `array_`、CLI `basic_array_rest_` | 共有の修正は driver `--test array_alias_elements --test alias_type_functions --test inferred_collection_owner` |
| Array と List の基本の22の操作 | backend `--test basic_sequence`、runtime `sequence_tests`、CLI `basic_sequence_` | Array の `last` は O(1)。List は含まれる後ろの部分を共有する |
| Array と List の `concat` | backend `--test basic_sequence`、runtime `sequence_tests`、CLI `basic_concat_` | |
| Array と List の `sort`、`sortBy`、`groupBy` | backend `--test sequence_order`、runtime `sequence_order_tests`、CLI `basic_sequence_order_` | 安定した下からのマージ。キーは入力の順に1回だけ評価する |
| List の `filterMap`、`flatMap` | backend `--test list_transform --test basic_sequence`、runtime `list_transform_tests`、CLI `basic_list_transform_` | |
| List の先頭の共有 | lowering `--test match_decision`、backend `--test native_list list_prefix`、CLI `--test rust_backend list_prefix_decisions_and_effects_match_raw_strict_ts_and_native` | 速さは `measure_list_prefix.py` で測る |
| `fromIterable` と `Map.fromEntries` | backend `--test iterable_build --test iterator`、runtime `iterable_tests`、CLI `basic_iterable_build_` | 重複したキーは値を更新し、最初のキーを残す |
| Iterator の `unfold`、`next` | backend `--test iterator`、runtime `iterator_tests`、CLI `basic_iterator_` | 保存した参照は driver `--test iterator_saved_functions` |
| `collection.reduceUntil` | backend `--test reduce_until --test iterable_build`、runtime `collection_control_tests`、CLI `basic_reduce_until_` | `Done` の後に `next` を呼ばない |
| Maybe の `withDefault`、`orElse`、`sequence`、`traverse` | backend `--test maybe_operations --test hkt_projections --test reduce_until`、runtime `maybe_operation_tests`、CLI `basic_maybe_operations_` | NonEmptyList の provider は、#1048 と #1101 で ABI を確かめるまで拒否する |
| Map の基本 | backend `--test basic_map`、runtime `map_tests`、CLI `basic_map_` | パッケージは `.gitattributes` で LF に固定する |
| Set の基本と Map の `containsKey`、`entries` | backend `--test basic_collections`、runtime `set_tests`、`map_tests`、CLI `basic_collections_` | Set は Map の記憶域を共有する |
| Map の `upsert`、`mergeWith`、`mapKeysWith` | backend `--test basic_map_upsert`、runtime `map_tests`、CLI `basic_map_upsert_` | Hash を1回、Eq で探し、更新を1回 |
| Map の `filter`、`mapValues` | backend `--test basic_map_operations --test basic_map`、runtime `map_tests`、`set_tests`、CLI `basic_map_operations_`、`basic_set_operations_` | |
| Set の `filter`、`union`、`intersection`、`difference`、`isSubsetOf`、`toList` | backend `--test basic_set_operations --test basic_collections`、runtime `set_tests`、CLI `basic_set_operations_` | |
| `Set.map` | backend `--test basic_set_map`、runtime `set_tests`、CLI `basic_set_map_` | `Map.mapKeysWith` と実装を共有する |

### テキストとバイト

| 対象 | テスト | 補足 |
| --- | --- | --- |
| `split`、`trim`、`contains`、`replace` | backend と CLI `--test basic_standard`、runtime `text_tests` | |
| `startsWith`、`endsWith` | backend `--test basic_standard`、runtime `text_tests`、CLI `basic_text_boundaries_` | |
| `lines`、`join` | backend `--lib text_tests`、`--test text_processing`、CLI `--test text_processing` | |
| String の Eq | backend `--test string_equality --test equality_partials`、CLI `--test string_equality` | |
| Bytes の `fromInts`、`toInts`、`get`、`toInt` | backend `--test basic_bytes`、runtime `bytes_tests`、CLI `basic_bytes_` | |

### JSON

| 対象 | テスト | 補足 |
| --- | --- | --- |
| 読み取りと書き出し | backend `--test basic_json`、runtime `json_tests`、CLI `basic_json_` | 深さ20,000をヒープで確かめる |
| 標準の12の変換の辞書 | backend `--test basic_json_codecs`、runtime `json_codec_tests`、CLI `basic_json_codecs_` | DecodeError の型は driver `--test json_decode_error --test struct_construction_kind` |
| Array、List、Maybe、Either、タプル、Map、Set、閉じたレコード | backend `--test basic_json_containers`、runtime `json_container_codec_tests`、CLI `basic_json_containers_` | 変換の深さの上限は128 |
| デコーダーの組み合わせ（`field`、`oneOf`、`flatMap` など） | backend `--test basic_json_decoders`、runtime `json_decoder_tests`、CLI `basic_json_decoders_` | 同時に128を超えると、スタックを壊さずに Defect にする |
| 自動導出（代数的データ型） | backend `--test basic_json_derived_adts`、runtime `json_derived_adt_tests`、CLI `basic_json_derived_adts_` | |
| 自動導出（newtype） | backend `--test basic_json_derived_newtypes`、CLI `basic_json_derived_newtypes_` | リテラルの表現は driver `--test newtype_literal_profiles`、CLI `newtype_literal_representations_` |
| 自動導出（struct） | backend `--test basic_json_derived_structs`、CLI `basic_json_derived_structs_` | struct の中のレコードの型は syntax の `nested_record_types_keep_their_struct_fields`、driver `--test struct_construction_kind`、CLI `nested_record_struct_fields_` |
| プロトタイプと同じ名前のフィールド | Playground の `json-record-runtime.test.ts`、CLI `json_record_prototype_field_` | |

### 型と共有のコンパイラー

| 対象 | テスト |
| --- | --- |
| 制約つきの関数や辞書のメソッドが返す関数を続けて呼ぶ | backend `--test returned_evidence_stages --test strict_partials`、driver `--test strict_partials --test generic_call_instantiation --test hkt_erasure`、CLI `--test alias_constructor_sections` |
| 標準の関数が返す関数の呼び出しと、if の分岐の推論 | driver `--test standard_returned_application --test conditional_branch_inference`、CLI `basic_standard_returned_application_`、`basic_standard_conditional_maybe_callback_` |
| ジェネリクスの呼び出しに渡す、先頭が空の入れ子の Array と List | driver `--test nested_empty_call_inference`、CLI `inferred_nested_empty_collection_` |
| import した呼び出せる別名の中の Prelude の型からの推論 | driver `--test generic_call_instantiation --test hkt_erasure --test strict_partials`、CLI `inferred_json_decoder_package` |
| 取り込んだ基本型の推論 | semantics の `--lib`、driver `--lib --test compile_module --test captured_primitive_identity`、CLI `--test captured_primitive_identity` |
| 基本型の Eq の TypeScript での形 | CLI `--test string_equality --test inherited_equality`。Bool と Unit の Eq と前置の `!` は backend と CLI `--test basic_scalars` |
| import した代数的データ型の同一性 | driver `--test frontend_payload_identity --test frontend_inference --test imported_returns`、CLI `--test string_equality --test nominal_lambda_identity` |
| struct の構築の書き方で別の型に変えてしまう不具合 | driver `--test struct_construction_kind` と semantics |
| レコードの変性 | driver `--test variance` |
| ジェネリクスの impl の演算子 | driver と CLI `--test generic_impl_operators` |
| 高カインド型のデータ、フィールド、公開の別名 | driver と CLI `--test hkt_data_projection`、driver `--test hkt_annotation_kinds` |
| 別名の引数を固定したコンストラクター | driver と CLI `--test alias_constructor_sections` |
| 別名の引数の並べ替え、重複、破棄 | semantics `type_functions::tests`、lowering `core::types::tests`、driver と backend `--test alias_type_functions`、driver と CLI `--test alias_constructor_sections`。形の別名は driver `--test structural_aliases --test structural_alias_facades`、CLI `--test rust_backend alias_head` |
| 互いに呼び合う閉じた Effect の TypeScript の型 | driver と CLI `--test mutual_effect_returns` |
| `do` の最後に分岐がある Effect | driver と CLI `--test deferred_effect_environment` |
| 閉じたタプルの Show と Debug の辞書の使い回し | backend `--test closed_display`、CLI `--test rust_backend closed_display_`。速さは `measure_display_runtime.py` |

### Effect と実行時

| 対象 | テスト | 補足 |
| --- | --- | --- |
| Console の String の約束 | driver と backend と CLI `--test console_string_contract`、`bun test runtime/ts/tests/console-string-contract.test.ts` | |
| Clock の `sleep` | backend と CLI `--test clock_sleep`、runtime `clock_tests` | native の CLI の provider を明示的に選ぶ。共通のスケジューラーを変えるので、ファイル操作、Fiber、プログラムの WASM の回帰も確かめる |
| 開発サーバーの停止とリクエストの待ち行列 | CLI `--bin seseragi dev::tests::`、`--test dev` | 実際に待ち受けるソケットに複数のリクエストを並べる。止めるときの猶予は5秒 |
