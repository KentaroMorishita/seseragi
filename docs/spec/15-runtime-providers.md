# Runtime Provider Contract

## 15.1 目的と正本

Runtime Provider Systemは、HTTP、filesystem、database、clockなどの外部capabilityを、
applicationが特定hostへ依存せず要求するための境界です。次の4層は別の責務とversionを持ちます。

```text
Seseragi std / package API
        ↓
backend非依存のProvider Contract
        ↓
backend固有のRuntime ABI / shared bridge
        ↓
runtime provider → host API / external library
```

| 層 | 所有するもの | 所有しないもの |
| --- | --- | --- |
| std / package API | applicationが使う型、Effect、typed failure、値の正規化 | Bun / Node等の実装identity、ABI表現 |
| Provider Contract | service identity、論理operation、input、success、typed failure、operation種別、portable性 | TypeScript object、Promise、Rust trait、host handle |
| Runtime ABI / bridge | backend上の値、call、result、handleの受け渡しとstd値への変換 | serviceの公開意味、provider選択 |
| runtime provider | host API呼び出しとABI値への変換 | std内部表現、application API |

Provider Contractの規範的な意味は本章、machine-readableなschema 1の例は
`examples/spec/artifacts/provider-contract-schema-1/`を正とします。JSON fixtureはContractを
TypeScript interfaceやRust traitへ固定するためのものではなく、compiler、resolver、backend projector、
conformance runnerが同じ論理契約を読めるようにするartifactです。

## 15.2 Service identityとrequirement

一つのContract artifactは一つのserviceを定義します。service identityは公開service型のcanonical
identityです。canonical module identity自体がpackage identityとrelative module pathを`::`で
分ける場合も、その完全なidentityを保持します。

```text
<canonical module identity>::<UpperCamelCase service name>
```

標準serviceでは`std/clock::Clock`、`std/fs::FileSystem`のように書きます。外部packageのserviceも
`acme/payments::service::Payments`のように6章と11章のpackage / module identityを使います。backend名、provider
package名、entry module名をservice identityへ含めません。`bun/clock::Clock`や
`typescript/promise::Promise`はportable Contract identityではありません。

`requirement`はEffect environmentへ現れるcanonical fieldと型を表します。

```json
{
  "field": "clock",
  "type": "std/clock::Clock"
}
```

`requirement.type`はContractの`identity`と一致しなければなりません。applicationとdependencyから抽出する
program requirementはこのfield / typeだけを要求し、BunProvider、NodeProvider、npm package、ABI entryを
含みません。同じservice型を複数fieldで使い分けるapplicationは5.4のclosed environment recordでfieldを
区別しますが、provider選択規則は後続節で定めます。

## 15.3 Contract version

`schema`はartifact JSONの読み方、`version`は一つのservice contractの意味を表し、package versionとは
独立です。

```json
{
  "schema": 1,
  "kind": "provider-contract",
  "identity": "std/clock::Clock",
  "version": { "major": 1, "minor": 0 }
}
```

- `schema`は正確に`1`、`kind`は`provider-contract`です。
- `version.major`は1以上、`minor`は0以上の整数です。
- package releaseは複数Contractを含められるため、package SemVerからContract versionを推測しません。
- Contract versionとbackend ABI versionを同一値にする必要はありません。

breaking / additive change、range negotiation、runtime handshakeの詳細は本章の後続仕様で定めます。このsliceでは
version layerを増やさず、schema、service contract、package、backend ABIが別identityであることだけを固定します。

## 15.4 論理型

operationのinput、success、failureはbackend非依存の論理型treeです。schema 1は次を認識します。

| kind | field | 意味 |
| --- | --- | --- |
| `unit` | なし | 引数なし、または有用なsuccess値なし |
| `never` | なし | inhabitantのないfailure / success |
| `primitive` | `name` | `bool`、`bytes`、`float`、`int`、`string` |
| `named` | `identity` | canonical module identityを持つ公開型 |
| `array` | `items` | 同一論理型のimmutable列 |
| `record` | `fields` | 名前付きfieldのclosed record |

`record.fields`はsource順を意味にせず、field名を重複できません。各fieldはlowerCamelCaseの`name`と
再帰的な`type`だけを持ちます。`named.identity`はservice identityと同じcanonical type identity形式です。

Contractには`Promise`、`AbortSignal`、`Uint8Array`、JavaScript object、Rust type、native pointerを
書きません。たとえば`Bytes`のTypeScript表現が`Uint8Array`でもContractは`primitive: bytes`を使い、
投影とownershipはTypeScript Runtime ABIが所有します。schemaで定義していないfieldやkindをconsumerが
黙って無視してはなりません。

## 15.5 Operation identityと最小分類

operation identityは次の形式です。

```text
<service identity>#<lowerCamelCase operation name>
```

各operationは次のfieldを必ず持ちます。

```json
{
  "identity": "std/clock::Clock#now",
  "kind": "one-shot",
  "input": { "kind": "unit" },
  "success": { "kind": "named", "identity": "std/time::Instant" },
  "failure": { "kind": "never" },
  "portability": { "kind": "portable" },
  "summary": "Observe the current monotonic instant."
}
```

`identity`は同じContract内で一意です。`input`は全引数を一つの論理値へまとめ、引数なしは`unit`、
複数のnamed引数は`record`で表します。`failure`は省略できず、失敗しないoperationは`never`です。
host exceptionやinvalid provider valueをtyped failureへ追加してはなりません。

schema 1の`kind`は次の分類だけを持ちます。

- `one-shot`: 一回の開始に対して一回だけsuccessまたはtyped failureを完了する。
- `resource`: scopeに所属するresourceを取得する。release protocolの詳細は後続仕様で定める。
- `subscription`: 複数eventを届けるlifetimeを開始する。callback / backpressureの詳細は後続仕様で定める。

この分類だけからcancellation、close、Stream、callback表現を推測しません。operationごとの意味は本章と
対応するstd / package仕様が所有し、`summary`はartifactを監査する短い説明であって規範本文の代用ではありません。

## 15.6 Portable operationとtarget extension

全targetで同じ意味を持つoperationは次を使います。

```json
{ "kind": "portable" }
```

target固有operationを後から追加する場合は次の形でportable surfaceから区別します。

```json
{ "kind": "target-extension", "target": "bun" }
```

このmarkerはBun固有APIをstandard portable serviceへ昇格しません。target extensionの命名、import、
package portability、diagnostic、互換性は後続仕様で確定します。現時点で具体的なBun / Node extensionを
標準Contractへ予約しません。

## 15.7 Clockとfilesystemによる検証

`provider-contract-schema-1/clock/contract.json`は、host resourceを持たない小さいserviceを検証します。

- `Clock#now`: UnitからInstantを返すone-shot operation。
- `Clock#sleep`: Durationを受け取りUnitで完了するone-shot operation。
- どちらもprovider identity、Promise、timer handleをContractへ含めません。

`provider-contract-schema-1/filesystem/contract.json`は、値だけでは足りないresource境界を検証します。

- `FileSystem#openRead`: Pathからopaqueな論理`FileHandle`を取得するresource operation。
- `FileSystem#read`: handleとlimitをclosed recordで受け、BytesまたはFileErrorを返すone-shot operation。
- `FileSystem#close`: host file descriptorではなく論理handleを受けるone-shot operation。

この二例が同じschemaを使えるため、Contract vocabularyはHTTP request / response objectやJavaScript Promiseを
前提にしません。一方、`resource`のcleanup、`sleep`のcancellation、closeの冪等性は未確定のままにせず、
後続のEffect / resource contractで定義します。

## 15.8 既存境界との関係

- 5章のEffect environmentはContractの`requirement`をapplication型として運びます。Provider Contractは
  Effectのcold性、failure、cancellation意味を上書きしません。
- 7章のTypeScript `foreign`はprovider実装がhost packageを呼ぶ手段になれますが、Contractそのものでは
  ありません。
- 11章のpackage identity / version / source identityはprovider packageにも適用しますが、service contract
  versionをpackage SemVerへ畳みません。
- 12章のtarget capabilityはprovider resolutionの入力になります。現在の`console`、`stdin`、`dom` registryを
  Contract artifactへ暗黙変換しません。
- `runtime-schema-1/core/abi.json`は現行TypeScript runtime feature registryです。backend非依存の
  `provider-contract-schema-1`とは別artifactであり、feature IDやimport pathをContractへ複製しません。

## 15.9 Schema 1の拒否条件

consumerは少なくとも次をContract errorとして拒否します。

- envelope、version、requirement、operation、logical type、portabilityのunknown field
- 不正schema / kind、空operations、0のcontract major
- requirement typeとservice identityの不一致
- canonicalでないservice / named type / operation identity
- 同じoperation identityまたはrecord field名の重複
- 未知operation kind、logical type kind、primitive、portability kind
- backend固有namespaceを論理type identityへ入れること

unknown fieldを無視すると新producerの意味を旧consumerが誤読するため、schema majorが一致していてもclosed
objectとして検査します。package manifest、TypeScript ABI、Effect / resource、Streamの追加fieldを先回りして
このschemaへ入れず、それぞれの後続契約へ分離します。
