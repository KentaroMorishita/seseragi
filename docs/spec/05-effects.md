# 5. 失敗・非同期・副作用

## 5.1 純粋な式

通常の Seseragi 関数は、引数だけから結果を計算します。外部状態の読み書き、時刻、
乱数、console、network、filesystem は通常の値を返す関数として公開できません。

回復可能な副作用と非同期処理は `Task<E, A>` で表します。純粋性は annotation ではなく、
利用できる値の型によって保たれます。

## 5.2 Maybe

```seseragi
type Maybe<A> =
  | Nothing
  | Just A
```

`Maybe<A>` は値が存在しないこと自体が正常な場合に使います。失敗理由が必要な場合は
`Either<E, A>` を使います。

## 5.3 Either

```seseragi
type Either<E, A> =
  | Left E
  | Right A
```

`Either` は、評価済みの同期的な結果です。`Left` と `Right` のどちらも通常の値で、
生成時に追加の計算は起きません。

## 5.4 Task

`Task<E, A>` は、実行すると非同期に `E` で失敗するか `A` で成功する計算です。

- Task は cold。作っただけでは実行しない。
- 同じ Task を二度実行すると、計算も二度実行する。
- `map` は成功値だけを変換する。
- `flatMap` / `>>=` は前の Task が成功した後に次の Task を作る。
- `catch` は failure channel `E` だけを扱う。
- Task の callback は一度だけ完了する。

```seseragi
fn loadProfile id: UserId -> Task<HttpError, Profile> =
  fetchUser id >>= fetchProfile
```

並列実行は `Task.parallel` のような標準関数で明示します。式を並べただけでは並列に
なりません。

## 5.5 Task の実行境界

Task を実行する操作は通常の Seseragi コードには公開されません。ホストは `main`、
test runner、callback adapter などの明示された境界でだけ Task を実行します。

この制約により、純粋関数の途中で隠れて同じ Task が複数回実行されることを防ぎます。

## 5.6 defect

回復可能な失敗と、プログラム継続を保証できない defect を区別します。

defect の例:

- 整数 overflow と zero division
- コンパイラまたは runtime の内部不変条件違反
- memory exhaustion
- safe wrapper の契約に違反した外部値

defect は `Either` や Task の failure channel へ暗黙変換されません。通常のコードから
catch できません。入力や外部失敗として想定できるものを defect にしてはなりません。

## 5.7 cancellation と resource

Task は cooperative cancellation を持ちます。ホストが cancellation を要求すると、
未完了の Task は子 Task へ伝播し、登録された finalizer を LIFO で一度だけ実行します。

resource は `Task.bracket acquire use release` で扱います。`release` は success、failure、
cancellation のすべてで実行します。cancellation は `E` の値ではありません。

## 5.8 Signal

`Signal<A>` は core syntax ではなく標準ライブラリの状態 abstraction です。読み取りと
更新は Task を通して行い、`:=` のような代入構文は持ちません。

```seseragi
fn read<A> signal: Signal<A> -> Task<Never, A>
fn set<A> value: A -> signal: Signal<A> -> Task<Never, Unit>
```

## 5.9 例外と try

Seseragi に throw / catch / try 構文はありません。外部例外は interop adapter が捕捉し、
`Either<ForeignError, A>` または `Task<ForeignError, A>` に変換します。
