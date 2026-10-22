# Seseragi Playground

Rust compilerのshared driverをWASMから呼び出すmobile-first Playgroundです。
compiler semanticsをUIへ複製せず、CLI / LSPと同じpipelineを利用します。

Playground、Tour、Discover、Recipe、Showcaseの役割とdesktop / mobileの導線は
[`INFORMATION_ARCHITECTURE.md`](./INFORMATION_ARCHITECTURE.md)を正本とします。

## 境界

- compile: `seseragi-wasm` -> `seseragi-driver::compile_module`
- diagnostics: driverのUTF-8 byte rangeをUI境界でCodeMirrorのUTF-16 offsetへ変換
- execute: generated TypeScript -> `runtime/ts`のbrowser Console / Stdin host
- editor: CodeMirror 6とPlayground専用のSeseragi stream language

UIはparser、resolver、type checker、Effect semanticsを所有しません。CLI / LSP / Playgroundは同じdriver、
structured diagnostics、runtime entry contractを利用します。

sampleは`examples/spec`のcanonical sourceを直接bundleし、WASM compileとbrowser executionをtestします。

公開catalogはcompilerの内部fixtureを網羅する一覧ではなく、言語を段階的に試せる代表sampleへ絞ります。
各sampleの目的と主要conceptはeditor上の`Guide` overlayから確認でき、説明のためにworkspaceの表示行を
増やしません。interactive sampleはtyped ActionとSignalで状態を更新し、同じpreview iframeで
browser DOMまで実行します。

初回workspaceは`hello-world`のcanonical sourceを直接使うStarter-firstです。`New blank`は空の
`main.ssrg`を持つ別originを作り、Resetは現在のBlankまたはsampleのcanonical stateへ戻します。
どちらもdirty fileを確認なく破棄せず、復元済みworkspaceを初期starterで上書きしません。

通常のinteractive sampleは`dom.app { target, initial, update, view }`を使います。Signal生成、query、default options、
mount lifecycle、portableなerror変換は標準helperが所有し、effectful dispatchやcustom lifecycleが必要な場合だけ
低レベルの`dom.query` / `dom.run`へ降ります。

`Feature Composition` sampleは、その低レベル境界を一つのprivate `mount` helperへ閉じます。各featureはprivate stateから
read-onlyな`Signal<Html<Effect<{}, Never, Unit>>>`を作り、親は子Actionのvariantを知らずにhide / re-show、並べ替え、
差し替えを行います。実際のmodule privacyとshared stateは対応するmulti-module project fixtureで検証します。

formは`onInput` / `onChange`からnative Eventそのものではなく、immutableな`InputEvent` / `ChangeEvent` snapshotを
typed Actionへ変換します。`onSubmit`はbrowser navigationより先に`preventDefault`されます。controlled inputと
textareaはstableな`id`を使うと、Signal更新による再render後もfocusとselectionを維持します。日本語IMEなどの
composition中はrerenderを保留し、browserごとのevent順序差を吸収して確定valueだけを一度Actionへ変換します。

## HTML preview

SSRはscriptを拒否するCSP付きdocumentへ表示し、Text表示中はHTML documentを作りません。PlaygroundとTourの実行はopaque originのsandbox iframeから開始します。DOMを使わないprogramは、そのiframeが所有する専用Workerで評価し、10秒で終了しない場合はWorkerを停止します。DOM programはiframe内で動き、生成されたloopと関数に同期計算量の監視を入れます。任意のforeign JavaScriptはこの監視を回避できるため、DOMとforeign runtimeを併用するprojectはPreviewでは拒否し、信頼できるprojectに限ってCLIでの実行を案内します。
親ページはprogramを評価せず、`MessageChannel`で開始、結果、キャンセルを受け渡します。iframeは`allow-scripts allow-forms`だけを持ち、same-origin、popup、top navigationを許可しません。HTTP / WebSocketと外部HTTPS画像は初期状態でCSPにより拒否し、Tools内のNetworkを利用者が選んだ場合だけ許可します。許可は保存せず、codeやprojectを変更すると解除します。外部form送信も拒否します。これは完全なoffline保証ではなく、Preview内の通常のlink navigationは残ります。
Storageはworkspaceごとのeditor所有identity（Tourはlesson ID）と256 Ki UTF-16 code unitsのbudgetを持つoperation窓口へ接続します。親ページのworkspaceや以前の共通application namespaceへは接続しません。古いapplication保存データは削除も自動移行もしません。NavigationはPreview内の独立したhistoryを使います。
共有source linkは空のstdinと独立した保存slotで開きます。初回保存後はURLをそのslotへ置き換え、reloadで編集済みのcopyを復元します。元のlocal workspaceを上書きせず、`/`へ戻ると元のworkspaceを復元します。
compiler Workerは15秒で応答がないと停止し、次の要求で再作成します。program出力は256 Ki UTF-16 code unitsまでです。DOM監視は協調的な制限であり、組み込み処理の巨大allocationやブラウザ全体のmemoryをOSのように制限する保証はありません。詳細は`docs/threat-model.md`を参照してください。
preview documentには
Playgroundが所有するTailwind風utility CSSの小さなsubsetを注入するため、Seseragi側は`class`へ
`flex`、`grid`、spacing、typography、color、border、shadow、`sm:` responsiveなどを指定できます。
利用可能tokenは注入するCSS selectorから機械的に導出され、`samples:check`が`outputMode: html`の
全sourceにある直接の`class`、`cx [...]`の文字列、expected HTMLの`class`を検証します。
`sm:`や`hover:`はescaped selectorを元のtokenへ戻して照合し、未定義tokenはsample ID・file・token付きで
失敗します。任意式でclassを組み立てる場合は`sample.json`の`preview.dynamicUtilities`へ候補tokenを列挙し、
見た目を持たないsemantic custom classだけを`preview.customClasses`で明示します。custom classへCSSは
自動追加されません。視覚的なcustom値とCSS variablesはutility検証外の`html.Style`へ置きます。
Fullscreenはiframeを作り直さず、未対応時にはsafe area対応の疑似全画面へ切り替えます。`target="_blank"`のlinkはsandboxを継承し、親ページへの参照を許可しません。
Web UI Showcaseのremote画像は検索・random endpointではなく固定photo IDのHTTPS URLを使い、意味のある`alt`、
`width` / `height`、`aspect-2-1 h-auto w-full object-cover`で成功時と失敗時のlayoutを安定させます。
Showcaseの完成条件は[`../../docs/SHOWCASE_QUALITY.md`](../../docs/SHOWCASE_QUALITY.md)を正本とし、
landing page reference、題材固有のvisual identity、desktop / mobile / interactionのhuman review、
source readabilityを同時に要求します。CI greenだけではvisual qualityの承認にしません。
TourのChapter / lesson一覧はmobileでsafe area対応の全画面sheetになり、背景scrollを固定します。明示的な
閉じるbuttonとEscape、循環するTab focusを持ち、lesson選択後はsheetを閉じて選択したlesson見出しへ移動します。

## mobile layout contract

iPhone Safariで編集時の自動zoomを避けるため、CodeMirrorのeditable surface、sample / Referenceの検索、Inputは
16px未満にしません。狭い画面では文字を縮める代わりに、line height、line number gutter、inline padding、
panel headingを圧縮します。lint diagnosticsは本文のunderlineとtooltipを維持し、空のgutter icon領域だけを
非表示にします。

portraitの小画面に加え、iPhone相当のlandscape viewportでもCode / I/Oのsingle-panel tabsを維持します。
panel切替時は非選択側のroot workspaceをlayoutから外し、Explorerの開閉状態に関係なく選択側だけが利用可能な高さを占有します。
SampleとRunは常時表示し、Reference、Reset、空白表示はkeyboard操作可能なoverflowへまとめます。Inputは
Output headingから必要なときだけ開きます。型tooltipはtouch cursorでも開き、visual viewport内で反転・scrollし、
editorと同じSeseragi分類でsignatureを表示します。空白表示は行中の通常spaceを汚さず、行頭indentとtrailing
whitespaceだけを示します。diagnostic cardはUTF-8 byte rangeを内部に保持しながら、
1始まりの行・列を表示し、選択箇所をCode panelへ戻します。touch targetは44pxを下回らず、viewport metaで
pinch zoomを禁止しません。

## 開発

```sh
bun run build:playground:wasm
bun run dev:playground
bun run check:playground
```

Rust側のcompiler、runtime contract、または`seseragi-wasm`を変更したcommitでは、最初のcommandで
`src/wasm/pkg`を再生成し、integration testと同じcommitへ含めます。
`bun run test:playground:wasm`は再生成後にGit差分がないことも検査し、古いdeployment artifactを拒否します。

## Vercel

Vercelは`vercel.json`に従い、このdirectoryのfrozen lockfileをinstallしてVite buildだけを実行します。
Vercelのbuild hostではRustや`wasm-pack`を実行しません。review済みのWASM packageをversioned deployment
artifactとしてrepositoryへ含めることで、Git integrationとlocal buildのcompiler binaryを一致させます。

```sh
bun run check:playground
bunx vercel deploy
```

productionで正常に動作することを確認してから`bunx vercel deploy --prod`を実行します。

Production: <https://seseragi.vercel.app/>
