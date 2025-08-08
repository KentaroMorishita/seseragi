# Seseragi 学習ガイド

---

## 第1部：Seseragiへの第一歩

この章では、Seseragiがどのような言語で、どのような課題を解決するのかを学びます。そして、実際にプログラムを動かし、その基本的な考え方に触れていきます。

### 1.1. はじめに

Seseragiは、TypeScriptにコンパイルされる新しい静的型付け言語です。

TypeScriptの強力な型システムと広大なエコシステムを基盤としながら、より安全で、より表現力豊かなコード記述を可能にすることを目指しています。

> **Seseragiという名前について**
>
> Seseragiは、日本語の「せせらぎ」から名付けられました。小川のさらさらとした流れのように、思考を妨げずに自然に書ける、なめらかなシンタックスを持つ言語を目指すという想いが込められています。また、開発者自身が日本人であることから、日本語由来の響きを大切にしたいという考えも反映されています。

#### TypeScript開発者のためのSeseragi

あなたがTypeScriptに慣れ親しんでいるなら、Seseragiの学習はスムーズに進むでしょう。Seseragiは、TypeScript開発者が日常的に直面するいくつかの課題を、言語機能によって解決することを目指しています。

-   **`null` と `undefined` からの解放**
    `Maybe`と`Either`という特別な型を導入し、`null`や`undefined`に起因する実行時エラーをコンパイル時に完全に防ぎます。これにより、コードの至る所に現れる煩雑なnullチェックから解放されます。

-   **複雑な条件分岐の簡素化**
    強力な`match`式（パターンマッチング）により、ネストした`if`文や複雑な`switch`文を、可読性が高く、漏れのない安全なコードに置き換えられます。

-   **予測可能な状態管理**
    データはデフォルトで **不変（イミュータブル）** です。TypeScriptで`const`を徹底するのと似ていますが、これが言語の標準です。意図しない副作用を防ぎ、アプリケーションの状態管理を劇的にシンプルにします。

-   **より正確な型表現**
    代数的データ型（ADT）を導入し、ビジネスロジックをより正確かつ柔軟に型で表現できます。

**重要**: Seseragiは純粋な関数型言語ではありません。関数型言語から生まれた実用的なアイデアを取り入れつつも、あくまでTypeScriptの延長線上にある、親しみやすい言語であることを目指しています。

---

### 1.2. 環境構築と最初のプログラム

まずはSeseragiを動かしてみましょう。

#### インストール

プロジェクトの依存関係をインストールします。（このプロジェクトでは`bun`を使用しています）

```bash
bun install
```

#### Hello, World!

1.  まず、`hello.ssrg` という名前のファイルを作成します。

2.  ファイルに以下のコードを記述します。

    ```rust
    // hello.ssrg
    print "Hello, Seseragi!"
    ```

3.  `seseragi run` コマンドで実行します。

    ```bash
    seseragi run hello.ssrg
    ```

4.  コンソールに `Hello, Seseragi!` と表示されれば成功です。

#### CLIツールの概要

Seseragiには、主に3つのコマンドが用意されています。

-   `seseragi run <file>`
    ファイルを直接実行します。内部的にはTypeScriptへのコンパイルと実行が同時に行われます。

-   `seseragi compile <file>` (または `seseragi <file>`)
    SeseragiのコードをTypeScriptファイル (`.ts`) に変換します。

-   `seseragi fmt <file>`
    コードを公式のフォーマットルールに従って整形します。

---

### 1.3. Seseragiの基本的な考え方

Seseragiのコードを書き始める前に、他の多くの言語とは異なる、2つの重要な概念を理解することが助けになります。

#### 式 (Expression) が中心の世界

多くの言語では、値を返さない「文（Statement）」と、値を返す「式（Expression）」が区別されます。

```typescript
// TypeScriptの `if` は「文」であり、値を返さない
let message: string;
if (score > 80) {
  message = "Pass";
} else {
  message = "Fail";
}
```

一方、Seseragiでは、**ほとんどの構文が値を返す「式」として設計されています**。

これにより、コードがより簡潔になり、変数の再代入を避けることができます。

```rust
// Seseragiの `if` は「式」であり、それ自体が値を返す
let message = if score > 80 then "Pass" else "Fail"
```

この「すべてが値を返す」という考え方は、Seseragiの様々な機能の根底に流れる重要な原則です。

#### 不変性 (Immutability) の重要性

Seseragiでは、`let`で束縛された変数はすべて **不変（イミュータブル）** です。

一度値を設定すると、その値を変更することはできません。

これはTypeScriptで常に`const`を使い、`let`による再代入を避けるスタイルに似ています。

---

## 付録: リアクティブプログラミング（Signal）

Seseragi には簡潔な FRP 風プリミティブ `Signal<T>` が用意されています。

- 生成: `let s: Signal<Int> = Signal(0)`
- 更新（プッシュ）: `s := 1`
- 現在値の取得: `let now = *s`
- 購読/解除: `let key = subscribe s (\\v -> print \`v=${v}\`); unsubscribe key`
- 後始末: `detach s`
- 合成: `<$>`（写像）、`<*>`（適用）、`>>=`（束縛）を `Signal` にも利用可能

例:

```rust
let s: Signal<Int> = Signal(0)
let doubled: Signal<Int> = (\\x -> x * 2) <$> s
let add = \\x -> \\y -> x + y
let sumSig: Signal<Int> = add <$> s <*> doubled
let key = subscribe sumSig (\\v -> print \`sumSig=${v}\`)

s := 1
s := 2

unsubscribe key
detach s
```

より詳しい例は `examples/intermediate/05-frp-signals.ssrg` を参照してください。

---

## 付録: 非同期計算（Task）

`Task<T>` は `() -> Promise<T>` を包む非同期計算です。

- 生成: `let t: Task<Int> = Task $ resolve 100`
- 実行: `run t  // Promise<Int> を返す`
- 合成: `<$>`（写像）、`<*>`（適用）、`>>=`（束縛）を `Task` に利用可能
- 失敗: `Task $ reject "boom"`、`tryRun t` は `Promise<Either<String, T>>` を返す

await できない場面では、写像内で副作用を行うと出力を確認できます。

```rust
let t: Task<Int> = Task $ resolve 100
let logged: Task<Int> = (\\x -> (\\() -> x) $ print \`x=${x}\`) <$> t
run logged  // 実行時に出力
```

詳しくは `examples/intermediate/06-tasks.ssrg` を参照してください。

Seseragiでは、この安全なプラクティスが言語レベルで標準となっています。

なぜ不変性が重要なのでしょうか？

-   **予測可能性**: 値が途中で変わらないため、プログラムの動作が追いやすくなります。
-   **副作用の抑制**: 「知らないうちに値が変更されていた」という種類のバグを根本的に防ぎます。
-   **安全な並行処理**: 複数の場所から同じデータを参照しても、競合状態が発生しません（高度なトピック）。

データの一部を変更したい場合は、元のデータを変更するのではなく、変更箇所だけが新しくなったデータのコピーを生成します。

Seseragiには、これを効率的に行うための構文も用意されています。

---

## 第2部：基本的なプログラミング要素

この章では、プログラミングの基本的な構成要素である「変数」「型」「演算子」「関数」「制御フロー」について、Seseragiならではのアプローチを学びます。

### 2.1. 変数・基本型・演算子

#### 変数束縛と基本型

`let` を使って、値を名前（変数）に束縛します。

Seseragiは型推論を持つため、型を省略することができます。

```rust
let companyName = "TechCorp" // String型
let yearEstablished = 2024     // Int型
let rating = 4.5               // Float型
let isPublic = True            // Bool型
```

型を明記したい場合は、変数名の後に `: 型名` を付けます。

```rust
let companyName: String = "TechCorp"
let yearEstablished: Int = 2024
let rating: Float = 4.5
let isPublic: Bool = True
```

#### 文字列補間

バッククォート(`` ` ``)で囲んだ文字列の中では、`${...}` を使って変数を埋め込むことができます。

```rust
let name = "Alice"
let message = `こんにちは、${name}さん！`
```

#### 基本的な演算子

標準的な算術演算子 (`+`, `-`, `*`, `/`, `%`)、べき乗演算子 (`**`)、比較演算子 (`==`, `!=`, `>`, `<`)、論理演算子 (`&&`, `||`, `!`) が利用できます。

---

### 2.2. 関数：Seseragiの心臓部

Seseragiでは、関数は単なる手続きの集まりではなく、プログラムを組み立てるための最も重要な「部品」です。

#### `fn`による定義

`fn` キーワードを使って関数を定義します。

本体が一つの式で終わる場合は `=` を、複数の処理を記述する場合は `{}` ブロックを使います。

```rust
// `=` を使った単一式の関数
fn square x: Int -> Int = x * x

// `{}` を使ったブロック構文の関数
// ブロックの最後の式が戻り値になる
fn getHypotenuse a: Float -> b: Float -> Float {
  let a2 = a ** 2.0
  let b2 = b ** 2.0
  (a2 + b2) ** 0.5
}
```

#### 自動カリー化と部分適用

Seseragiのすべての関数は、自動的に **カリー化** されます。

これにより、関数に一部の引数だけを渡して、特定の仕事に特化した新しい関数を簡単に作る **部分適用** が可能です。

```rust
fn add x: Int -> y: Int -> Int = x + y

// 部分適用: 「10を足す」関数を作る
let addTen = add 10

show $ addTen 3 // 13
```

#### 高階関数とラムダ式

関数は値として扱えます（**高階関数**）。`\` でラムダ式（無名関数）を定義できます。

```rust
// 配列の各要素を2倍する (演算子については後の章で詳述)
let numbers = [1, 2, 3]
let doubled = (\x -> x * 2) <$> numbers
// doubled は [2, 4, 6]
```

---

### 2.3. 制御フロー：分岐を極める

#### `if-then-else` 式と三項演算子

Seseragiの `if` は、値を返す **式** であり、`else` を省略できません。

```rust
let score = 75
let grade = if score >= 80 then "A" else if score >= 60 then "B" else "C"
```

より簡潔な糖衣構文として、三項演算子 (`? :`) も利用できます。

```rust
let grade = score >= 80 ? "A" : score >= 60 ? "B" : "C"
```

#### パターンマッチング (`match`) 入門

`match` 式は、`if-else` の連鎖や `switch` 文を、より安全で表現力豊かにしたものです。

値がどの「形（パターン）」に一致するかを調べ、対応する処理を実行します。

コンパイラがすべての可能性を網羅しているかチェックしてくれるため、バグの防止に繋がります。

```rust
fn getHttpStatusMessage code: Int -> String = match code {
  200 -> "OK"
  404 -> "Not Found"
  500 -> "Internal Server Error"
  _ -> "Unknown Status" // `_` はどのパターンにも一致しなかった場合のデフォルト
}

show $ getHttpStatusMessage 404 // "Not Found"
```

この章では基本的なリテラル（具体的な値）のマッチングを紹介しました。

`match` の真価は、第3部以降で学ぶ複雑なデータ構造と組み合わせることで発揮されます。

---

## 第3部：データ構造を使いこなす

この章では、データをまとめるための様々な「入れ物」と、それらを効率的に扱う方法について学びます。

### 3.1. 基本的な複合データ

#### タプル (Tuple): 一時的な値のペアリング

タプルは、複数の値を一時的にまとめるための最もシンプルな方法です。

`let`束縛で複数の値を定義したり、`match`式で複数の値を一度に評価したい場合に特に役立ちます。

```rust
let myTuple = (10, "hello", True)

// 分割代入でタプルの要素を取り出す
let (num, str, bool) = myTuple
// numは10, strは"hello", boolはTrue

// パターンマッチングで使う
match myTuple {
  (10, _, True) -> print "Match!"
  _ -> print "No match"
}
```

タプルは、その場で使うための一時的な構造であり、長期的に使うデータには次に説明するレコードや構造体を使うのが一般的です。

#### レコード (Record): 名前付きフィールドを持つデータ

レコードは、TypeScriptのオブジェクトリテラルやインターフェースのように、名前（キー）と値のペアを集めたものです。

フィールド名でデータにアクセスするため、タプルよりも意図が明確になります。

```rust
let user = { name: "Alice", age: 30, active: True }

// フィールドへのアクセス
show user.name // "Alice"

// レコードは「構造的型付け」に従う
// つまり、フィールドの構成が同じであれば、同じ型として扱われる
// (型エイリアスについては第4部で詳述)
fn greet u: { name: String } -> String {
  `こんにちは、${u.name}さん！`
}

show $ greet user // "こんにちは、Aliceさん！"
```

---

### 3.2. コレクションの選択と活用: `Array` vs `List`

Seseragiには、複数の要素をまとめるためのコレクションとして `Array` と `List` の2種類があります。

これらは似ていますが、得意なことと苦手なことがあります。

#### `Array`: TypeScriptライクな高速アクセス

`Array` は、TypeScriptの配列とほぼ同じ感覚で使えます。

内部的にはJavaScriptの配列としてコンパイルされるため、既存のJavaScriptライブラリとの連携も容易です。

-   **いつ使うか？**: インデックスを使ったランダムアクセスが頻繁に必要な場合や、要素数を頻繁に確認する場合。
-   **定義**: `[1, 2, 3, 4, 5]`
-   **安全なアクセス**: `myArray[index]` は、範囲外エラーを防ぐために `Maybe` 型を返します。

```rust
let numbers = [10, 20, 30]

show numbers.length // 3

match numbers[0] {
  Just n -> show `最初の要素: ${n}` // 最初の要素: 10
  Nothing -> show "要素がありません"
}
```

#### `List`: 不変で再帰的なデータ処理

`List` は、関数型プログラミングで伝統的に使われる「連結リスト」です。

その本質は、「**`Empty`（空リスト）**」か、「**`Cons h t`（先頭要素 `h` と残りのリスト `t`）**」のどちらかである、という再帰的な構造にあります。

-   **いつ使うか？**: リストの先頭から順番に処理する再帰的なアルゴリズムや、パターンマッチングでリストを分解する場合。

-   **定義（段階的解説）**:

    1.  **基本構造**: `List`は`Cons`と`Empty`で構成されます。
        ```rust
        let list1 = Cons 1 (Cons 2 (Cons 3 Empty))
        ```

    2.  **`:` 演算子**: `Cons`の糖衣構文（シンタックスシュガー）です。`:`は右結合なので、括弧なしで繋げられます。
    また、`` `[] ``は`Empty`の糖衣構文です。
        ```rust
        let list2 = 1 : 2 : 3 : `[] // list1と等価
        ```

    3.  **`` `[...] `` 構文**: 最も直感的で推奨される糖衣構文です。
        ```rust
        let list3 = `[1, 2, 3] // list1, list2と等価
        ```

-   **基本操作**: `^` (head) と `>>` (tail) 演算子を使います。

```rust
let numbers = `[10, 20, 30]

show $ ^numbers // Just 10
show $ >>numbers // `[20, 30]

// パターンマッチングとの組み合わせが強力
fn sum list: List<Int> -> Int = match list {
  `[] -> 0 // 空のリストなら0
  `[h, ...t] -> h + sum t // 先頭(h)と残り(t)に分解し、再帰的に合計する
}

show $ sum numbers // 60
```

---

### 3.3. 宣言的なコレクション操作: 内包表記

`for`ループの代わりに、Seseragiでは **内包表記 (Comprehension)** を使って、既存のコレクションから新しいコレクションを宣言的に生成します。

これは、TypeScriptの `map` と `filter` を組み合わせたような処理を、より直感的な構文で記述できる機能です。

```rust
// 1から10までの数値のうち、偶数だけを取り出し、それぞれを2乗した配列を作る
let evenSquares = [x * x | x <- 1..=10, x % 2 == 0]
// evenSquares は [4, 16, 36, 64, 100]
```

-   `x <- 1..=10`: `1..=10` のコレクションから、各要素を `x` として取り出す。（ジェネレータ）
-   `x % 2 == 0`: `x` が偶数のものだけを対象とする。（ガード）
-   `x * x`: 対象となった `x` を2乗する。（出力式）

内包表記は `Array` (`[]`) と `List` (`` `[] ``) の両方で利用できます。

---

## 第4部：強力な型システム

この章では、Seseragiの表現力を支える型定義機能について深く掘り下げます。

TypeScriptの型システムに慣れている方なら、その違いと利点をより深く理解できるでしょう。

### 4.1. 独自の型を定義する

#### 型エイリアス (`type`): 型に別名を付ける

`type` キーワードは、既存の型に分かりやすい別名を付けるために使います。

これはTypeScriptの `type` エイリアスとまったく同じです。

新しい型を生成するわけではなく、あくまで別名です。

```rust
// 基本的な型に別名を付ける
type UserId = Int
type Email = String

// 複雑な型に別名を付ける
type UserProfile = {
    name: String,
    age: Int
}
type OnSuccess = (data: String) -> Unit
```

型エイリアスを使うことで、コードの意図が明確になり、ドキュメントとしての役割も果たします。

#### 構造体 (`struct`): データと振る舞いをカプセル化する

`struct` は、関連するデータをひとまとめにした「構造」を定義します。これはTypeScriptの `class` に似ていますが、よりデータ中心の設計になっています。

`struct` の重要な特徴は、それが **名前的型付け (Nominal Typing)** である点です。

つまり、たとえフィールドの構成がまったく同じでも、`struct` の名前が異なれば、それらは完全に別の型として扱われます。

これにより、意図しない型の混同を防ぎます。

```rust
struct Point {
    x: Float,
    y: Float
}
struct Vector {
    x: Float,
    y: Float
}

let p = Point { x: 1.0, y: 2.0 }
let v = Vector { x: 1.0, y: 2.0 }

// pとvは、たとえ形が同じでも、型が違うので互換性はない
```

#### `impl`: メソッドと演算子オーバーロード

`impl` ブロックを使うことで、定義した `struct` に関連する振る舞い（メソッド）や、演算子のカスタム実装（オーバーロード）を追加できます。

```rust
impl Point {
  // `self` はインスタンス自身を指す。TypeScriptの `this` に相当。
  fn distanceToOrigin self -> Float {
    (self.x ** 2.0 + self.y ** 2.0) ** 0.5
  }

  // `+` 演算子の振る舞いをPoint型専用に定義する
  operator + self -> other -> Point {
    Point { x: self.x + other.x, y: self.y + other.y }
  }
}

let p1 = Point { x: 3.0, y: 4.0 }
let p2 = Point { x: 1.0, y: 2.0 }

// メソッド呼び出し
show $ p1 distanceToOrigin() // 5.0

// オーバーロードされた演算子の使用
show $ p1 + p2 // Point { x: 4.0, y: 6.0 }
```

---

### 4.2. 代数的データ型 (ADT): 状態を正確に表現する

代数的データ型（ADT）は、Seseragiの型システムの中でも特に強力な機能の一つです。「**この型は、これらの決まった形のいずれか一つを取る**」ということを、型レベルで厳密に表現できます。

これはTypeScriptの合併型（Union Types）に似ていますが、各バリエーションが「タグ」によって明確に区別されるため、より安全で、パターンマッチングとの相性が抜群に良いという利点があります。

#### ADTの定義

`type` キーワードと `|` を使って定義します。

各バリエーション（型コンストラクタ）は、大文字で始める必要があります。

```rust
// 最もシンプルなADT（列挙型）
type Status =
  | Idle
  | Loading
  | Success
  | Failure

// 各バリエーションがデータを持つADT
type WebEvent =
  | PageLoad
  | Click { x: Int, y: Int }
  | KeyPress String
```

#### ADTの利点: ありえない状態をなくす

例えば、APIリクエストの状態を考えてみましょう。

TypeScriptでは、以下のように `isLoading`, `data`, `error` といった複数の状態で管理することがよくあります。

```typescript
interface ApiState {
  isLoading: boolean;
  data?: string;      // 具体的な型に
  error?: Error;
}
```

この設計には、「`isLoading`が`true`なのに`data`も存在する」や「`data`と`error`が同時に存在する」といった、本来ありえないはずの状態が生まれる余地があります。

ADTを使うと、これらの状態を相互に排他的なバリエーションとして定義できるため、ありえない状態を型システムレベルで完全に排除できます。

```rust
type ApiState =
  | Loading
  | Success String
  | Failure String

let state: ApiState = Loading

// パターンマッチングで、それぞれの状態を安全かつ網羅的に処理できる
fn render state: ApiState -> String = match state {
  Loading -> "読み込み中..."
  Success data -> `データ: ${data}`
  Failure msg -> `エラー: ${msg}`
}
```

---

## 第5部：安全なエラーハンドリング

Seseragiは、多くの言語で一般的な`null`、`undefined`、そして`try-catch`による例外処理とは一線を画す、型システムに基づいた安全なエラーハンドリング機構を提供します。

### 5.1. `null`と`try-catch`がなぜ問題なのか？

TypeScript（やJavaScript）における`null`や`undefined`は、「値が存在しない」ことを示す便利な仕組みですが、同時に「ぬるぽ」として知られる`TypeError: Cannot read properties of null`のような実行時エラーの最大の原因でもあります。

コードの至る所で`if (value != null)`のようなチェックが必要になり、コードが煩雑になるだけでなく、チェック漏れによるバグが後を絶ちません。

また、`try-catch`による例外処理は、エラーが発生しうるコードと、それを処理するコードが離れてしまいがちです。

どの関数が例外を投げる可能性があるのかが関数のシグネチャ（型）から分からず、見えないエラーに怯えながらプログラミングすることになります。

Seseragiは、これらの問題を「**失敗する可能性を型で表現する**」ことで解決します。

---

### 5.2. `Maybe<T>`型: 「値がない」かもしれない場合

`Maybe<T>`は、「`T`型の値が**存在する (`Just T`)**」か、「**存在しない (`Nothing`)**」かのどちらかであることを示す型です。

これは、関数の戻り値がオプションである場合や、配列・辞書検索のように結果が見つからない可能性がある場面で使います。

`Maybe`型を返す関数を使うと、`null`チェックを忘れるということがなくなります。

なぜなら、`Maybe`型の値から中身を直接取り出すことはできず、**`match`式を使って`Just`と`Nothing`の両方のケースを必ず処理しなければならない**からです。

コンパイラがチェックを強制してくれます。

```rust
let names = ["Alice", "Bob", "Charlie"]

// `[]` アクセスは Maybe<String> を返す
let first = names[0]
let fifth = names[4]

fn showName maybeName: Maybe<String> -> String = match maybeName {
  Just name -> `名前: ${name}`
  Nothing -> "名前がありません"
}

show $ showName first // 名前: Alice
show $ showName fifth // 名前がありません
```

---

### 5.3. `Either<L, R>`型: 「失敗の理由」を伝えたい場合

`Either<L, R>`は、`Maybe`をさらに強力にしたものです。

処理が「**成功 (`Right R`)**」したか、「**失敗 (`Left L`)**」したかを示します。

`Maybe`との最大の違いは、失敗した場合に**なぜ失敗したのかというエラー情報 `L` を保持できる**点です。

```rust
// 文字列を数値にパースする関数
fn parseInt text: String -> Either<String, Int> = match text {
  "123" -> Right 123
  "abc" -> Left "無効な数値形式です"
  _ -> Right 0 // 簡単のため
}

let result1 = parseInt "123"
let result2 = parseInt "abc"

match result2 {
  Right num -> show `パース成功: ${num}`
  Left err -> show `パース失敗: ${err}` // パース失敗: 無効な数値形式です
}
```

`try-catch`と比べて、`Either`を使う利点は、**エラーが関数の戻り値の型の一部として明記される**ことです。

これにより、その関数が失敗する可能性があることが一目瞭然となり、呼び出し側は必ずエラーケースの処理を記述する必要に迫られます。

---

### 5.4. `>>=`による安全な処理の連鎖

`Maybe`や`Either`の真価は、失敗する可能性のある処理を安全に **連鎖（パイプライン化）** できる点にあります。

もし、複数の処理を連続して行い、どれか一つでも失敗したら、そこで処理を中断したい場合、TypeScriptでは`if`のネストや`try-catch`ブロックが必要になります。

Seseragiでは、`>>=`（バインド）演算子を使うことで、この一連の流れを驚くほどクリーンに記述できます。

`m >>= f`は、`m`が成功（`Just`または`Right`）した場合にのみ、その中身を関数`f`に渡して次の処理を実行します。

`m`が失敗（`Nothing`または`Left`）した場合、`f`は実行されず、失敗がそのまま最終結果となります。

#### 具体例: 設定値の安全な取得

ユーザーIDから設定を探し、その設定からホスト名を取得する、という一連の処理を考えます。

```typescript
// TypeScriptでのネストしたnullチェック
interface Config { host: string; port: number; }

function findConfig(userId: string): Config | undefined { ... }
function getHost(config: Config): string | undefined { ... }

const config = findConfig("user1");
if (config) {
  const host = getHost(config);
  if (host) {
    console.log(host); // "example.com"
  } else {
    console.log("ホスト名が見つかりません");
  }
} else {
  console.log("設定が見つかりません");
}
```

同じ処理をSeseragiで書くと、以下のようになります。

```rust
type Config = { host: String, port: Int }

// ユーザーIDから設定を取得する。ユーザーが存在しないかもしれない。
fn findConfig userId: String -> Maybe<Config> = match userId {
  "user1" -> Just $ { host: "example.com", port: 8080 } as Config
  _ -> Nothing
}

// Configからホスト名を取得する
fn getHost config: Config -> Maybe<String> = Just config.host

// 処理の連鎖
let hostname = Just "user1"
  >>= findConfig
  >>= getHost

show hostname // Just "example.com"

// ユーザーが存在しない場合
let hostname2 = Just "user2"
  >>= findConfig
  >>= getHost

show hostname2 // Nothing
```

`if`のネストが、`>>=`演算子による一直線の処理の流れに置き換わっていることが分かります。

`>>=`が途中の`Nothing`を自動的に処理してくれるため、開発者は成功した場合の処理（`findConfig`と`getHost`）を繋ぐことだけに集中できます。

---

## 第6部：発展的なトピック

これまでの章で、Seseragiの基本的な要素はすべて学びました。

この最終章では、それらの知識を組み合わせ、より実践的なコーディングに役立つ高度なテクニックと、今後の展望について解説します



### 6.1. パターンマッチング詳解

`match`式は、これまで見てきた基本的な使い方以外にも、複雑なデータ構造を簡潔に、かつ安全に分解するための様々なパターンをサポートしています。

#### `when`によるガード

パターンにマッチした上で、さらに追加の条件（ガード）を満たす場合にのみ、その処理を実行させることができます。

```rust
fn checkNumber n: Int -> String = match n {
  0 -> "ゼロです"
  x when x % 2 == 0 -> `偶数: ${x}`
  x when x % 2 != 0 -> `奇数: ${x}`
  _ -> "?"
}

show $ checkNumber 0 // "ゼロです"
show $ checkNumber 10 // "偶数: 10"
show $ checkNumber 7  // "奇数: 7"
```

#### データ構造の分解

`match`式の真価は、`List`や`struct`のような複雑なデータ構造と組み合わせた際に発揮されます。

構造の内部まで深く入り込み、特定のパターンに一致する要素を直接取り出すことができます。

-   **Listの分解**: `h : t` パターンで、リストを先頭要素 (`h`) と残りのリスト (`t`) に分解できます。

    ```rust
    fn describeList list: List<Int> -> String = match list {
      `[] -> "空のリストです"
      `[h] -> `要素が一つだけのリスト: ${h}`
      `[h, ...t] -> `先頭は${h}で、残りがあります`
    }
    ```

-   **Arrayの分解**: `[]` の中で、固定長の配列パターンにマッチさせることができます。

    ```rust
    fn describeArray arr: Array<Int> -> String = match arr {
      [] -> "空の配列です"
      [a] -> `要素が一つ: ${a}`
      [a, b] -> `要素が二つ: ${a}と${b}`
      _ -> "要素が3つ以上あります"
    }
    ```

---

### 6.2. TypeScriptとの連携（今後の展望）

SeseragiはTypeScriptにコンパイルされるため、原理的には広大なJavaScript/TypeScriptのエコシステムと連携することが可能です。

#### 生成されるコード

`seseragi compile`コマンドで生成される`.ts`ファイルは、人間が読める、素直なコードを目指して設計されています。

例えば、Seseragiの`struct`はTypeScriptの`class`に、ADTはタグ付き合併型に変換されるため、TypeScript側からでも比較的容易に利用することができます。

#### FFI (Foreign Function Interface)

将来的には、既存のTypeScriptの関数やライブラリを、Seseragiのコード内から型安全に呼び出すための仕組み（FFI）の導入が検討されています。

これにより、例えば`npm`でインストールしたパッケージの機能を、Seseragiの強力な型システムと組み合わせながら利用できるようになり、言語の可能性が大きく広がることが期待されます。

---

これで、Seseragi学習ガイドは終わりです。お疲れ様でした。

このガイドが、あなたのSeseragiでのプログラミング体験を、より楽しく、より生産的なものにする一助となれば幸いです。
