export type PlaygroundSampleDefinition = {
  readonly id: string
  readonly label: string
  readonly category: "基本" | "アプリ" | "型と抽象化" | "Web"
  readonly summary: string
  readonly concepts: readonly string[]
  readonly sourcePath: string
  readonly stdin: string
  readonly expectedOutput: string
  readonly outputMode?: "text" | "html"
  readonly interactive?: boolean
}

export const sampleCatalog: readonly PlaygroundSampleDefinition[] = [
  {
    id: "hello-world",
    label: "Hello world",
    category: "基本",
    summary:
      "公開mainからConsole Effectを実行する、最小のSeseragi programです。",
    concepts: ["effect fn", "Console", "公開main"],
    sourcePath: "examples/spec/lessons/01-hello-world.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi!",
  },
  {
    id: "pipeline-application",
    label: "関数適用とパイプライン",
    category: "基本",
    summary: "低優先順位の$と左から流す|>を、同じ関数適用として比較します。",
    concepts: ["関数適用", "$", "|>"],
    sourcePath:
      "examples/spec/artifacts/schema-1/pipeline-application/main.ssrg",
    stdin: "",
    expectedOutput: "Pipeline answer: 42",
  },
  {
    id: "template-interpolation",
    label: "型安全なtemplate",
    category: "基本",
    summary:
      "値をtemplateへ埋め込み、選択されたShow evidenceで文字列化します。",
    concepts: ["template", "Show", "型クラス"],
    sourcePath:
      "examples/spec/artifacts/schema-1/template-interpolation/main.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi: Active",
  },
  {
    id: "range-comprehension",
    label: "Rangeと内包表記",
    category: "基本",
    summary:
      "Rangeをgeneratorへ流し、guardと変換を一つの内包表記で組み立てます。",
    concepts: ["Range", "内包表記", "guard"],
    sourcePath:
      "examples/spec/artifacts/schema-1/range-comprehension/main.ssrg",
    stdin: "",
    expectedOutput: "even square total: 220",
  },
  {
    id: "comprehension-pattern-filter",
    label: "Patternで値を選ぶ",
    category: "基本",
    summary:
      "refutable patternをfilterとして使い、payloadだけを安全に集計します。",
    concepts: ["pattern", "Array", "filter"],
    sourcePath:
      "examples/spec/artifacts/schema-1/comprehension-pattern-filter/main.ssrg",
    stdin: "",
    expectedOutput: "pattern-filter totals: 4 / 40",
  },
  {
    id: "rock-paper-scissors",
    label: "型付きじゃんけん",
    category: "アプリ",
    summary:
      "ADT、match、Stdin、typed failureを一つの実行programへまとめます。",
    concepts: ["ADT", "match", "typed Effect"],
    sourcePath:
      "examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
    stdin: "rock\nscissors\n",
    expectedOutput: "Player 1 wins!",
  },
  {
    id: "mini-adventure",
    label: "ミニ冒険",
    category: "アプリ",
    summary: "標準入力をdomainの選択へ変換する、小さなbranching storyです。",
    concepts: ["Stdin", "match", "domain modeling"],
    sourcePath: "examples/spec/playground/01-mini-adventure.ssrg",
    stdin: "forest\nrope\n",
    expectedOutput: "You cross the ravine and discover a hidden lake!",
  },
  {
    id: "signal-transaction",
    label: "Signalのtransaction",
    category: "アプリ",
    summary:
      "複数Signalの変更を一度にcommitし、derived値を一回だけ通知します。",
    concepts: ["Signal", "transaction", "atomic update"],
    sourcePath: "examples/spec/artifacts/schema-1/signal-transaction/main.ssrg",
    stdin: "",
    expectedOutput: "signal: 63",
  },
  {
    id: "signal-switch-map",
    label: "SignalのswitchMap",
    category: "アプリ",
    summary: "選択中のSignalだけを購読し、dependencyを動的に切り替えます。",
    concepts: ["Signal", "switchMap", "subscription"],
    sourcePath: "examples/spec/artifacts/schema-1/signal-switch-map/main.ssrg",
    stdin: "",
    expectedOutput: "switchMap: 42",
  },
  {
    id: "newtype-user-id",
    label: "newtypeでUserId",
    category: "型と抽象化",
    summary: "runtime costを増やさず、Intと混同できないnominal IDを作ります。",
    concepts: ["newtype", "nominal type", "pattern"],
    sourcePath: "examples/spec/artifacts/schema-1/newtype-user-id/main.ssrg",
    stdin: "",
    expectedOutput: "UserId keeps its nominal boundary: 42",
  },
  {
    id: "generic-struct",
    label: "Generic Structの推論",
    category: "型と抽象化",
    summary: "fieldの値からBox<Int>を推論し、明示型引数なしで構築します。",
    concepts: ["Struct", "generic inference", "field access"],
    sourcePath: "examples/spec/artifacts/schema-1/generic-struct/main.ssrg",
    stdin: "",
    expectedOutput: "Generic Struct: 42",
  },
  {
    id: "user-add-operator",
    label: "自作の足し算",
    category: "型と抽象化",
    summary: "user-defined Add instanceを通して+を型安全にdispatchします。",
    concepts: ["operator", "Add", "instance"],
    sourcePath: "examples/spec/artifacts/schema-1/user-add-operator/main.ssrg",
    stdin: "",
    expectedOutput: "User Add: 42",
  },
  {
    id: "applicative-validation",
    label: "Validationでエラーを集める",
    category: "型と抽象化",
    summary: "Applicativeで独立した検証を合成し、複数errorを同時に集めます。",
    concepts: ["Applicative", "Validation", "error accumulation"],
    sourcePath:
      "examples/spec/artifacts/schema-1/applicative-validation/main.ssrg",
    stdin: "",
    expectedOutput: "NameRequired, AgeMustBePositive\nValid user",
  },
  {
    id: "monad-maybe",
    label: "型クラスを定義する",
    category: "型と抽象化",
    summary: "Functor / Applicative / Monadをuserlandで定義する完全版です。",
    concepts: ["trait定義", "instance", "Monad"],
    sourcePath: "examples/spec/artifacts/schema-1/monad-maybe/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42\nJust 42\nJust 44\nNothing\nJust 42\nNothing",
  },
  {
    id: "monad-either",
    label: "PreludeのMonadとEither",
    category: "型と抽象化",
    summary: "PreludeのEither instanceをdoと明示>>=の両方から利用します。",
    concepts: ["Prelude", "Either", "do / >>="],
    sourcePath: "examples/spec/artifacts/schema-1/monad-either/main.ssrg",
    stdin: "",
    expectedOutput: "Right 42\nRight 42\nLeft stopped\nLeft stopped",
  },
  {
    id: "signal-applicative",
    label: "Signalの<$> / <*>",
    category: "型と抽象化",
    summary: "SignalのFunctor / Applicative instanceを演算子から選択します。",
    concepts: ["Signal", "<$>", "<*>"],
    sourcePath: "examples/spec/artifacts/schema-1/signal-applicative/main.ssrg",
    stdin: "",
    expectedOutput: "Signal Functor / Applicative: 42",
  },
  {
    id: "web-html-ssr",
    label: "HTMLをSSR preview",
    category: "Web",
    summary: "pureなHtml treeをescapeし、安全な文字列としてSSR previewします。",
    concepts: ["Html tree", "escaping", "renderToString"],
    sourcePath: "examples/spec/artifacts/schema-1/web-html-ssr/main.ssrg",
    stdin: "",
    expectedOutput:
      '<div id="app" class="container"><p>Hello &lt;Seseragi&gt;</p><button type="button">OK</button></div>',
    outputMode: "html",
  },
  {
    id: "web-html-components-style",
    label: "関数コンポーネントとStyle",
    category: "Web",
    summary:
      "関数component、reusable Style、CSS variables、utility classを組み合わせます。",
    concepts: ["function component", "html.Style", "CSS variables"],
    sourcePath:
      "examples/spec/artifacts/schema-1/web-html-components-style/main.ssrg",
    stdin: "",
    expectedOutput:
      '<main id="app" class="min-h-screen bg-emerald-50 p-8 sm:p-12"><h1 class="mx-auto mb-6 max-w-xl text-3xl font-bold text-emerald-950">Seseragi components</h1><section class="mx-auto max-w-xl" style="--card-shadow: 0 4px 16px #0002; background-color: #fff; box-shadow: var(--card-shadow); border-radius: 16px; padding: 24px"><h2 style="color: #185f50; margin-top: 0">Reusable card</h2><p style="color: #315c53; margin-bottom: 0">Function component from children</p></section></main>',
    outputMode: "html",
  },
  {
    id: "web-dom-counter",
    label: "Signalで動くFlow UI",
    category: "Web",
    summary:
      "pure reducerとdom.appでtyped state machineをinteractiveに動かします。",
    concepts: ["typed Msg", "pure reducer", "dom.app"],
    sourcePath: "examples/spec/artifacts/schema-1/web-dom-counter/main.ssrg",
    stdin: "",
    expectedOutput: "",
    outputMode: "html",
    interactive: true,
  },
]
