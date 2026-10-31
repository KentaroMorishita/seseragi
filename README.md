<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/brand/extension/logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/brand/extension/logo-light.svg">
    <img src="./assets/brand/extension/logo-light.svg" alt="Seseragi" width="720">
  </picture>
</p>

<p align="center">English | <a href="./README.ja.md">日本語</a></p>

<p align="center"><strong>One language for data shapes, state and UI.</strong></p>

<p align="center">
  <a href="https://seseragi-docs.vercel.app/en">Docs</a>
  · <a href="https://seseragi.vercel.app/">Playground</a>
  · <a href="https://seseragi.vercel.app/tour/">Tour</a>
  · <a href="./extensions/seseragi/README.md">VS Code</a>
</p>

## Write complex apps and keep the code clear

As an app grows, its code gets murky.
State piles up and exceptions come from everywhere. A function that looks like a plain calculation turns out to make network calls.
Before long, reading the code no longer tells you what will happen.

Seseragi is a language built to push back against that.
Its goal is to see how far a complex app can go while its code stays clear.

The name means "a babbling brook", and that is the kind of code it aims for.
Data flows downstream through pipelines, and you can see to the bottom: where each value changes shape, and where things can fail.
To get there, Seseragi makes these promises:

- **Nothing dangerous is hidden** — there is no `null`, no `undefined` and no exceptions. A value that may be missing is a `Maybe`; work that may fail is an `Either` or an `Effect`, and it shows in the type
- **No magic** — type inference and implicit choices succeed only when the compiler can give a single reason. When it is unsure, it reports an error instead of guessing
- **Dependencies are inferred; failures are designed** — the compiler collects the services a function uses (console, clock, HTTP and so on) into its type. Which failures can happen is a type you design
- **Abstractions stay readable** — the goal is not shorter code. It is code where the shape of the data and the flow of the work can be read at a glance

Seseragi compiles to TypeScript and runs on Bun, Node.js and in the browser.
CLI tools and web app UIs are written in the same language, with the same ideas.

> [!NOTE]
> Seseragi is an experimental language under development. Its syntax and semantics may still change.

## Sound familiar from TypeScript?

- You model state with optional `?` properties, and impossible states such as "finished loading, but no data and no error" still pass the type checker
- The type does not tell you whether a function throws. The value in `catch` is `unknown`, and you only find out what it is at run time
- A function that looks like a plain calculation was doing network calls or logging behind your back

Seseragi catches all of these before the program runs. Let's go through them.

### Impossible states and missed cases

When loading data in React and similar libraries, state is often written like this:

```ts
type UserState = {
  isLoading: boolean
  user?: { name: string }
  error?: string
}

function message(state: UserState): string {
  if (state.isLoading) return "Loading..."
  if (state.error) return `Error: ${state.error}`
  return `Hello, ${state.user?.name}`
}
```

This type also allows impossible combinations, such as "loading, but with an error".
Passing `{ isLoading: false }` prints `Hello, undefined`, and it still compiles in strict mode.

In Seseragi, you write the state as "exactly one of these":

```seseragi
type UserState =
  | Loading
  | Loaded String
  | Failed String

fn message state: UserState -> String =
  match state {
    Loading -> "Loading..."
    Loaded name -> `Hello, ${name}`
    Failed reason -> `Error: ${reason}`
  }
```

A name exists only in `Loaded`, so you can never print `undefined`.
On top of that, `match` always checks that every state is handled. Add a new state and forget to handle it, and you get an error before the build.

```text
error[SES-T0301]: This match does not cover every possible value
  |   match state {
  |   ^ missing patterns: Failed _
```

[▶ Open in Playground](https://seseragi.vercel.app/?source=type%20UserState%20%3D%0A%20%20%7C%20Loading%0A%20%20%7C%20Loaded%20String%0A%20%20%7C%20Failed%20String%0A%0Afn%20message%20state%3A%20UserState%20-%3E%20String%20%3D%0A%20%20match%20state%20%7B%0A%20%20%20%20Loading%20-%3E%20%22Loading...%22%0A%20%20%20%20Loaded%20name%20-%3E%20%60Hello%2C%20%24%7Bname%7D%60%0A%20%20%20%20Failed%20reason%20-%3E%20%60Error%3A%20%24%7Breason%7D%60%0A%20%20%7D%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20Loading%20%7C%3E%20message%20%7C%3E%20println%0A%20%20Loaded%20%22Aoi%22%20%7C%3E%20message%20%7C%3E%20println%0A%20%20Failed%20%22timeout%22%20%7C%3E%20message%20%7C%3E%20println%0A%7D%0A)

### Failures as types instead of exceptions

In TypeScript, whether a function can fail does not show in its signature.

```ts
function parseNumber(text: string): number {
  const value = Number(text)
  if (Number.isNaN(value)) throw new Error(`not a number: ${text}`)
  return value
}
```

In Seseragi, work that may fail returns `Either<ErrorType, SuccessType>`.
Success is `Right value` and failure is `Left error`. Callers can see from the type that it may fail.

To call several steps that may fail, use `do` notation. `<-` means "take the value out if it succeeded".
If any step returns a `Left`, the block stops there and that `Left` becomes the result. You write only the success path, top to bottom, without `try/catch`.

```seseragi
import * as either from "std/either"
import * as int from "std/int"

fn parseNumber text: String -> Either<String, Int> =
  int.parse text |> either.mapLeft (\_ -> `not a number: ${text}`)

fn divide total: Int -> count: Int -> Either<String, Int> = match count {
  0 -> Left "cannot divide by zero"
  _ -> Right (total / count)
}

fn calculate left: String -> right: String -> Either<String, Int> = do {
  total <- parseNumber left
  count <- parseNumber right
  divide total count
}

// calculate "84" "2"    => Right 42
// calculate "84" "zero" => Left not a number: zero
// calculate "84" "0"    => Left cannot divide by zero
```

[▶ Open in Playground](https://seseragi.vercel.app/?source=import%20%2A%20as%20either%20from%20%22std%2Feither%22%0Aimport%20%2A%20as%20int%20from%20%22std%2Fint%22%0A%0Afn%20parseNumber%20text%3A%20String%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%0A%20%20int.parse%20text%20%7C%3E%20either.mapLeft%20%28%5C_%20-%3E%20%60not%20a%20number%3A%20%24%7Btext%7D%60%29%0A%0Afn%20divide%20total%3A%20Int%20-%3E%20count%3A%20Int%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%20match%20count%20%7B%0A%20%200%20-%3E%20Left%20%22cannot%20divide%20by%20zero%22%0A%20%20_%20-%3E%20Right%20%28total%20%2F%20count%29%0A%7D%0A%0A%2F%2F%20The%20first%20Left%20stops%20the%20block%20and%20becomes%20the%20result%0Afn%20calculate%20left%3A%20String%20-%3E%20right%3A%20String%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%20do%20%7B%0A%20%20total%20%3C-%20parseNumber%20left%0A%20%20count%20%3C-%20parseNumber%20right%0A%20%20divide%20total%20count%0A%7D%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%222%22%29%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%22zero%22%29%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%220%22%29%0A%7D%0A)

### Side effects show in the type

Side effects such as printing or network calls can only happen inside an `effect fn`.
Calling `println` from a plain `fn` is a type error.

```seseragi
fn greet name: String -> Unit = println `Hello, ${name}`
```

```text
error[SES-T0101]: Expression type does not match the expected type
  = expected: Unit
  = actual: Effect<{ console: Console }, ConsoleError, Unit>
```

`Effect<{ console: Console }, ConsoleError, Unit>` means
"work that uses the console, may fail with `ConsoleError`, and returns `Unit` on success".
The type alone tells you what it depends on and how it can fail.

You design the failures of an `effect fn` as a type and declare them with `fails`.

```seseragi
type OrderError =
  | OutOfStock String
  | TooMany Int

effect fn reserve item: String -> quantity: Int -> String fails OrderError =
  match (item, quantity) {
    ("tea", _) -> fail $ OutOfStock item
    (_, n) when n > 10 -> fail $ TooMany n
    _ -> succeed `${quantity} x ${item}`
  }
```

[▶ Open in Playground](https://seseragi.vercel.app/?source=import%20%2A%20as%20effects%20from%20%22std%2Feffect%22%0A%0Atype%20OrderError%20%3D%0A%20%20%7C%20OutOfStock%20String%0A%20%20%7C%20TooMany%20Int%0A%0Aeffect%20fn%20reserve%20item%3A%20String%20-%3E%20quantity%3A%20Int%20-%3E%20String%20fails%20OrderError%20%3D%0A%20%20match%20%28item%2C%20quantity%29%20%7B%0A%20%20%20%20%28%22tea%22%2C%20_%29%20-%3E%20fail%20%24%20OutOfStock%20item%0A%20%20%20%20%28_%2C%20n%29%20when%20n%20%3E%2010%20-%3E%20fail%20%24%20TooMany%20n%0A%20%20%20%20_%20-%3E%20succeed%20%60%24%7Bquantity%7D%20x%20%24%7Bitem%7D%60%0A%20%20%7D%0A%0Afn%20explain%20error%3A%20OrderError%20-%3E%20String%20%3D%20match%20error%20%7B%0A%20%20OutOfStock%20item%20-%3E%20%60%24%7Bitem%7D%20is%20out%20of%20stock%60%0A%20%20TooMany%20quantity%20-%3E%20%60cannot%20reserve%20%24%7Bquantity%7D%20at%20once%60%0A%7D%0A%0Aeffect%20fn%20order%20item%3A%20String%20-%3E%20quantity%3A%20Int%20-%3E%20String%20fails%20Never%20%3D%0A%20%20reserve%20item%20quantity%0A%20%20%7C%3E%20effects.recover%20%28%5Cerror%20-%3E%20succeed%20%24%20explain%20error%29%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20coffee%20%3C-%20order%20%22coffee%22%202%0A%20%20println%20coffee%0A%20%20tea%20%3C-%20order%20%22tea%22%201%0A%20%20println%20tea%0A%20%20many%20%3C-%20order%20%22coffee%22%2020%0A%20%20println%20many%0A%7D%0A)

## More features

### Pipelines

`|>` passes a value on to the next function.
Functions are curried, so a function given only some of its arguments can go straight into a pipeline.

```seseragi
import * as arrays from "std/array"

fn double n: Int -> Int = n * 2

pub effect fn main =
  [1, 2, 3, 4]
    |> arrays.filter (\n: Int -> n > 2)
    |> map double
    |> sum
    |> show
    |> println
// => 14
```

### UI in the same language

The UI is written with the same types and functions as everything above.
State lives in a `Signal`, and `signals.map` builds the view from the state.
When a button changes the state, the view updates automatically. No React, no JSX.

```seseragi
import * as dom from "std/web/dom"
import * as html from "std/web/html"
import * as signals from "std/signal"
import { MutableSignal } from "std/signal"

fn label current: Int -> html.Html<Task<Unit>> = html.p { children: `Count: ${current}` }

fn view count: MutableSignal<Int> -> current: Int -> html.Html<Task<Unit>> =
  html.div {
    children: [
      label current,
      html.button {
        onClick: signals.update (\n: Int -> n + 1) count,
        children: "+1"
      },
      html.button { onClick: signals.set 0 count, children: "Reset" }
    ]
  }

fn perform action: Task<Unit> -> Task<Unit> = action

pub effect fn main -> Unit with dom: dom.Dom fails String = do {
  count <- signals.make 0
  target <- dom.query "#app"
    |> mapError (\error: dom.DomError -> "#app not found")
  dom.run (dom.defaultOptions ()) target perform (signals.map (view count) count)
    |> mapError (\error: dom.DomRuntimeError<Never> -> "render failed")
}
```

[▶ Open in Playground](https://seseragi.vercel.app/?source=import%20%2A%20as%20dom%20from%20%22std%2Fweb%2Fdom%22%0Aimport%20%2A%20as%20html%20from%20%22std%2Fweb%2Fhtml%22%0Aimport%20%2A%20as%20signals%20from%20%22std%2Fsignal%22%0Aimport%20%7B%20MutableSignal%20%7D%20from%20%22std%2Fsignal%22%0A%0Afn%20label%20current%3A%20Int%20-%3E%20html.Html%3CTask%3CUnit%3E%3E%20%3D%20html.p%20%7B%20children%3A%20%60Count%3A%20%24%7Bcurrent%7D%60%20%7D%0A%0Afn%20view%20count%3A%20MutableSignal%3CInt%3E%20-%3E%20current%3A%20Int%20-%3E%20html.Html%3CTask%3CUnit%3E%3E%20%3D%0A%20%20html.div%20%7B%0A%20%20%20%20children%3A%20%5B%0A%20%20%20%20%20%20label%20current%2C%0A%20%20%20%20%20%20html.button%20%7B%0A%20%20%20%20%20%20%20%20onClick%3A%20signals.update%20%28%5Cn%3A%20Int%20-%3E%20n%20%2B%201%29%20count%2C%0A%20%20%20%20%20%20%20%20children%3A%20%22%2B1%22%0A%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20html.button%20%7B%20onClick%3A%20signals.set%200%20count%2C%20children%3A%20%22Reset%22%20%7D%0A%20%20%20%20%5D%0A%20%20%7D%0A%0Afn%20perform%20action%3A%20Task%3CUnit%3E%20-%3E%20Task%3CUnit%3E%20%3D%20action%0A%0Apub%20effect%20fn%20main%20-%3E%20Unit%20with%20dom%3A%20dom.Dom%20fails%20String%20%3D%20do%20%7B%0A%20%20count%20%3C-%20signals.make%200%0A%20%20target%20%3C-%20dom.query%20%22%23app%22%0A%20%20%20%20%7C%3E%20mapError%20%28%5Cerror%3A%20dom.DomError%20-%3E%20%22%23app%20not%20found%22%29%0A%20%20dom.run%20%28dom.defaultOptions%20%28%29%29%20target%20perform%20%28signals.map%20%28view%20count%29%20count%29%0A%20%20%20%20%7C%3E%20mapError%20%28%5Cerror%3A%20dom.DomRuntimeError%3CNever%3E%20-%3E%20%22render%20failed%22%29%0A%7D%0A)

### The editor and the compiler give the same answer

The CLI, the editor's LSP, the formatter and the browser Playground all use the same compiler core.
The types and errors your editor shows never disagree with what an actual build reports.

### The output is plain TypeScript

Seseragi compiles to TypeScript and runs on Bun, Node.js and in the browser.
For example, the `type UserState` above becomes a union type distinguished by its `tag` property.

```ts
type UserState =
  | { readonly tag: "Loading" }
  | { readonly tag: "Loaded"; readonly value: string }
  | { readonly tag: "Failed"; readonly value: string };
```

Calling existing TypeScript code and npm packages from Seseragi is still in development (see [Status](#status)).

## TypeScript to Seseragi at a glance

| TypeScript | Seseragi |
| --- | --- |
| `const total = 1` | `let total = 1` |
| `function add(a: number, b: number): number { return a + b }` | `fn add a: Int -> b: Int -> Int = a + b` |
| `(x) => x + 1` | `\x -> x + 1` |
| `format(parse(text))` | `text \|> parse \|> format` |
| `{ kind: "a" } \| { kind: "b"; value: number }` | `type T = \| A \| B Int` |
| `switch (value.kind) { ... }` | `match value { ... }` |
| `T \| undefined` | `Maybe<T>` (`Just value` / `Nothing`) |
| `throw` / `try ... catch` | `Either<E, A>`, `effect fn ... fails E` |
| `async function` | `effect fn` (async work is an Effect too) |
| `interface` and its implementations | `trait` and `instance` |
| `{ ...user, name: "Aoi" }` | `{ ...user, name: "Aoi" }` |
| `` `Hello, ${name}` `` | `` `Hello, ${name}` `` |

A detailed side-by-side comparison with TypeScript, covering functions, data, failures, async work and how it differs from React,
is in the docs guide [From TypeScript to Seseragi](https://seseragi-docs.vercel.app/en/docs/guides/typescript-to-seseragi).

## Try it

You can try it in the browser right away, without installing anything.

- **[Docs](https://seseragi-docs.vercel.app/en)** — Learn the language step by step, find task-based guides, and look up rules in the reference
- **[Playground](https://seseragi.vercel.app/)** — Write code, type-check it, run it and preview HTML
- **[A Tour of Seseragi](https://seseragi.vercel.app/tour/)** — Go from Hello world through Effect, Signal and web UI, one step at a time
- **[Samples](./examples/samples/README.md)** — Programs that run on the current compiler

For example, FizzBuzz looks like this:

<!-- canonical-example: path=examples/samples/fizzbuzz/main.ssrg -->
```seseragi
fn fizzBuzz number: Int -> String =
  // A tuple lets one match classify both divisibility rules.
  match (number % 3, number % 5) {
    (0, 0) -> "FizzBuzz"
    (0, _) -> "Fizz"
    (_, 0) -> "Buzz"
    _ -> `${number}`
  }

pub effect fn main = for number <- 1 ..= 30 {
  println $ fizzBuzz number
}
```
<!-- /canonical-example -->

## Install

Download the archive for your OS and CPU from [GitHub Releases](https://github.com/KentaroMorishita/seseragi/releases).
It contains the CLI and the LSP. Running generated programs requires [Bun](https://bun.sh/) 1.4.4 or later.

```sh
seseragi run hello.ssrg          # run a file
seseragi new web my-app          # create a web app project
seseragi dev my-app              # start the dev server
seseragi build hello.ssrg        # build
seseragi format hello.ssrg       # format
```

For step-by-step instructions, see [Local development](https://seseragi-docs.vercel.app/en/docs/learn/local-development) in the docs; to build a web app, see [Build a Web application](https://seseragi-docs.vercel.app/en/docs/guides/web-app).
The VS Code extension (ID: `seseragi-dev.seseragi`) installs from the VSIX in the same GitHub Releases.
See [Seseragi for VS Code](./extensions/seseragi/README.md) for details.

> [!NOTE]
> Bun 1.4.3 and earlier have a vulnerability (CVE-2026-111497).
> If you run on Node.js, use a version that includes the October 2026 security fixes
> (22.23.4 / 24.22.1 / 26.12.1 or later).

## Status

- On the TypeScript target, every feature shown in this README works
- Interop with existing TypeScript code and npm packages is in development
- The standard library for files, HTTP, Stream and so on is only partly implemented
- A target that produces native Rust binaries is also in development (progress: [#809](https://github.com/KentaroMorishita/seseragi/issues/809))

See [STATUS](./docs/STATUS.md) for what works today and [ROADMAP](./docs/ROADMAP.md) for what comes next.

## Contributing to the compiler

The compiler is written in Rust. Building from source requires Rust and Bun
(the Rust version is pinned in `rust-toolchain.toml`).

```sh
# Install the CLI from the main branch (not a release build)
cargo install --git https://github.com/KentaroMorishita/seseragi --locked seseragi-cli

# Start the Playground locally
bun run dev:playground

# Run the checks for what you changed (there are also check:sample / check:playground / check:extension / check:release)
bun run check:rust
```

> [!NOTE]
> Use rustup 1.29.2 or later (earlier versions have a certificate validation vulnerability, CVE-2026-111943).

[Scoped checks](./docs/SCOPED_CHECKS.md) explains which check to run,
and [IMPLEMENTATION](./docs/IMPLEMENTATION.md) describes how the compiler is organized.

| Path | Contents |
| --- | --- |
| `crates/` | Compiler, CLI, LSP, WASM and test runner |
| `runtime/ts/` | Runtime used by the generated TypeScript |
| `apps/playground/` | Playground and Tour |
| `examples/samples/` | Working samples |
| `examples/tour/` | Tour lessons |
| `examples/spec/` | Language specification test cases |
| `docs/spec/` | Language specification |
| `extensions/seseragi/` | VS Code extension |

## License

Apache-2.0
