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

## 15.7 Clock、filesystem、HTTPによる検証

`provider-contract-schema-1/clock/contract.json`は、host resourceを持たない小さいserviceを検証します。

- `Clock#now`: UnitからInstantを返すone-shot operation。
- `Clock#sleep`: Durationを受け取りUnitで完了するone-shot operation。
- どちらもprovider identity、Promise、timer handleをContractへ含めません。

`provider-contract-schema-1/filesystem/contract.json`は、値だけでは足りないresource境界を検証します。

- `FileSystem#openRead`: Pathからopaqueな論理`FileHandle`を取得するresource operation。
- `FileSystem#read`: handleとlimitをclosed recordで受け、BytesまたはFileErrorを返すone-shot operation。
- `FileSystem#close`: host file descriptorではなく論理handleを受けるone-shot operation。

TypeScript runtimeの最小sliceは`seseragi/runtime-bun#filesystem`と
`seseragi/runtime-node#filesystem`を同じContractへ解決します。両providerはhost file handleをABI外へ漏らさず、
read結果をcopied Bytesとして返します。bridgeはhandle ownerを検査し、取得時のEffect cancellation scopeへcloseを
登録します。cancellation cleanup、明示close、provider shutdownが競合してもhost closeは一回だけで、close後のreadは
resource-closed defectです。今回はopen/read/closeだけを実装し、directory、write、metadata、Streamは含めません。

`provider-contract-schema-1/http-server/contract.json`は、async handler callbackとserver resourceを
portableな論理型として検証します。`listen`は`ListenRequest`からopaqueな`ServerHandle`を取得し、`close`は
その論理handleを冪等に解放します。request / response recordやJSON helperはapplication APIとRuntime ABIが
所有し、ContractはBunの`Server`、`Request`、`Response`を公開しません。

`provider-contract-schema-1/http-client/contract.json`は、同じportable `HttpClient#send`をBun / Nodeの
異なるproviderへ解決できることを検証します。request / responseはnormalized header pairとcopied Bytesを持つ
closed valueで、response bodyの消費中にEffectがcancelされた場合もhost requestをabortし、typed failureへ
変換しません。この最小sliceはbody全体を一度に読むため、pull streamingは後続scopeです。

この四例が同じschemaを使えるため、Contract vocabularyはJavaScript Promiseやhost objectを前提にしません。
一方、`resource`のcleanup、`sleep`のcancellation、closeの冪等性は未確定のままにせず、後続のEffect /
resource contractで定義します。

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

## 15.10 Provider manifest

provider packageは、提供する各serviceを一つのclosed `provider.json` artifactとして公開します。packageの
`seseragi.toml`はartifactへのpackage-root相対pathだけを宣言し、artifact内のserviceやtargetをTOMLへ
重複記載しません。

```toml
[provider]
artifacts = ["providers/clock.json", "providers/http-client.json"]
```

built-in providerも同じ`provider.json` schemaを使います。違いはartifactの発見元がtoolchain catalogか、
root packageの直接dependencyかだけです。built-in専用の暗黙Contractや選択優先度は持ちません。

```json
{
  "schema": 1,
  "kind": "runtime-provider",
  "identity": "seseragi/runtime-bun#clock",
  "service": "std/clock::Clock",
  "contractVersion": { "major": 1, "minor": 0 },
  "backend": { "family": "typescript", "abiMajor": 1 },
  "targets": ["bun-process"],
  "entry": {
    "module": "seseragi/runtime-bun/clock",
    "export": "provider"
  },
  "requires": {
    "runtimeFeatures": ["foreign.task-load"],
    "hostPackages": []
  }
}
```

provider identityは`<package name>#<kebab-name>`です。package versionとsource identityは11.2のresolved
package identityから得るため、identity文字列へversionを埋めません。同じresolved package内でidentityは一意で、
一つのartifactは一つの`service`だけを提供します。複数serviceを実装するpackageはartifactを分けます。

11.10のproject target resolverが選ぶ`process` / `web`はapplicationとcommandのlogical targetです。provider manifestの
`targets`にある`bun-process`等はtoolchain adapter identityで、logical target選択後にtoolchainが対応adapterへ写像します。
target selectionとprovider selectionを同一のfallback処理へ統合せず、provider不足を別logical targetへの切替で隠しません。

- `service`: 15.2のContract identity。
- `contractVersion`: providerが実装するservice Contractのmajor / minor。
- `backend`: backend familyと、そのbackend Runtime ABI major。
- `targets`: 実行可能なtarget IDの非空・重複なし集合。
- `entry`: provider packageが公開するcanonical module specifierとbackend上のexport名。
- `requires.runtimeFeatures`: target toolchainがbuild時に提供すべきversioned runtime feature ID。
- `requires.hostPackages`: providerのforeign host dependency名とversion range。exact解決結果はhost lockfileに従う。

entry moduleの評価mode、値表現、operation call protocolはbackend ABIが所有します。manifestの`entry`だけから
moduleをeager loadしたり、operationをPromiseと推測したりしません。

## 15.11 Requirement収集と候補の可視性

resolverはlinked programの公開`main`が持つclosed Effect environmentからservice requirementを収集します。
使用されるtransitive dependencyのEffect requirementも型を通してこのenvironmentへ合成され、各requirementは
最初に導入したpackage / module / source rangeのtraceを保持します。dependency graphに存在するだけで使われない
serviceは要求へ加えません。

同じservice majorを要求する複数traceは、必要minorの最大値へ統合します。異なるmajorを同時に要求した場合は
providerを選ぶ前に`provider.requirement-conflict`です。一方のdependencyを黙って優先しません。

候補として可視なのは次だけです。

1. 選択targetのtoolchain catalogに登録されたbuilt-in provider artifact。
2. root packageが直接dependencyとして宣言したpackageのprovider artifact。

transitive libraryはservice requirementを追加できますが、providerをapplicationの代わりに選べません。
transitive dependencyにprovider artifactが含まれていても、rootの直接dependencyでない限り候補へ自動昇格
しません。これにより、通常のlibrary追加だけでprocess / network / filesystem implementationが変わることを
防ぎます。

## 15.12 Compatibility filter

一つのrequirementに対する候補は、次を上からすべて満たす必要があります。

1. `service`がrequirementのcanonical identityと一致する。
2. 選択targetが`targets`に含まれる。
3. Contract majorが一致し、provider minorがrequired minor以上である。
4. backend familyとRuntime ABI majorがtoolchainの選択backendと一致する。
5. `requires.runtimeFeatures`をtarget artifactがすべて提供する。
6. `requires.hostPackages`がforeign host resolverとlockfileで一意に解決済みである。

minorをadditive-compatibleと扱える条件とABI handshakeの詳細は15章のversion互換性節で確定します。それまでは
同major・provider minor以上をschema 1の選択前提とし、major違いを黙ってadapter変換しません。

候補を落とした理由はcandidate rejectionとして保持します。一件のproviderが複数条件に違反しても、target、
Contract、ABI、runtime feature、host packageの順で最初のactionableな理由をprimaryにし、残りをnotesへ残します。

## 15.13 Deterministic selection

compatibility filter後の選択順は次で固定します。

1. root manifestのexplicit selection。
2. target toolchainのdefault provider。
3. compatible候補が正確に一件ならその候補。

root manifestはservice identityごとにprovider identityを指定できます。

```toml
[providers]
"std/http::HttpClient" = "acme/undici-provider#http-client"
```

これはapplication sourceへ実装identityを露出させる仕組みではなく、deployment / toolchain configurationです。
library packageは`[providers]`を書いてconsumerの選択を固定できません。

explicit selectionまたはtoolchain defaultがある場合、resolverはそのidentityだけを検査します。見つからない、
target / Contract / ABI / runtime featureが不適合であっても、別候補へfallbackしません。指定なしでcompatible候補が
0件ならmissing、2件以上ならambiguousです。package version、dependency order、filesystem order、provider identityの
lexicographic orderをtie-breakerとして使いません。

## 15.14 事前diagnostic

provider resolutionはgenerated moduleのstageやentry moduleの評価より前に完了します。error payloadは最低でも
service identity、required Contract version、requirement trace、target、backend / ABI、候補identity、候補ごとの
rejection理由、explicit / default selectionの出所を持ちます。

| code | label | 条件 |
| --- | --- | --- |
| `SES-K0201` | `provider.missing` | service候補が一件もない |
| `SES-K0202` | `provider.ambiguous` | 指定なしでcompatible候補が複数ある |
| `SES-K0203` | `provider.target-mismatch` | 選択providerがtargetを提供しない |
| `SES-K0204` | `provider.contract-mismatch` | service Contract major / minorが不適合 |
| `SES-K0205` | `provider.abi-mismatch` | backend familyまたはABI majorが不適合 |
| `SES-K0206` | `provider.runtime-feature-mismatch` | target runtime featureまたはhost packageが不足 |
| `SES-K0207` | `provider.requirement-conflict` | transitive requirementのContract majorが衝突 |
| `SES-K0208` | `provider.selection-unavailable` | explicit / default identityが候補として不可視 |

target不一致は既存のtarget capability検査と競合しません。target registryはまずprogramが要求するserviceをtargetが
原理的に扱えるかを検査し、その後provider resolverが具体provider artifactを選びます。たとえばprocess targetで
DOMだけを要求するprogramは従来どおりtarget mismatch、Bun processが扱えるHttpClientにproviderがない場合は
`provider.missing`です。どちらも未提供fieldへ`undefined`を注入せず、実行前にexit code 2で停止します。

## 15.15 Lock / build metadataと具体例

`seseragi.lock`は選択providerについて、provider identity、resolved packageのexact version / source identity /
content digest、provider artifact digest、service / Contract version、backend / ABI、target、entry module、host packageの
exact identityを固定します。build artifactは同じ情報とruntime feature集合を記録し、absolute pathや候補探索順は
記録しません。manifest、lock、artifactのいずれかが変わればstale selectionとしてbuild前に拒否します。

machine-readable fixtureは次の異なる候補を同じschemaで検査します。

- `bun-clock`: host packageを持たないClockのtoolchain default候補。
- `bun-http-server`: Bun processの組み込みlistenerを使うHttpServerのtoolchain default候補。
- `bun-http-client-native` / `node-http-client`: 同じHttpClient Contractを各processの組み込みfetchへ接続する候補。
- `bun-http-client`: Bun / Nodeの両targetでexternal `undici`を使うHttpClient候補。
- `bun-filesystem` / `node-filesystem`: 同じFileSystem Contractを各processの組み込みfilesystemへ接続する候補。

conformance modelは、HTTP候補が複数ある場合のexplicit / default / ambiguity、FileSystemの一意選択、Clockの
target / Contract / ABI / runtime feature mismatch、候補なし、不可視explicit selection、transitive major conflictを
固定します。このmodelはcompiler resolver実装ではなく、後続実装が満たす選択contractです。

## 15.16 TypeScript Provider Runtime ABI v1

TypeScript backendでProvider Contractを実行する境界のidentityは
`seseragi/provider-abi/typescript`、ABI majorは`1`です。machine-readableなclosed artifactは
`examples/spec/artifacts/provider-typescript-abi-schema-1/core/abi.json`を正とします。

このartifactは15.3のservice Contract versionとも、`runtime-schema-1/core/abi.json`とも別物です。後者は
generated moduleが`@seseragi/runtime`からimportするfeature registryであり、本節のartifactはprovider entryへ
渡す値、operation call、result、handleを定義します。同じTypeScript backendかつ同じmajorでも、consumerは
identityを取り違えてはなりません。

provider manifestの`backend.family = "typescript"`かつ`backend.abiMajor = 1`を選んだ後、bridgeはprovider
artifactのABI identity / majorと完全一致する実装だけをloadします。minor negotiationをこのABI v1へ先回りして
追加しません。不一致は15.14の`provider.abi-mismatch`でentry moduleの評価前に拒否します。

## 15.17 論理値のTypeScript投影

Contract schema 1の論理型kindは次のTypeScript境界値へ投影します。bridgeはinputをencodeし、provider resultを
decodeするたびに表の検査を行います。

| 論理型 | ABI値 | 検査・ownership |
| --- | --- | --- |
| `unit` | `undefined` | 正確に`undefined` |
| `never` | `never` | 値が現れればboundary defect |
| `primitive: bool` | `boolean` | `typeof`が`boolean` |
| `primitive: bytes` | `Uint8Array` | 両方向でsnapshot copy |
| `primitive: float` | `number` | `typeof`が`number`、NaN / infinity / signed zeroを保持 |
| `primitive: int` | `number` | safe integerを検査し、`-0`を`0`へ正規化 |
| `primitive: string` | `string` | `typeof`が`string` |
| `array` | `ReadonlyArray<unknown>` | 要素を再帰decodeし、新しいimmutable列へsnapshot |
| `record` | readonly plain object | 宣言fieldだけをown propertyとして持ち、再帰decodeしてsnapshot |
| `named` | registered codecの入力値 | canonical type identityで選んだbridge / std codecが検査 |

providerが返したobjectやarrayをSeseragi値として参照共有しません。`Bytes`もforeign境界と同じcopy規則を使い、
zero-copy viewはv1に存在しません。recordはprototype、accessor、symbol field、未知fieldを意味へ含めず、decode時に
getterを実行しません。named codecはtype identityごとにbackend projectorが生成またはruntime / std packageが提供し、
provider packageがSeseragi struct、ADT、Effect、Streamの内部表現を直接構築することを許可しません。

## 15.18 `null`、`undefined`、missing

三つを同じ「値なし」として扱いません。

- `undefined`は`unit`のABI値としてだけ受理します。別logical typeのfieldへ現れればinvalid valueです。
- `null`は既定ではどのlogical kindにも変換しません。named codecが公開型の意味として明示した場合だけ、そのcodecが
  解釈できます。`null`から`Maybe`を推測しません。
- missingはrequired record fieldのown propertyが存在しない状態です。presentなfieldの値が`undefined`であることと
  区別し、どちらもfield typeに従って個別に検査します。

Contract schema 1のrecord fieldはすべてrequiredです。将来optional fieldを追加する場合もpresenceを先に検査してから
値をdecodeし、`null` / `undefined`へ畳みません。これらを曖昧に受理したprovider固有の慣習をstd APIの意味へ持ち込んでは
なりません。

## 15.19 Operation callとresult envelope

manifestの`entry.export`は、operation名をreadonly function memberとして持つobjectです。bridgeはContractのinput全体を
一つのABI値へencodeし、該当memberを正確に一回呼びます。`unit` inputでもargumentを省略せず`undefined`一個を渡します。

ABI v1のoperation memberは必ず`Promise<ProviderResult>`を返します。同期値を返すproviderを暗黙に
`Promise.resolve`で受理しません。戻り値は次のclosed envelopeだけです。

```ts
type ProviderResult =
  | Readonly<{ kind: "success"; value: unknown }>
  | Readonly<{ kind: "failure"; failure: unknown }>;
```

`success.value`と`failure.failure`はContractの論理型でdecodeします。`failure`は宣言済みtyped failureのABI値であり、
bridge / std wrapperがSeseragi failure値へ変換します。providerが`defect` variantを通常resultとして返すこと、exceptionを
typed failureへ変換すること、Promise rejectionを`failure`として扱うことはできません。

bridgeが観測する同期throw、Promise rejection、Promiseでないreturn、malformed envelope、success / failure payloadの
decode失敗は、すべて`provider-boundary` defectへ変換します。内部runnerへ渡すbridge outcomeは
`success` / `failure` / `defect`を区別しますが、`defect`はproviderが返す`ProviderResult` variantではありません。

defect metadataはprovider、service、operation、`input | call | result` stage、短いmessageを持ちます。同期throwまたは
Promise rejectionの元のhost valueは`cause: unknown`として保持し、文字列化だけで置換しません。ただしcauseを
applicationのtyped failure型へcastしたり、JSON serializableだと仮定したりしません。

## 15.20 Opaque handle

resource successのhost tokenは、readonly branded objectへ包んでbridgeへ返します。v1 handle metadataは次だけです。

- opaqueなhost object token
- 選択provider identity
- canonical service identity
- canonical logical handle type identity

bridgeはhandleを受けるoperationごとにbrand、provider owner、service、logical typeを検査してからtokenをproviderへ戻します。
別provider、別service、別handle typeへ渡すこと、tokenをserialize / clone / inspectすること、Contractが宣言しない
ownership transferを行うことを拒否します。close、scope、idempotence、partial acquire、leak検出の意味は次のresource
contractが所有し、本節はその意味をTypeScript object shapeから推測しません。

## 15.21 変換責任

一回のcallでは責任を次の順に固定します。

1. std / package wrapperが公開値の正規化とapplication向けdecoderを所有する。
2. generated / shared bridgeがContract logical typeとABI値のencode / decode、copy、envelope、handle検査を行う。
3. providerがABI値をhost API値へ変換し、host APIを呼び、ABIのsuccess / failureへ戻す。
4. bridgeがhost throw / rejectionとinvalid boundary valueをdefectとして隔離する。

providerはstd内部型をimportして構築せず、bridgeはHTTP statusやSQL error等のservice固有意味を発明せず、std wrapperは
provider identityやhost objectへ依存しません。これによりbackend ABIを差し替えてもProvider Contractとapplication APIを
再設計しない境界を保ちます。

## 15.22 異なるcapabilityへの投影

- Clock `now`: inputは`undefined`、successは`std/time::Instant`のregistered codecが読むclosed valueです。timer objectや
  `Date`を公開値として返しません。
- HTTP client `send`: wrapperが正規化したheader pair列とrequest recordをbridgeがsnapshotし、bodyはコピーした
  `Uint8Array`です。responseもclosed recordとしてdecodeし、providerがstd Responseを構築しません。
- filesystem `openRead`: Pathはnamed codecでencodeし、successはfilesystem provider所有のopaque handleです。後続readへ
  他providerのhandleを渡せません。
- PostgreSQL `query`: query input、row列、database failureはpackage Contractのlogical typeとregistered codecで投影します。
  driver固有class instance、symbol、prototypeをapplication recordとして漏らしません。

値だけのClock、structured bytesを持つHTTP、lifetimeを持つfilesystem、external driverのrow / failureを同じcallとvalue
boundaryで表せます。streaming body、cursor、cancellation、cleanupのprotocolは後続節に委ね、ここでPromise一回完了の
ふりをさせません。

## 15.23 ABI v1の拒否条件

consumerはunknown field、未知logical kind、欠けたvalue mapping、重複mapping、identity / backend / ABI major不一致、
sync return、malformed result tag / payload、共有mutable Bytes、未登録named codec、recordのmissing / unknown field、
不正handle owner / typeを実行境界で拒否します。entry評価前に分かるartifact不整合はbuild error、call後にしか分からない
host value不整合は`provider-boundary` defectです。どちらもprovider都合でtyped failureへ追加しません。

## 15.24 Provider lifecycle contract

provider operationをEffectへ接続する規範は5.9〜5.11を正本とし、本節はその意味をProvider Contract / Runtime ABIへ
投影します。machine-readableな共通fixtureは
`examples/spec/artifacts/provider-lifecycle-schema-1/core/contract.json`です。identityは
`seseragi/provider-lifecycle`、schema / contract versionはともに1です。

このartifactはservice operationへEffect、cancellation、resourceの既存意味を適用するcontractであり、別のeffect
systemやprovider固有schedulerを導入しません。具体operationのservice仕様は、cancellation modeを`cooperative`または
`unavailable`として選びます。providerごとの都合で同じportable Contractのterminal outcomeを変えてはなりません。

## 15.25 Cold Effectと一回の開始

service wrapperの呼び出しはcold `Effect<R, E, A>`を構築するだけで、provider object取得、argument encode、operation
member呼び出し、Promise作成を行いません。そのEffectをhost runnerまたは別Effectが実行した時点で初めてbridgeが
provider operationを開始します。

一回のEffect runにつきprovider memberは正確に一回だけ呼びます。Effect valueを別runとして再実行すれば、それぞれが
新しい一回のoperationです。retry、memoization、deduplication、timeoutはprovider bridgeの暗黙policyではなく、公開APIや
applicationが明示するEffect compositionです。providerが同期に副作用を開始するconstructorやgetterをentry objectへ置くことも
禁止します。

## 15.26 Terminal outcomeの分類

一回のrunは次の四つのterminal outcomeを混同しません。

| 観測 | Effect outcome |
| --- | --- |
| ABIの`success`とvalid payload | success `A` |
| ABIの`failure`とvalid declared payload | typed failure `E` |
| synchronous throw / Promise rejection | defect |
| malformed envelope / invalid boundary value / invariant違反 | defect |
| caller cancellationがraceに勝つ | cancellation |

typed failureはContractのfailure logical typeに宣言された回復可能なdomain / I/O failureだけです。bridgeやproviderが
exception textから`E`を推測しません。defectは`recover`、`mapError`、retry-on-typed-failureへ入りません。
cancellationは`E`のconstructorでも`defect`でもなく、5.11のcooperative cancellationとして伝播します。

## 15.27 Cancellation通知と競合

operation runは`running`から一つのterminal stateへだけ遷移するlinearization pointを持ちます。

- valid result / defectが先にterminal stateをcommitした後のcancel requestは、その完了結果を変更しません。
- cancel requestが先にcancellationをcommitした場合、後続result / throw / rejectionをcaller outcomeにしません。
- bridgeはcooperative operationのprovider cancellation hookをrequest後に高々一回呼びます。重複cancelはno-opです。
- cancellation後のhost Promiseはunhandled rejectionにせず最後までobserveし、late completionとしてdiscardします。
- acquireがcancellation後に成功した場合は、handleを公開せずreleaseを完了してからdiscardします。

「同時」はwall-clock時刻で決めず、bridge terminal stateへの最初のcommitで一意に決めます。late defectは既に決まった
typed outcomeへ変換せずdiagnostic noteへ保持します。ただしlate acquireで必要なrelease defectはscope cleanup defectとして
扱い、resourceを黙って失いません。

`cancellation: unavailable`のoperationではbridgeは存在しないhost abortを捏造しません。Effect outcomeはcancel requestで
cancellationをcommitできますが、host Promiseをsupervised taskとしてobserveし続けます。late success / failureは捨て、
late resource successだけはreleaseします。このsupervision自体を新しいunscoped Fiberとしてapplicationへ露出しません。

## 15.28 Resource acquireとhandoff

Contractで`kind: resource`のoperationは、valid handle取得とcurrent scopeへのfinalizer登録をcancellationに対してatomicに
行います。次のどちらかだけが起きます。

1. acquireが失敗 / defect / cancellationし、公開handleも登録済みfinalizerもない。
2. handleが成功し、対応release finalizerが登録済みでuseへ渡る。

provider内部で複数host resourceを取得してからhandleを返す場合、公開前のpartial initialization stateはprovider自身が
逆順にcleanupします。bridgeは受け取っていないtokenを推測してcloseしません。partial cleanupのthrow / rejectionは元の
acquire failureへtyped errorとして混ぜずdefect metadataへ残します。

公開handleの`close` Effectはidempotentです。最初のcloseだけがprovider releaseを正確に一回開始し、並行または後続closeは
同じ完了を待つか既完了Unitを返します。close開始後にそのhandleで新operationを開始するとresource-closed defectです。

## 15.29 Scope cleanupとshutdown順序

releaseはuseのsuccess、typed failure、defect、cancellationのすべてで走り、一つのscope内では登録のLIFO順です。
resourceにchild operation / resourceがある場合、shutdownは次の順です。

1. 新規operation受付を停止する。
2. 未完了childへcooperative cancellationを要求する。
3. childのlate completionとchild finalizerを待つ。
4. parent provider handleをreleaseする。
5. parent scopeを閉じる。

HTTP serverはrequest childrenを止めてからlisten handleを閉じ、database poolはcursor、checked-out connection、poolの順に
閉じます。filesystem handle単体でも同じscope規則を使います。process shutdownはroot scopeへのcancellationで開始し、
11.1のgrace period内はこの順序を変えません。forced terminationだけが保証外です。

finalizerはtyped failureを持ちません。releaseがthrow / reject / invalid resultで失敗しても残りのfinalizerを実行し、最初の
cleanup defectをprimary、後続cleanup defectをordered notesにします。useのtyped failureよりcleanup defectを優先しますが、
元のoutcomeをcause / noteとして保持します。この規則は5.11と同一で、providerのclose APIに合わせて逆転しません。

## 15.30 Causal metadata

bridgeはprovider identity、service / operation identity、Contract / backend ABI version、source call range、run ID、lifecycle
stage（`acquire | use | cancel | release`）、host causeをdefect metadataへ付けます。providerはSeseragi source rangeを
捏造せず、generated bridgeがcompiler metadataから対応付けます。host stackはcross-language frameとして保持し、typed failureの
公開payloadへ埋め込みません。

## 15.31 異なるlifecycleの検証

- Clock `sleep`: resourceを持たないcold one-shotで、timer cancellationを高々一回通知します。
- filesystem `openRead`: cancel不能なhost openがlate successした場合も、file handleをreleaseしてから捨てます。
- HTTP server `listen`: acquireしたserver handleをscopeへatomic登録し、shutdown時はrequest childrenのcleanup後に閉じます。
- PostgreSQL pool: partial acquireはproviderが接続を逆順cleanupし、公開後はcursor / connection / poolの親子順を守ります。

one-shotとlong-lived resource、cancel可能 / 不可能、単一handle / child graphを同じterminal outcomeとscope規則で表せます。
retry、timeout、protocol engine、process-wide manager、leak detector実装、Stream backpressureは本contractへ混ぜません。

## 15.32 Lifecycle schema 1の拒否条件

consumerはunknown field、cold construction以外、run中の複数provider start、terminal outcomeの重複commit、cancellationの
typed failure化、重複cancel notification、late completionの未観測、late acquire handleの未解放、non-idempotent close、
success / failure / cancellationでreleaseを省くこと、scope内FIFO cleanupをcontract違反として拒否します。

## 15.33 Provider callback / stream contract

host callback、subscription、body chunk、cursor rowをprovider固有objectのまま公開せず、15.17のvalue boundaryと
10.12の`Stream<R, E, A>`へ接続します。machine-readableな最小contractは
`examples/spec/artifacts/provider-stream-schema-1/core/contract.json`、identityは
`seseragi/provider-stream`、schema / contract versionは1です。

このcontractはproviderとshared bridgeのevent protocolだけを所有します。Streamの公開combinator、ordering、failure、
resource semanticsは10.12、Signal subscriptionは5.15が正本です。providerがStream / Signal / Subscriptionのruntime内部表現を
importして直接構築しません。

## 15.34 one-shot callbackとmulti-shot callback

host callbackは登録前にkindを固定します。

- one-shot callbackは正確に一件のterminal callbackを受けます。callback APIを15.19のone-shot Promise resultへadaptする
  場合に使い、複数回呼ばれれば最初だけをcommitして後続をprovider protocol defectとして記録します。
- multi-shot callbackは0件以上の`next`の後に、`complete`、declared typed failure、defectのいずれか一件だけで終了します。
  terminal後の`next`や二件目のterminalはcallerへ届けず、observeしてdiscardします。

callbackが一件か複数件かを実行中の回数から推測しません。one-shot operationをmulti-shot subscriptionとして公開したり、
multi-shot callbackの最初のeventだけをPromiseへ変換して残りをleakさせたりしてはなりません。

## 15.35 登録と解除

bridgeはhost registerを呼ぶ前にcallback receiver、terminal state、正のcapacityを持つregistration queueを準備します。
hostがregister中に同期callbackを呼んでも取りこぼさず、registration successをcommitするまではapplication consumerへ
deliveryしません。

registrationがtyped failure、defect、cancellationで終わった場合は、得られたdetach tokenがあれば解除し、queue済みeventを
discardしてSubscriptionを公開しません。registration successとscope finalizer登録のhandoffは15.28と同じくatomicです。

unsubscribeはidempotentなEffectで、host detachを正確に一回開始します。consumer cancellation、Stream terminal operationの
早期終了、scope終了はいずれも新規demandを止め、unsubscribeし、bufferを破棄してから終了します。unsubscribe後のhost
callbackもreceiver自体はobserveしますが、decodeやapplication callbackを再開せずlate eventとしてdiscardします。

## 15.36 Demandと最小backpressure

provider stream bridgeのdemandは正の整数countです。outstanding demandはoverflowを検査した非負整数として保持し、
`next`を一件deliveryするたび一件減らします。

- pull sourceはoutstanding demandを超えてeventをemitできません。HTTP body readとdatabase cursor fetchは、downstreamの
  demandが0なら新しいhost read / fetchを開始しません。
- push sourceはhostを完全停止できない場合があるため、公開wrapperが正の有限capacityとoverflow policyを明示してから
  registerします。unbounded queueはありません。
- providerがpause / resumeを正確に実装すると宣言した場合だけ、満杯時にproducerをsuspendするlossless modeを使えます。
  callback threadを任意にblockしてsuspendを捏造しません。

producerの通常completeではbufferをFIFOでdrainしてから完了します。producer typed failure / defectはbuffer値より優先し、
bufferを破棄してterminalを通知します。この規則は10.12のbuffer semanticsと一致します。

## 15.37 Overflowとprotocol violation

capacity到達時の選択肢は公開APIが明示した次だけです。

- lossless suspend: providerがpause / resumeを提供する場合。
- drop oldest / latest: application-facing APIがlossy strategyを明示した場合。
- fail: service Contractのfailure typeがBufferOverflow相当を宣言した場合だけtyped failure。

providerがdemand超過eventを送る、capacityを無視する、不正なevent envelopeを送る、terminal後に無制限に送信し続けることは
provider protocol defectです。宣言にないoverflowをapplicationのtyped failureへ追加しません。逆に、公開Contractが通常の
backpressure overflowを回復可能failureとして宣言したなら、bridge都合でdefectへ格上げしません。

## 15.38 Producer / consumer cancellation

consumer cancellationはoutstanding demandを0にし、未開始pullを起動せず、開始済みhost operationとsubscriptionへ
cancellation / unsubscribeを高々一回通知します。bufferはdiscardし、late event / terminalは15.27と同じくobserveして
caller outcomeへ入れません。unsubscribe中のdefectはcleanup defectとして保持します。

producerが通常completeまたはtyped failureで終われば、consumer側の未使用demandは消滅します。producer terminalが先に
commitした後のconsumer cancelはterminalを変更しません。consumer cancelが先なら後続producer terminalはlateです。
providerがconsumerのStream scopeを越えてcallbackを保持することを禁止します。

## 15.39 Stream / Signalへの変換責任

providerはABI event envelopeとhost detach / demand hookだけを実装します。shared bridgeがcallback linearization、value
decode、bounded queue、demand accounting、unsubscribeを所有し、std / package wrapperがそれを公開`Stream`または`Signal`へ
変換します。

Signalへ変換するwrapperは5.15のcurrent value、transaction、observer serializationを守り、provider callbackを直接Signal
subscriberとして登録しません。Streamへ変換するwrapperは10.12のcold再実行、failure type、operator semanticsを守ります。
同じStream descriptionを二度runすれば別subscription / scopeを作り、provider subscriptionを共有しません。

## 15.40 Capability境界

- HTTP request / response body: pull modeで一demandにつき高々一つのcopied Bytes chunkをreadします。full HTTP framingや
  compressionはprovider / protocol layerの責務で、Stream contractへ入れません。
- PostgreSQL cursor: row demandに従ってfetchし、cancel / early terminationでcursor handleをcloseします。row decode failureは
  declared database failureまたはboundary defectとして分類します。
- SSE: push modeの有限bufferと明示overflow policyを使います。reconnect / Last-Event-ID APIはここでは確定しません。
- WebSocket: message callbackの登録・解除・bounded deliveryだけを表し、handshake、frame、ping、close code、fan-outは
  future protocol contractです。

HTTP bodyとdatabase cursorでlossless demandが成立し、停止できないSSE / WebSocket callbackにも有限bufferを要求できます。
この共通部分を固定することは、full WebSocket / SSE API、高度なreplay / fan-out、transport固有flow control、distributed
backpressureを現在実装したことを意味しません。

## 15.41 Stream schema 1の拒否条件

consumerはunknown field、callback kind未宣言、one-shotの重複callback、multi-shot terminal後event、非atomic registration、
non-idempotent unsubscribe、0 / 負 / unbounded capacity、pull demand超過、未宣言dropping / failing policy、late event再delivery、
providerによるStream / Signal内部値の構築をcontract違反として拒否します。

## 15.42 Portable surfaceとtarget extension

portable serviceは`std/http/server::HttpServer`のようなtargetを含まないcanonical identityと
`portability.kind: portable`を使います。target extensionはportable Contractへoperationを足さず、
`std/http/bun::BunHttpServer`のようなtarget segmentを含む別service / moduleから明示importします。そのoperationは
`portability.kind: target-extension`と同じ`target`を持たなければなりません。

portable markerなのにservice moduleが`bun` / `node` / `browser`等のtarget namespaceを含む場合、またはextension markerの
targetがmodule identityにない場合、Provider Contract validatorが拒否します。
`provider-contract-schema-1/bun-http-extension`はこの機械検査を固定します。

source / interface / build metadataはimport closureからpackage portabilityを導出し、portableまたはsorted target集合として
表示します。宣言だけでportableを名乗れません。target extensionをpublic exportするlibraryはそのtarget集合をconsumerへ
伝播し、別target buildではimport rangeを指す事前diagnosticにします。provider identityやABI entryはapplication importへ
現れません。

## 15.43 独立したversion role

次のversionは交換可能ではありません。

- artifact schema: JSON fieldの読み方。consumerが対応するschema majorだけを読む。
- service Contract: operationと意味。same majorかつprovider minorがrequired minor以上。
- backend ABI: TypeScript / 将来backendのvalue / call shape。major完全一致。
- runtime package:実装SemVer、source identity、content digestをlockfileでexact固定。
- compiler: artifact producer / consumer feature supportを明示handshake。

package SemVerやcompiler versionから他のversionを推測しません。handshake順はartifact schema、target extension、service
Contract、backend ABI、runtime package、compiler feature、provider conformanceです。前段不一致でentryを評価せず、
後段fallbackで別providerを黙って選びません。

## 15.44 Additive / breaking change

同じmajorでadditiveなのは、旧consumerが意味を誤読しないoptional artifact metadata、新しい別target extension、既存operationを
変えないservice minor operation追加、既存semanticsを具体化するconformance case追加です。

operationの削除 / rename、logical input / success / failureやterminal outcome変更、cancellation / cleanup / backpressure保証の
弱化、ABI value / call shape変更はbreakingです。新required JSON fieldを同schema majorへ追加することもbreakingで、closed
schemaをoptional扱いして回避しません。breaking changeは該当Contract / ABI / schema majorを上げます。

## 15.45 Handshake diagnostic

既存`SES-K0204 provider.contract-mismatch`と`SES-K0205 provider.abi-mismatch`に加え、次を使います。

| code | label | 条件 |
| --- | --- | --- |
| `SES-K0209` | `provider.extension-mismatch` | importしたtarget extensionと選択targetが不一致 |
| `SES-K0210` | `provider.runtime-mismatch` | locked runtime package identity / digestがartifactと不一致 |
| `SES-K0211` | `provider.compiler-mismatch` | compilerがrequired producer / consumer featureを未対応 |
| `SES-K0212` | `provider.conformance-mismatch` | required conformance profile / resultが不足またはstale |

payloadは全version identity、required / actual、provider / target、manifest / lock / artifact path、source import rangeを保持します。
どのmismatchもapplication開始前にexit code 2で停止します。

## 15.46 Provider conformance case model

provider conformance profileは最低でもsuccess、declared typed failure、defect、cancellation、cleanup、concurrent operation、
invalid boundary value、target mismatch、ABI mismatch、provider ambiguityを独立caseとして持ちます。caseはContract operation
identity、seed / virtual time、input、expected event trace / terminal、cleanup後active handle数を記録できます。

型のshapeだけ一致しても、cancel後event、cleanup漏れ、demand超過、failure / defect混同があれば不適合です。すべての
capability実装をこのsliceで作りませんが、後続runnerが同じcase identityと結果分類を使えるclosed modelを
`provider-compatibility-schema-1/core`で固定します。

## 15.47 Backend差し替え

BunとNodeは同じProvider Contract / application APIを使い、provider manifest entryとTypeScript runtime packageだけを
差し替えます。将来Wasm / nativeを追加する場合も新backend ABI projectionを定義し、service identity、Effect / Stream意味、
application importを変更しません。target extensionを使うsourceだけは明示target dependencyを持つため、portable sourceと
同じだとは表示されません。

future backend用ABIを今定義・実装すること、registry migrationやcertification serviceを運用することは対象外です。

## 15.48 Compatibility schema 1の拒否条件

consumerはportable / extension markerとmodule identityの不一致、handshake順の変更、version roleの混同、breaking changeの
minor扱い、required conformance case欠落、runtime digest不一致、backend差し替えでapplication Contract identityが変わることを
拒否します。

## 15.49 5 capabilityによる最終検証

確定した4層を性質の異なるserviceへ適用した結果を次に固定します。

| capability | std / package application API | Provider Contract operation | TypeScript ABI / bridge | provider |
| --- | --- | --- | --- | --- |
| Clock | `std/clock.now` / `sleep` | `std/clock::Clock#now` / `#sleep` | Unit / Instant codec、Promise result、cancel race | monotonic clock / timer |
| HTTP client | `std/http.sendBytes` / `sendEmpty` | `std/http::HttpClient#send` | closed record、copied Bytes、one-shot cancellation | fetchまたはexternal client |
| HTTP server | `std/http/server.listen` / `serveOnce` / `close` | `std/http/server::HttpServer#listen` / `#close` | callback queue、opaque handle、child cleanup | listener / response writer |
| filesystem | `std/fs.readBytes` / `readChunks` | `std/fs::FileSystem#openRead` / `#read` / `#close` | named codec、copied Bytes、owner-checked handle | filesystem / descriptor |
| PostgreSQL | PostgreSQL固有package API | `acme/postgres::Postgres#openPool` / `#query` / cursor operations | driver value codec、pool/cursor handle、row demand | external driver adapter |

Clockは小さいvalue / cancellation、filesystemはopaque resource、PostgreSQLはstd外packageとexternal dependencyを反証例に
するため、共通modelはHTTP objectやBun APIへ過適合していません。通常application APIにBunProvider / NodeProvider、entry
module、ABI identityは現れず、manifest / toolchainがproviderを選びます。

各例はmissing / ambiguous / target / Contract / ABI mismatchを15.14 / 15.45で開始前に拒否し、success、typed failure、
defect、cancellation、cleanup、concurrency、invalid value、mismatch、ambiguityのconformance caseへ写像できます。
machine-readableな対応表と実装handoffは`provider-design-validation-schema-1/system/validation.json`です。

この表の左列とContract operationは同じnamespaceではありません。たとえばHTTP small-response wrapperは
`sendBytes`と`sendEmpty`を一つの`HttpClient#send`へ投影し、filesystem wrapperは複数回のopen / read / closeを
組み立てます。`std/http.exchange`のbody Streamはone-shot `#send`の別名ではなく、15.33〜15.40を満たす
subscription operationをContractへ追加してから接続します。`acme/postgres`はProvider Systemの反証用identityであり、
標準`Database` application APIや将来の配布package名を予約しません。

PostgreSQLの実装sliceは`pg`と`pg-cursor`をhost packageとしてmanifestへ宣言し、wire protocolを所有しません。
`acme/postgres::Postgres` Contractはpoolとcursorを別のopaque resourceとして扱い、query / fetchのrowをclosed logical
recordへcopyします。driver rejectionは`QueryError`へ写像し、未宣言のDateやclass instance等がrowへ現れた場合はtyped
failureではなくresult boundary defectです。cancellation、明示close、provider shutdownはいずれもcursor close、checked-out
connection release、pool endの順を守り、各releaseを一回だけ実行します。

## 15.50 実装Epicへの依存順handoff

実装はContract artifact foundation、manifest / resolution、TypeScript ABI bridge、target diagnostic、runtime provider package
boundary、最小Clock provider、HTTP server縦slice、HTTP clientとNode差し替え、filesystem resource、PostgreSQL external
driver、conformance / authoring guideの順に分割します。前段の共通基盤を後段capabilityへ重複実装しません。

この順序はprotocol engine自作、full streaming / WebSocket、Wasm / native provider、本番registryを現在scopeへ含めません。

## 15.51 設計監査結果

4層責任、backend非依存Contract、TypeScript ABI、manifest / resolution / diagnostic、Effect / failure / cancellation / cleanup、
Stream / callback / backpressure、portable extension、version / conformance、5 capability適用のすべてに規範本文とclosed fixtureが
あります。共通schema変更を必要とする反例は残らず、実装Epicは上の依存順で開始できます。

provider実装を追加するときのartifact、entry、probe、共通profileへの接続手順は
[`docs/PROVIDER_AUTHORING.md`](../PROVIDER_AUTHORING.md)に定めます。実行可能なprofileは
`provider-conformance-profile-schema-1/core/profile.json`で固定し、successからambiguityまでのcaseに加えて、
cleanup後のactive handleを直接観測する`leak` caseを要求します。

## 15.52 application capability再基準化

Provider System実装後に10章、13章、関連fixtureを再監査し、application surfaceからProvider Contractへ
投影する規則を10.2へ集約しました。監査で解消したドリフトは次です。

- HTTP client requirementは実装・Contractと同じ`{ httpClient: HttpClient }`へ統一し、旧`http` fieldを廃止した。
- Clock operationをpure time value moduleから`std/clock`へ分離し、`now` / `sleep`のtyped failureをContractと
  同じ`Never`にした。timer障害やinvalid boundary valueはdefect、sleep中断はcancellationである。
- 最終監査artifactの`applicationApi`と`contract`を別identityとして検査し、service member名をapplication APIと
  呼ばないようにした。
- filesystemの`FileError`から`FileSystemError`へのcontext付加、HTTP small-response wrapper、server handler、
  PostgreSQL package wrapperはapplication層の責務であり、providerが公開型を構築しないことを確認した。
- Stream、callback、resource、browser-only capability、TypeScript foreign bindingは既存の共通lifecycle / target /
  interop境界を再利用し、個別moduleがPromise、AbortSignal、host handle、provider identityを公開しない。

未実装surfaceへ仮のContract operationを予約しません。navigation / storage / WebSocket / SSE等のbrowser
capability、process I/O、full HTTP streaming、database packageは、10.2のidentity・failure・resource規則に従って
各実装IssueでContractを追加します。standard moduleの存在・export・availabilityのmachine-readable SSOTは#359、
fixtureの実装状態分類は#363、target既定値とCLI overrideは#364が所有し、本再基準化で別の正本を作りません。
