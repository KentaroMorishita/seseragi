# Seseragi Playground

Rust compilerのshared driverをWASMから呼び出すmobile-first Playgroundです。
compiler semanticsをUIへ複製せず、CLI / LSPと同じpipelineを利用します。

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
増やしません。interactive sampleはtyped messageとSignalで状態を更新し、同じpreview iframeで
browser DOMまで実行します。

通常のinteractive sampleは`dom.app { target, initial, update, view }`を使います。Signal生成、query、default options、
mount lifecycle、portableなerror変換は標準helperが所有し、effectful dispatchやcustom lifecycleが必要な場合だけ
低レベルの`dom.query` / `dom.run`へ降ります。

## HTML preview

SSRとinteractive DOMは、iframe-owned scriptをCSPの`script-src 'none'`で拒否する同じsandbox iframeへ表示します。
WebKitが親pageから登録したevent listenerも`allow-scripts`なしでは停止するため、sandbox tokenは
`allow-same-origin allow-scripts`とし、実行可否はpreview documentのCSPで固定します。preview documentには
Playgroundが所有するTailwind風utility CSSの小さなsubsetを注入するため、Seseragi側は`className`へ
`flex`、`grid`、spacing、typography、color、border、shadow、`sm:` responsiveなどを指定できます。
inline styleとCSS variablesは`html.style`で併用できます。外部CDNとiframe内scriptには依存しません。

## mobile layout contract

iPhone Safariで編集時の自動zoomを避けるため、CodeMirrorのeditable surface、sample / Referenceの検索、Inputは
16px未満にしません。狭い画面では文字を縮める代わりに、line height、line number gutter、inline padding、
panel headingを圧縮します。lint diagnosticsは本文のunderlineとtooltipを維持し、空のgutter icon領域だけを
非表示にします。

portraitの小画面に加え、iPhone相当のlandscape viewportでもCode / I/Oのsingle-panel tabsを維持します。
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
