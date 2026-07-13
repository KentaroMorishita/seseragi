# Seseragi Playground

新Rust compilerのsingle-file driverをWASMから呼び出すmobile-first Playgroundです。旧`playground/`の
React / Monaco UIや旧TypeScript compilerは利用しません。

## 境界

- compile: `seseragi-wasm` -> `seseragi-driver::compile_module`
- diagnostics: driverのUTF-8 byte rangeをUI境界でCodeMirrorのUTF-16 offsetへ変換
- execute: generated TypeScript -> `runtime/ts`のbrowser Console / Stdin host
- editor: CodeMirror 6とPlayground専用のSeseragi stream language

UIはparser、resolver、type checker、Effect semanticsを所有しません。CLI / LSP / Playgroundは同じdriver、
structured diagnostics、runtime entry contractを利用します。

現在はHello world、型付きじゃんけん、doの逐次実行、pure let、bindの5 sampleを収録しています。
いずれも`examples/spec`のcanonical sourceを直接bundleし、WASM compileとbrowser executionをtestします。

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
