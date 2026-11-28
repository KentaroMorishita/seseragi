# Playground information architecture

この文書は、Playground と Tour で公開する学習・探索画面の役割と導線の基準です。
見た目や部品の細部は [`DESIGN.md`](./DESIGN.md) にまとめます。

## 利用者向けの三つの入口

| 入口 | 利用者の目的 | 内容と順序 |
| --- | --- | --- |
| Playground | コードを自由に編集して実行する | 現在のワークスペース。例を読み込んだあとも編集できる |
| Tour | Seseragi を最初から順に学ぶ | `examples/tour/curriculum.json` が定める順序付きの経路 |
| Search（`⌘K`） | 例、レッスン、API、設定を横断して探す | 範囲（All / Examples / Lessons / API / Settings）を切り替えて順不同で選ぶ |

`Learn` は独立したナビゲーション、タブ、一覧名にしません。学習を案内するときは、内容と進み具合を二重化せず、明示的に Tour へ導きます。

## Playground の初期状態

Playground は、初めて開いた利用者に動くコードと結果をすぐ見せます。

- 初回表示（保存済みのコードも `?source=` 付きのリンクもなく、`?example=` で別の例が指定されていないとき）は `weather-advice`（「天気のアドバイス」）を読み込みます。ブラウザー保存の Hints が有効な場合、一度だけ自動で実行し、書き換える場所を指すひとことと Try next のチップを出します（Hints がオフの場合はひとこと・Try next を隠し、自動実行もしません）。なお、幅 760px 以下、または幅 960px 以下かつ高さ 520px 以下の狭い画面ではひとことの吹き出しは表示しません。
- `?example=<id>` で別の例を直接開いたとき（Tour や Deep Dive の例リンクなど）は、指定された例に切り替えます。`weather-advice` の自動実行やひとことは表示せず、Hints が有効ならその例の Try next を表示します（指定例に `guidance.autoRun` が設定されている場合は、Hints が有効なら自動実行も行います）。
- 2 回目以降に前回保存したコードが復元できたとき、または `?source=` 付きのリンクや Search（`⌘K`）の `New file` から始めたときは「自由に書く」状態で開きます。案内（ひとこと、Try next）は出さず、自動実行もしません。
- 利用者はいつでも次を明示的に選べます。
  - **New file（白紙）**: Search（`⌘K`）から選ぶ。空の `main.ssrg` を持つワークスペースを作り、例の解説や Try next を持たない。
  - **Examples（Search `⌘K` 内）**: `examples/samples/` の例を選んで読み込む。
  - **Reset**: 現在の例（または白紙のワークスペース）を元の状態に戻す。
  - **Tour**: 順序付きの学習ページ（`/tour/`）へ移動する。

New file と例の切り替えは、保存していない変更があるときに確認を出して黙って破棄しません。Reset は現在のワークスペースを元の状態へ直ちに戻します。

## 内容の振り分け

- **Lesson**: 後続の概念の前提になる一つの概念を段階的に教える。Tour だけに置く。
- **Recipe**: 既習の概念を組み合わせ、一つの実用目的を達成する。Examples の一覧に一度だけ置く。
- **Showcase**: 複数の機能を組み合わせた完成例。Examples の一覧に一度だけ置く。

Recipe と Showcase は `examples/samples/discover-groups.json` の一つのグループにだけ所属させます。Lesson を Examples に入れず、同じ例を別名のカードや固定 HTML として複製しません。設計文書だけで使う内部の分類名（Recipe、Showcase、画面区分、補助分類など）は画面に出しません。

## Featured

Featured はカタログ上で最初に薦める例として選ぶ基準です（現行の Search（`⌘K`）の Examples 一覧では `discover-groups.json` のグループ順に並べており、Featured による優先表示や絞り込みは行っていません）。

- 最大 8 件とする。
- 表示順は `discover-groups.json` のグループ順、その中の並び順とする。別の手動順序を持たない。
- 現行のコンパイラー、CLI、WASM で実行でき、コード、Guide、メタデータが一致しているものだけを選ぶ。
- Web UI の例は、Preview、モバイル表示、主要な操作を確認済みにする。
- Showcase は公開前に [`docs/SHOWCASE_QUALITY.md`](../../docs/SHOWCASE_QUALITY.md) の基準を満たす。

更新時は例の `sample.json` の `featured` と `discover-groups.json` だけを変更し、マニフェストを再生成して `bun run check:playground` を実行します。

## 操作の五層構造

操作は作用する範囲を基準に、次の五層に分けます。

| 層 | 責務 | 置き場所 |
| --- | --- | --- |
| 画面全体 | 現在地、Playground / Tour の切り替え、Search（`⌘K`）、テーマ、言語（JA / EN）、More | 上のバー |
| ワークスペース | 例・ワークスペースの切り替え、New file、Reset、Files（Explorer）、Guide | コードのカードの見出し、または Search（`⌘K`） |
| エディター操作 | Format、検索と置換、行コメントなど現在ファイルへの編集操作 | コードのカードの見出し・エディター内 |
| ペイン操作 | Explorer の開閉、Tour の Contents（コースマップ）、Output（Console / Preview / 全画面、Playground の Input / Clear） | 対象ペインの見出しまたは境界 |
| 設定 | テーマ、空白の表示、フォーマッターの行幅、Hints、外部通信の許可管理 | Search（`⌘K`）の Settings（詳細設定は別ダイアログ） |

上のバーに新しい操作ボタンを直接足しません。まず作用する範囲を判断し、ワークスペース、エディター、ペイン、Search の Settings のいずれかに置きます。上のバーには画面全体に効く操作だけを残します。

Run ボタンは上のバーに置きません。Playground ではコードのカードと出力のカードの間の流れの上に、Tour では右列のコードと出力の間の流れの上に置きます。

### 画面の切り替え（Playground / Tour）

Playground と Tour のどちらも、上のバーに `Playground` / `Tour` の二択を常に表示します（デスクトップでは中央、コンパクト表示では左寄せ）。メニューに畳まず、今いる側を選択中として見せます。デスクトップでもモバイルでも、常設の別リンクや「戻る」ボタンを増やしません。

Examples・API・Settings への入口は Search（`⌘K`）に一本化します。Playground のコードのカード見出しにある例の名前を押すと、Search が Examples の範囲で開きます。例を選んだあとも編集対象は Playground のワークスペースのままです。なお、Settings のうち `Editor settings` と `Network access` は Search から選んだあとに独立した設定ダイアログを開きます。

### ペインの所有権

- **Explorer（Files）**: 開くボタンはコードのカードの見出しに置き、閉じるボタンは Explorer の見出しに置く。
- **Tour の Contents（コースマップ）**: レッスン列の上部バーにある `Contents` から開く。章をカードで並べて進み具合と現在地（Current）を示し、開いても上のバーは残す。
- **Format**: Playground / Tour とも、コードのカードの見出しに置く。
- **Output**: `Console` / `Preview` の切り替え、診断カードによるエラー表示、実行状態のチップ（`✓ 18 ms`、`1 error` 等）、Preview の全画面は出力のカードが所有し、上のバーへ移さない。Playground では `Input` と `Clear` も出力のカードに置く一方、Tour では `Input` はコードのペイン内に置き、出力の見出しには `Clear` ボタンを置かない。
- レイアウトの変更や開閉で、コードや実行結果を暗黙に書き換えない。

### 共有設定

Playground と Tour は同じ設定（`EditorPreferences`）とブラウザー保存キーを共有します。空白の表示（`Show indentation whitespace`）とフォーマッターの行幅は Search（`⌘K`）の Settings から開く `Editor settings` ダイアログで変更でき、テーマも含めて一方で変更した値を他方にも反映します。外部通信の許可管理（`Network access` ダイアログ）は同じ一覧を両画面から管理できますが、許可そのものは Playground では例または作業場所単位、Tour ではレッスン単位で記録されるため、画面や別の例・レッスンをまたいで自動共有されません。なお、Hints のオン・オフ切り替えは Playground 側の Search Settings にのみ置き、Tour 側には表示しません。

## デスクトップとモバイルの対応

デスクトップでは、Playground は左にコード・中央の流れに Run・右に出力の横並び、Tour は左にレッスン本文・右にコード / Run / 出力の縦並びとします。

モバイル／コンパクト表示（幅 760px 以下、または幅 960px 以下かつ高さ 520px 以下の低い横画面）では上を二段までに収め、入りきらない操作は削らずにシートや浮かぶボタンへ移します。

| 画面 | 1 段目（上のバー） | 2 段目 | 本体と Run |
| --- | --- | --- | --- |
| Playground | ロゴ、Playground / Tour、Search、テーマ / 言語 / More | 例の名前、Format、Files / Guide 等 | コードを表示。出力は下から出るシート（畳む・196px・画面の 7 割の三段）。右下に Run を浮かべる |
| Tour | 同じ | Contents、章の名前、進み具合のバー、位置（`1/24` 等） | 下の `Lesson` / `Code` で切り替える。Code を開いたときだけ右下に Run を浮かべる |

モバイルで配置を変えても、名称、作用範囲、遷移先、カタログの内容は変えません。

## 追加時の判断

新しい公開機能が必修の前提概念なら Tour へ、一つの目的を解く例なら Examples の Recipe へ、完成形を見せる統合例なら Showcase へ追加します。API を調べる入口は Search の API、法則や設計背景は Deep Dive（Tour の Going further 等）に置きます。

Showcase を追加するときは、この振り分けを前提にし、情報設計を作り直さず内容と品質に集中します。
