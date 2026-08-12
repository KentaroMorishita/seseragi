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
- `bun-http-client`: Bun / Nodeの両targetでexternal `undici`を使うHttpClient候補。
- `node-filesystem`: Node targetだけでFileSystemを提供する一意候補。

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
