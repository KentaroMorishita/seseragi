# SQLite application package

本章は標準ライブラリではない公式package `seseragi/sqlite` の公開契約を定める。
SQLiteの単一connection、file / memory database、parameter binding、transaction、busy failureを
保持し、PostgreSQLや将来のdatabase packageと万能な`Database` APIへ統合しない。

## 17.1 identityとrequirement

applicationはmanifestで`seseragi/sqlite`へdependencyを宣言し、export rootをimportする。
packageの`Sqlite`型をEffect requirementの`sqlite` fieldへ置く。

```seseragi
import * as sqlite from "sqlite"

pub effect fn main -> Unit
with sqlite: sqlite.Sqlite
fails sqlite.Error =
  -- ...
```

version付きnominal identity `seseragi/sqlite@VERSION::lib::Sqlite`はProvider解決時だけ
stable service identity `seseragi/sqlite::Sqlite`へ投影する。package SemVerとProvider
Contract versionは独立で、application sourceはprovider identityやentry moduleを指定しない。

## 17.2 databaseとparameter

`Sqlite`、`Database`、`Transaction`、`Row`、`Value`、`Decoder<A>`、
`TransactionProgram<A>`はopaqueである。`openMemory busyTimeoutMillis`はconnection所有の
in-memory databaseを開く。`openFile FileConfig`はfilename、readOnly、create、
busyTimeoutMillisを明示する。filenameはprocess Providerのpath境界で解釈し、working directoryや
symlinkを言語仕様から推測しない。portableな`std/path`型が利用可能になるまではraw Stringである。

`Statement`はSQL textと`Array<Value>`を分離する。値は`textValue`、`intValue`、
`floatValue`、`boolValue`、`bytesValue`、`nullValue`から構築し、SQL文字列へ暗黙補間しない。
BoolはSQLite integer 0 / 1へ写像する。`query`は全logical rowをsnapshot copyし、`execute`は
非負のchangesとSeseragi Int範囲内のlastInsertRowIdを返す。

## 17.3 row decode

RowはJSONではない。`string`、`int`、`float`、`bool`、`bytes`は列名から
`Decoder<A>`を作り、`map2`でstruct constructor等へ合成する。missing column、型不一致、
Seseragi Int範囲外は`RowDecodeFailure RowDecodeError`となる。ProviderがContract外の値を返した
場合はtyped decode failureではなくprovider boundary defectである。Bytesはbind時とresult時にcopyする。

## 17.4 transactionとresource

`transactionExecute`と`transactionQuery`はcoldな`TransactionProgram<A>`を作る。
`transactionThen first second`は同じTransactionでfirst成功後にsecondを実行する。
`transaction database program`は実行時に`BEGIN IMMEDIATE`を開始する。

- program successでは`COMMIT`後に成功値を返す。
- typed failureでは`ROLLBACK`完了後に元のfailureを返す。
- defectまたはcancellationでもmasked cleanupで`ROLLBACK`し、cancellationを`Error`へ変換しない。
- transaction中は親Databaseへの直接query / executeを拒否する。
- 明示close、scope cleanup、Provider shutdownはactive transactionを先にrollbackし、Databaseを一回だけ閉じる。

SQLite driverは同期operationを実行するため、開始済みの一つのdriver callを途中でinterruptできない。
cancellationはoperationの前後で観測し、取得済みresourceを解放してlate resultをapplicationへ渡さない。

## 17.5 failure、target、将来のstream

`Error`はlock contentionを保持する`BusyFailure DriverError`、その他の回復可能なdriver errorを
保持する`DriverFailure DriverError`、application decodeの`RowDecodeFailure RowDecodeError`からなる。
DriverErrorはoperation、driver code、messageを保持する。malformed envelope、別Providerのhandle、
Contract外valueはdefectである。

canonical Contractは`seseragi/sqlite::Sqlite@1.0`である。TypeScript Provider
`seseragi/runtime-sqlite#bun`はBun組み込み`bun:sqlite`をwrapし、`bun-process`だけを対象にする。
browser targetではentry評価前に`SES-K0201 provider.missing`となる。

本章はCursorやStreamを公開しない。`std/stream`完成後に、SQLite row demandとstatement lifetimeを
明示できる場合だけ追加する。現在の全row queryをPromise列や擬似Streamへ読み替えない。
