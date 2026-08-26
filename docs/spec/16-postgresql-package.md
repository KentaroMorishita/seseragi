# PostgreSQL application package

本章は標準ライブラリではない公式package `seseragi/postgres` の公開契約を定める。
全databaseを一つの`Database`型へ統合せず、PostgreSQLのparameter、row、transaction、
cursorの意味を保つ。wire protocolとhost driverはpackage APIではなくProvider実装の責務である。

## 16.1 identityとrequirement

applicationはmanifestで`seseragi/postgres`へdependencyを宣言し、export rootをimportする。
packageの`Postgres`型をEffect requirementの`postgres` fieldへ置く。

```seseragi
import * as postgres from "postgres"

pub effect fn main -> Unit
with postgres: postgres.Postgres
fails postgres.Error =
  -- ...
```

version付きnominal identity `seseragi/postgres@VERSION::lib::Postgres`はProvider解決時だけ
stable service identity `seseragi/postgres::Postgres`へ投影する。package SemVerとProvider
Contract versionは独立であり、application sourceはprovider identity、entry module、host packageを
指定しない。

## 16.2 opaque resourceと値

`Postgres`、`Pool`、`Transaction`、`Cursor`、`Row`、`Value`、`Decoder<A>`、
`TransactionProgram<A>`はopaqueである。applicationはprovider handle、checked-out connection、
driver row、JavaScript objectを構築・検査・serializeできない。

`Config`は`connectionString: String`と正の`maxConnections: Int`を持つ。
`openPool`はcold Effectであり、成功したPoolをcurrent resource scopeへ登録する。明示的な
`closePool`とscope cleanupはidempotentで、transactionとcursorを接続より先に閉じる。

`Value`は`textValue`、`intValue`、`floatValue`、`boolValue`、`bytesValue`、`nullValue`から
構築する。`Query`はSQL textと`Array<Value>`を分離する。値をSQL文字列へ暗黙補間する経路は
存在しない。parameterなしの場合は`emptyValues ()`を用いる。

## 16.3 query resultとrow decode

`query pool query decoder`は`Effect<{ postgres: Postgres }, Error, QueryResult<A>>`を返す。
`QueryResult<A>`はdecoded `rows: Array<A>`、非負の`rowCount: Int`、PostgreSQL command tagの
`command: String`を持つ。

RowはJSONではない。`string`、`int`、`float`、`bool`、`bytes`は列名から
`Decoder<A>`を作る。`Decoder`は`Functor`と`Applicative`のinstanceを持ち、複数列は
`<$>`と`<*>`で通常のcurried constructorへ合成する。

```seseragi
fn person id: Int -> name: String -> active: Bool -> Person =
  Person { id: id, name: name, active: active }

fn personDecoder -> postgres.Decoder<Person> =
  person
  <$> postgres.int "id"
  <*> postgres.string "name"
  <*> postgres.bool "active"
```

`Applicative.pure`はrowを読まずに値を返すDecoderを作る。`<*>`は同じrowに対して左から順に
Decoderを評価し、最初のdecode failureを返す。missing column、型不一致、Seseragi Int範囲外は
`RowDecodeFailure RowDecodeError`であり、JSON decoderのfield semanticsやtagged ADT wire形式を
流用しない。ProviderがContract外のclass instance等を返した場合はtyped decode errorではなく
provider boundary defectである。

## 16.4 transaction

`transactionQuery query decoder`はcold `TransactionProgram<QueryResult<A>>`を記述する。
`transaction pool program`を実行したときだけconnectionをcheckoutして`BEGIN`する。

- program successでは`COMMIT`後にconnectionをreleaseして成功値を返す。
- typed failureでは`ROLLBACK`とreleaseを完了してから元のfailureを返す。
- defectまたはcancellationでもmasked cleanupとして`ROLLBACK`とreleaseを行い、cancellationを
  `Error`へ変換しない。
- commit / rollback / releaseは競合しても各transactionにつき一回だけ開始する。
- pool cleanupは未完了transactionをrollbackしてからpoolを閉じる。

applicationへraw `begin` / `commit` / `rollback` handleを公開しない。これらは
`seseragi/postgres::Postgres` Contractのoperationであり、package wrapperが上のresource規則へ
組み立てる。

## 16.5 cursor

`openCursor query pool`はPool配下のCursor resourceを取得する。`fetch limit decoder cursor`は
正の有限limit以下のrowだけをpullし、同じPostgreSQL decoderで`Array<A>`へ変換する。
`closeCursor`、scope cleanup、cancellation、pool closeはいずれもcursor close、checked-out
connection release、pool closeの順序を守る。

本章ではCursorを`Stream`であるとは扱わない。`std/stream`完成後だけ、同じfetch demandと
Cursor lifetimeを保つadapterを追加できる。現在のCursorをPromise列やunbounded bufferへ
読み替えてはならない。

## 16.6 failureとProvider境界

`Error`は回復可能なdriver failureを保持する`DriverFailure DriverError`と、application decodeの
`RowDecodeFailure RowDecodeError`からなる。DriverErrorはoperation、driver code、messageを保持する。
host throw / rejection、malformed envelope、invalid logical value、別providerのhandleはdefectであり、
driver messageから新しいtyped constructorを推測しない。

canonical Contractは`seseragi/postgres::Postgres@1.1`で、pool/query、transaction、cursorのoperationを
所有する。TypeScript Provider `seseragi/runtime-postgres#pg`は`pg`と`pg-cursor`をwrapし、独自wire
protocolを実装しない。driver packageはrelease artifactへbundleするが、application importと
lockfileのservice identityはbundle layoutに依存しない。
