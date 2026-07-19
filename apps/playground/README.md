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

## HTML preview

SSRとinteractive DOMは、script実行を許可しない同じsandbox iframeへ表示します。preview documentには
Playgroundが所有するTailwind風utility CSSの小さなsubsetを注入するため、Seseragi側は`className`へ
`flex`、`grid`、spacing、typography、color、border、shadow、`sm:` responsiveなどを指定できます。
inline styleとCSS variablesは`html.style`で併用できます。外部CDNとiframe内scriptには依存しません。

## mobile layout contract

iPhone Safariで編集時の自動zoomを避けるため、CodeMirrorのeditable surface、sample select、Stdinは
16px未満にしません。狭い画面では文字を縮める代わりに、line height、line number gutter、inline padding、
panel headingを圧縮します。lint diagnosticsは本文のunderlineとtooltipを維持し、空のgutter icon領域だけを
非表示にします。

portraitの小画面に加え、iPhone相当のlandscape viewportでもCode / I/Oのsingle-panel tabsを維持します。
touch targetは44pxを下回らず、viewport metaでpinch zoomを禁止しません。

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
