# Runtime Provider authoring guide

このguideは、Seseragi application APIを変えずに新しいruntime providerを追加し、共通
conformance profileへ接続するための実装手順です。公開意味の正本は
[Runtime Provider Contract](./spec/15-runtime-providers.md)、実行可能なprofileの正本は
`examples/spec/artifacts/provider-conformance-profile-schema-1/core/profile.json`です。

## 1. 最初に固定する境界

providerは次の4層を分離します。

1. application API: `std/http::HttpClient`のような通常のSeseragi API。
2. Provider Contract: operation、logical input / success / typed failure、operation kind。
3. backend ABI / bridge: logical valueのencode / decode、Promise result、opaque handle。
4. provider entry: Bun / Node APIやexternal driverを呼ぶhost固有実装。

application source、Contract、typed failureへprovider identity、host object、Promise、driver classを
漏らしてはいけません。host throw / rejectionと未宣言boundary valueはtyped failureではなくdefectです。

## 2. Artifactを追加する

既存の最も近いcapabilityを複製元にし、次を別artifactとして追加します。

- `examples/spec/artifacts/provider-contract-schema-1/<service>/contract.json`
- `examples/spec/artifacts/provider-manifest-schema-1/<provider>/provider.json`
- TypeScript ABIの新しいlogical typeが必要な場合だけ
  `provider-typescript-abi-schema-1/core/abi.json`

Contract identityはpackage / module / service identity、provider identityは
`<package>#<provider>`です。Contract version、backend ABI major、target、entry export、runtime
feature、host packageはmanifestで明示します。external driverを使うproviderはSemVer rangeをmanifestと
runtime packageの両方へ置き、lockfileのexact version / source / digestをresolution結果へ残します。

artifact schemaはclosedです。unknown fieldを将来互換として無視せず、schema majorを変えない新fieldは
optionalでなければなりません。

## 3. TypeScript entryを実装する

共通bridgeは`@seseragi/runtime/provider`、package entry / load / shutdownは
`@seseragi/runtime/provider-package`を使います。provider entryは次を守ります。

- Effectはcoldで、一runにつきhost operationを一回だけ開始する。
- declared failureだけを`{ kind: "failure", failure }`へ写す。
- throw、rejection、invalid input / result valueはboundary defectとして保持する。
- cancellation通知は高々一回。cancel後のlate completionは観測して破棄する。
- resource acquireとfinalizer登録をatomicにし、closeは冪等にする。
- child resourceからparent resourceへLIFOで解放し、shutdownはchild cleanupを待つ。
- Stream / cursorはconsumer demandを超えてpullせず、unsubscribe後のeventを渡さない。

opaque handleはowner provider / serviceを検査し、application値や別providerへhost handleを渡しません。
record、array、Bytesはbridgeで検証してcopyします。

## 4. 共通profileへ接続する

providerごとのprobeは実operationを通し、観測結果を
`@seseragi/runtime/provider-conformance`へ渡します。profileは次の10 caseを一つずつ要求します。

| case | 必須観測 |
| --- | --- |
| `success` | success terminal |
| `typed-failure` | declared failure channel |
| `defect` | input / call / result boundary defect |
| `cancellation` | 通知高々一回、late completion破棄 |
| `cleanup` | acquire / release一致、active resource 0 |
| `concurrency` | 2 operation以上がoverlapし、すべてsettle |
| `invalid-value` | application値へ漏れずdefect |
| `mismatch` | entry評価前にresolution失敗 |
| `ambiguity` | entry評価前にresolution失敗 |
| `leak` | cleanup後active handle 0 |

resource shapeには`cleanup`と`leak`、cancellable shapeには`cancellation`が必須です。
`mismatch` / `ambiguity`は個別entryを起動せずresolver fixtureで検査します。Never failureのoperationだけを
持つproviderは、suite内の別のdeclared failure operationで`typed-failure`を満たし、その対応をprofileへ記録します。

```ts
import { assertProviderConformanceCase } from
  "@seseragi/runtime/provider-conformance"

assertProviderConformanceCase({
  id: "leak",
  activeAfterCleanup: activeHandles.size,
})
```

canonical profileの各`evidence`はrepository-relativeな実在fileでなければなりません。新capabilityを追加するときは
profileのcapability / case対応とevidenceを同じcommitで更新します。

## 5. Probeの最低条件

provider probeはapplication側importとprovider側entryを分け、applicationにprovider identityが現れないことを
検査します。hostを決定的fixtureへ差し替えられる境界を用意し、少なくとも次を実行します。

- 正常値とdeclared failureをContract valueへ変換する。
- synchronous throw / rejected Promise / invalid boundary valueをdefectにする。
- cancelとcompletion、明示closeとshutdownを競合させる。
- closeを複数回呼び、host releaseが一回であることを確認する。
- cleanup後のactive handle数を0として観測する。
- 二つ以上のoperationを同時に進め、結果やresource ownershipが混線しないことを確認する。
- target / ABI mismatchとambiguityでentry module評価回数が0であることを確認する。

network、filesystem、databaseのprobeは終了時にlistener、file、cursor、connection、poolを残してはなりません。
一時artifactは成功時も失敗時もcleanupします。

## 6. 検証と変更の完了条件

まず対象crate / probeを絞って失敗を短くし、最後に変更範囲に対応するrepository gateを実行します。

```sh
cargo test -p seseragi-conformance provider_conformance_profile
cargo run -p seseragi-conformance -- .
bun run check
```

`cargo run -p seseragi-conformance -- . --json`では
`providerConformanceProfile`の件数と失敗pathをmachine-readableに取得できます。local TypeScript / Bun依存は
lockfileからinstallし、floating tool downloadや`--skipLibCheck`で検査を弱めません。

完了時はContract、manifest、ABI / bridge、runtime entry、probe、profile evidence、documentationを同じ
変更として揃えます。full gateがgreenでも、profileにcaseまたはevidenceがないproviderはconformance済みとは扱いません。
