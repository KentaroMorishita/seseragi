# Scoped checks

Seseragiの検証は、変更範囲に対応するscoped laneを先に実行します。`bun run check`
は、各laneを日常作業で無条件に繰り返すためのコマンドではなく、repository-wideな
最終統合gateです。

## Lane matrix

| 変更範囲 | コマンド | 含まれる検証 |
| --- | --- | --- |
| sample metadata / source | `bun run check:sample` | native CLI sample実行、Playground sample manifestのfreshness |
| sample / Playground UI | `bun run check:playground` | `check:sample`、Tour manifest、Playground test、TypeScript typecheck、Vite build |
| Rust / compiler | `bun run check:rust` | Rust format、workspace test（対象crateだけなら `bun run check:rust -- -p <crate>`） |
| conformance fixture | `bun run check:conformance` | canonical conformance runner（対象rootを引数で限定可能） |
| compiler/runtime/WASM boundary | `bun run check:wasm` | committed Playground WASMの再生成と差分確認 |
| VS Code extension | `bun run check:extension` | extension source lint、Bun test、現在のhost向けVSIX package / verify |
| repository-wide | `bun run check` または `bun run check:full` | format、lint、Rust workspace、全conformance、native samples、WASM、Playground、extension |

各laneは `scripts/check-scoped.sh` を共通runnerとして使います。Playgroundのtest、
typecheck、buildはcatalog / Tour manifest確認をlane内で一度だけ済ませてから直接実行し、
`package.json`の個別scriptが同じcatalog確認を三回繰り返す構造を避けています。

## Dependency installation

通常のscoped checkは依存installを行いません。lockfileまたは依存関係を変更した場合だけ、
該当workspaceで `bun install --frozen-lockfile` を実行します。新しいworktreeやCI runnerの
初回bootstrapで必要なinstallは、checkの検証本体とは別の準備手順です。full gateだけは
再現性のためPlaygroundとextensionのfrozen installを従来どおり含みます。

## Full gateを実行する条件

次のいずれかに該当するときだけ、scoped laneに加えて `bun run check` を実行します。

- compiler / runtime / WASM基盤を変更した
- 複数の領域を横断した
- release前の統合確認を行う
- #201のqueueをmainへ統合する
- Issue本文がfull checkを明示的に要求する

full gateを実行した場合は、作業ログまたはPR本文へ「必要理由」と「結果」を記録します。
長時間無出力になった場合は、該当phaseとprocess activityを確認し、放置しません。

## CIとlocalの責務

- localの末端Issue作業は、変更範囲に対応するscoped laneを担当する。
- `.github/workflows/vscode-extension.yml` はextension path変更時のOS別VSIX packageと
  native LSP testを担当する。local `check:extension`はhost platformの短い再現確認である。
- repository-wideのfull gateは、compiler/runtime/WASM変更、release、queue統合などの
  integration pointで明示的に実行する。CIに同じ長時間検査を各sample変更へ無条件に重ねない。
- scoped laneが不足している場合は、対象コマンドを個別に実行して結果を記録し、lane追加を
  次の基盤Issueとして扱う。
