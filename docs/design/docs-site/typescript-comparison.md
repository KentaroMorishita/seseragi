# Entrance comparison decision and evidence (#698)

Status: selected source and executable evidence for the #700 entrance rewrite.
This does not change Home or Introduction, accept their rendering, or close the
first-time-reader review. The audience and scope follow
[reader-contract.md](reader-contract.md).

## Selection

Use **calculating order totals under one shipping policy**. A regular call
receives a free-shipping threshold, shipping fee and order subtotal. Giving it
only the first two values creates a function for that policy; each order then
supplies just its subtotal. The purpose of reuse is recognizable before the
reader learns its name, partial application.

The TypeScript counterpart is an ordinary function and one arrow function. It
does not deliberately curry the original TS function, add a class, invent a
configuration object, or add error handling absent from Seseragi. Both versions
already separate the shipping policy from individual orders. The distinct
Seseragi affordance is that **ordinary application also creates the configured
function**. Reuse, centralizing a rule and typechecking string arguments are
not exclusive to Seseragi. Do not turn this into a line-count, performance,
allocation, or whole-language superiority claim.

Three small candidates were tried in both languages on 2026-10-01:

| Task | Observed result in both | Decision |
| --- | --- | --- |
| Shipping rule: free from ¥5,000, otherwise ¥500 | `3700, 5000` for ¥3,200 and ¥5,000 | Select. Fixed conditions and changing order input have a practical reason to be separate. An `if` and ordinary calls suffice. |
| Prefix an order notification subject | `[Support] Order shipped` | Reject as entrance. Very small, but TS string construction is already natural and the extra reusable function has little motivation. |
| Select products within a ¥500 budget | One matching product from Notebook ¥800 / Pen ¥300 | Defer until records and functions passed to array operations have been explained. A natural TS `products.filter(product => product.price <= 500)` is already concise; do not pad it with a predicate factory. |

Only the selected pair is retained as a canonical comparison and regression
gate. The other two were disposable selection probes, not maintained examples.
ADT, `match`, missing data and error-value comparisons are later work after
those concepts have their own explanations; none is a prerequisite here.

## Canonical artifact and publication contract

- Seseragi: [seseragi.ssrg](../../../apps/site/examples/comparisons/shipping-total/seseragi.ssrg)
- TypeScript: [typescript.ts](../../../apps/site/examples/comparisons/shipping-total/typescript.ts)
- Exact stdout: [expected.stdout](../../../apps/site/examples/comparisons/shipping-total/expected.stdout)
- Source registration and boundary cases:
  [comparisons.ts](../../../apps/site/scripts/comparisons.ts)
- Native/TS checks, invoked by the existing site example gate:
  [check-comparisons.ts](../../../apps/site/scripts/check-comparisons.ts)
- Source and Playground regression tests:
  [comparisons.test.ts](../../../apps/site/tests/comparisons.test.ts)

The existing `ExampleSource` input and code-panel component remain the rendering
path. The build registers `comparison-shipping-total-seseragi` and
`comparison-shipping-total-typescript`. Use these IDs in #700, with explicit
Seseragi / TypeScript captions; do not paste strings into locale modules.
`canonical-example.ts` is the existing source loader extracted for shared use,
not a second renderer. It reads, hashes and highlights the same source bytes
that the verification runs. TypeScript is rendered as plain code and has no
Seseragi Playground link. The Seseragi source, including its output entry, is
passed unchanged to the existing Playground source-link helper.

Both examples use integers in yen. The agreed comparison input domain is
`0..1,000,000` for each threshold, fee and subtotal. Thus the largest possible
sum stays below 2,000,000 and is exact in both versions. No version reads user
input, performs payments, changes an order, calculates tax or validates this
domain. A zero subtotal means applying the mathematical rule to zero; it is
not a claim that an empty shopping cart should incur a shipping charge.

## Japanese-first explanation for #700

### 課題と結果

5,000円以上の注文は送料無料、それ未満は送料500円とします。同じ送料の条件で、
小計3,200円と5,000円の注文の支払額を計算します。どちらのコードも
`3700, 5000` と表示します。三つの引数はすべて0〜1,000,000円の整数とし、
入力の検証はこの例に含めません。

### 最初の関数を読む

`fn` は関数を定義します。`totalWithShipping` は送料無料になる金額
`freeFrom`、送料 `fee`、注文の小計 `subtotal` を順番に受け取ります。
各引数の `: Int` は整数を受け取る指定で、最後の `-> Int` は整数を返す指定です。
途中の `->` は次の引数へ続きます。`=` の右側が計算です。

`if ... then ... else ...` は条件に合う方の値を返します。小計が送料無料の金額
以上なら小計をそのまま返し、未満なら送料を加えます。通常の呼び出しは
`totalWithShipping 5000 500 3200` です。関数名の後ろに空白で引数を並べると、
この場合は `3700` が返ります。

### 同じ条件を使い回す

`let standardTotal = totalWithShipping 5000 500` は、最初の二つの引数だけを
渡し、残りの小計を受け取る関数に `standardTotal` という名前を付けます。
まだ小計を渡していないので、この行では注文の金額を計算しません。
`standardTotal 3200` は `3700`、`standardTotal 5000` は `5000` になります。
引数の一部だけを渡して関数を作る、この使い方を「部分適用」と呼びます。
`let` で付けた名前を後から別の値へ代入し直すことはできません。

TypeScriptでも矢印関数で同じ処理を書けます。どちらも送料の条件を一か所に
まとめ、注文ごとには小計だけを渡せます。Seseragiでは、引数を全部渡す呼び出し
と同じ書き方で、条件を固定した関数も作れます。ここで試してほしいのは、
短さそのものより、普通の関数をそのまま自分の用途に合わせて使える感触です。

### 表示の行を読む

最後の `pub effect fn main` は、実行するときに使う公開された入口です。
`effect` は、この入口が画面への出力のような外部とのやり取りを行うことを示します。
`println` は文字列と改行を出力します。バッククォートで囲んだ文字列の
`${smallOrder}` と `${largeOrder}` には、計算済みの金額が入ります。
送料の計算は上の普通の `fn` にあり、この行はその結果の表示を担当します。

### 間違いと範囲

`standardTotal "3200"` と文字列を渡すと、整数が必要なため型検査に失敗します。
TypeScriptの `standardTotal("3200")` も同じ理由で失敗します。
この例では数値の `3200` に直してください。外部から届く文字列を検証して数値に
変換する処理は、別に必要です。

`Int` は金額専用の型ではなく、負数も受け取れます。送料と送料無料の金額は
同じ型なので、順番を取り違えても型検査では分かりません。引数を渡し忘れても、
残りを待つ関数として有効な場合があります。また、TypeScriptの `number` は
小数も受け取れますが、ここで使う `Int` は受け取れません。大きな整数の加算では
Seseragiが実行時に停止し、TypeScriptが数値を返す場合もあります。
この比較は上で定めた小さな非負整数の範囲での計算であり、完成した決済処理では
ありません。

## English counterpart for #700

### Task and result

Orders of ¥5,000 or more ship free; smaller orders have a ¥500 shipping fee.
Calculate the payable amounts for orders with subtotals of ¥3,200 and ¥5,000
under that same rule. Both programs print `3700, 5000`. Each of the three
arguments is a whole-yen amount from 0 through 1,000,000; input validation is
outside this example.

### Read the original function

`fn` defines a function. `totalWithShipping` takes the free-shipping threshold
`freeFrom`, shipping fee `fee` and order `subtotal`, in that order. Each
`: Int` says that the argument is an integer; the final `-> Int` says that the
result is an integer. The earlier arrows lead to the next argument. The
calculation is to the right of `=`.

`if ... then ... else ...` returns the value of the selected branch. If the
subtotal reaches the free-shipping threshold, return it unchanged; otherwise,
add the fee. An ordinary call is `totalWithShipping 5000 500 3200`: put the
arguments after the function name, separated by spaces. This call returns
`3700`.

### Reuse the same conditions

`let standardTotal = totalWithShipping 5000 500` supplies just the first two
arguments and names the resulting function `standardTotal`. That function
still needs a subtotal, so this line does not calculate an order's total yet.
`standardTotal 3200` returns `3700`; `standardTotal 5000` returns `5000`.
Creating a function by supplying only some arguments is called *partial
application*. A name introduced with `let` cannot later be reassigned.

TypeScript can express the same operation with an arrow function. Both
versions keep the shipping conditions in one place and pass only the subtotal
for each order. In Seseragi, the same application syntax that supplies all
arguments can also create a function with fixed conditions. The appeal to try
here is adapting an ordinary function directly to a particular use, rather
than saving characters alone.

### Read the output line

The final `pub effect fn main` declares the public entry used when the program
runs. `effect` marks that this entry interacts with the outside world, here by
printing output. `println` writes a string and a newline. Inside the backtick
string, `${smallOrder}` and `${largeOrder}` insert the calculated amounts.
The ordinary `fn` above owns the shipping calculation; this entry displays its
results.

### Mistakes and limits

Passing a string with `standardTotal "3200"` fails typechecking because the
function needs an integer. TypeScript also rejects `standardTotal("3200")`.
Use the numeric value `3200` in this example. Validating and converting text
received from outside the program requires separate code.

`Int` is not a money-specific type and allows negative values. The fee and
threshold have the same type, so typechecking cannot detect swapped arguments.
Omitting an argument can also be valid because it leaves a function awaiting
the rest. TypeScript's `number` accepts decimals whereas this `Int` parameter
does not. For very large integer additions, Seseragi can stop at runtime while
TypeScript returns a number. The comparison applies to the small nonnegative
integer domain stated above; it is not a complete payment implementation.

## Verification and limits of the evidence

Run through the existing example lane and the focused site regression test:

```sh
bun apps/site/scripts/check-examples.ts
bun test apps/site/tests/comparisons.test.ts
```

The example lane separately builds/typechecks and executes the exact Seseragi
source and runs strict `tsc --noEmit` plus Bun on the exact TS source. It checks
stdout byte for byte against the same expected file. The selected snippets
do not depend on #679's top-level-expression issue or #683's polymorphic `let`
issue: the Seseragi entry is explicit and the configured function is `Int -> Int`.

Supplementary probes keep the declarations unchanged and replace only the
output entry. Ten cases cover zero, below/equal/above the threshold, another
policy, zero fee, zero threshold and the upper comparison-domain boundary.
Each case exercises both the all-argument call and the partially applied form.
The test expects explicit results, not merely agreement between two versions
that could share the same mistake.

Failure evidence is deliberately separate from valid-domain equality:

- String subtotal: Seseragi reports `SES-T0101` with expected `Int` / actual
  `String`; TypeScript reports `TS2345`.
- Decimal subtotal `3200.5`: Seseragi rejects `Float` where `Int` is required;
  TypeScript typechecks and returns `3700.5`.
- With threshold `9007199254740991`, fee `500`, and subtotal
  `9007199254740990`, Seseragi builds but stops with `runtime defect`;
  TypeScript typechecks and prints `9007199254741490`. This is outside the
  shared domain and is not described as identical failure behavior.
- Neither snippet validates nonnegativity or the example's upper limit.
  Neither mutates input data. Console failures, locale-specific currency
  formatting and production checkout behavior are not compared.

The focused test verifies source hashes, lossless code-panel highlighting,
exact Playground query-source round-trip, absence of a TS Playground link,
and compilation/execution of that linked source through the committed WASM
driver and the existing Playground runtime. This is runtime evidence, not
evidence of a user pressing Run in a real browser.

Local verification on 2026-10-01 used TypeScript 5.8.3, Bun 1.3.9 and the
repository's Seseragi 0.61.19 development CLI (reported build commit
`ab42da841d76`, x86_64 Linux). Formatting, focused TypeScript checking, the
site-owned example gate and the two source/Playground tests passed. No compiler
or runtime source was changed. Full site rendering and browser review remain
blocked by the previously recorded environment/render problems; they were not
retried here. Japanese rendered-page lint and #702 first-time-reader review
remain pending until this comparison is used in a rendered page. The selection
review here is agent/editor self-review, not first-time-reader acceptance.

Semantic sources: [application](../../spec/01-syntax.md#14-関数適用),
[built-in types](../../spec/02-types.md#22-組み込み型),
[Console](../../spec/09-standard-library.md#912-console), and
[performance rules](../../spec/14-performance.md). These are internal evidence
links; they do not replace the local reader explanation above.
