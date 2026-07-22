export const sampleLevels = ["初級", "中級", "上級", "実践"] as const

export type PlaygroundSampleLevel = (typeof sampleLevels)[number]

export type PlaygroundSampleDefinition = {
  readonly id: string
  readonly label: string
  readonly level: PlaygroundSampleLevel
  readonly sequence: number
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
    level: "初級",
    sequence: 1,
    summary: "最小の公開mainから文字列を出力し、まず実行の手触りをつかみます。",
    concepts: ["effect fn", "main", "println"],
    sourcePath: "examples/playground/01-hello-world.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi!",
  },
  {
    id: "functions-and-pipelines",
    label: "関数とパイプライン",
    level: "初級",
    sequence: 2,
    summary: "小さな関数を定義し、値を|>で左から右へ流して組み立てます。",
    concepts: ["fn", "curry", "|>"],
    sourcePath: "examples/playground/02-functions-and-pipelines.ssrg",
    stdin: "",
    expectedOutput: "Answer: 42",
  },
  {
    id: "data-and-patterns",
    label: "データ型とPattern match",
    level: "初級",
    sequence: 3,
    summary: "取り得る状態をADTで表し、matchで漏れなく値を取り出します。",
    concepts: ["ADT", "constructor", "match"],
    sourcePath: "examples/playground/03-data-and-patterns.ssrg",
    stdin: "",
    expectedOutput: "Shipped to Osaka",
  },
  {
    id: "collections",
    label: "Collectionを組み立てる",
    level: "初級",
    sequence: 4,
    summary:
      "Range、内包表記、joinとMonoidのcombineでCollectionを読みやすく表示します。",
    concepts: ["Range", "内包表記", "join", "Monoid", "combine"],
    sourcePath: "examples/playground/04-collections.ssrg",
    stdin: "",
    expectedOutput: "[#2 / #4 / #6 / #8]",
  },
  {
    id: "records",
    label: "Recordと更新",
    level: "中級",
    sequence: 5,
    summary: "Recordのfield、spread更新、width typingを一つの値で確認します。",
    concepts: ["Record", "spread", "width typing"],
    sourcePath: "examples/playground/05-records.ssrg",
    stdin: "",
    expectedOutput: "Aoi Tanaka",
  },
  {
    id: "generic-structs",
    label: "Generic Structとimpl",
    level: "中級",
    sequence: 6,
    summary: "Box<Int>の推論と、genericなmethodが型を保つ流れを確認します。",
    concepts: ["Struct", "generic inference", "impl"],
    sourcePath: "examples/playground/06-generic-structs.ssrg",
    stdin: "",
    expectedOutput: "Box result: 42",
  },
  {
    id: "newtypes",
    label: "newtypeで境界を作る",
    level: "中級",
    sequence: 7,
    summary: "実行時コストを増やさず、Intと混ざらないdomainのIDを作ります。",
    concepts: ["newtype", "nominal type", "pattern"],
    sourcePath: "examples/playground/07-newtypes.ssrg",
    stdin: "",
    expectedOutput: "/users/42",
  },
  {
    id: "effects-and-do",
    label: "Effectをdoで合成",
    level: "中級",
    sequence: 8,
    summary: "複数のEffectとpureなletを、do blockの中で順番に合成します。",
    concepts: ["Effect", "do", "let"],
    sourcePath: "examples/playground/08-effects-and-do.ssrg",
    stdin: "",
    expectedOutput: "Building Seseragi...\nDone.",
  },
  {
    id: "either-and-monad",
    label: "EitherとMonad",
    level: "上級",
    sequence: 9,
    summary: "PreludeのEither Monadを、doと明示的な>>=の両方で利用します。",
    concepts: ["Either", "Monad", "do / >>="],
    sourcePath: "examples/playground/09-either-and-monad.ssrg",
    stdin: "",
    expectedOutput: "do: 42\n>>=: 42",
  },
  {
    id: "traits-and-instances",
    label: "Traitとinstance",
    level: "上級",
    sequence: 10,
    summary: "型が満たす振る舞いをTraitで宣言し、instanceから選択します。",
    concepts: ["trait", "instance", "dictionary"],
    sourcePath: "examples/playground/10-traits-and-instances.ssrg",
    stdin: "",
    expectedOutput: "active",
  },
  {
    id: "impl-and-operators",
    label: "implと演算子",
    level: "上級",
    sequence: 11,
    summary: "Structのimplでmethodと演算子をdomainの型へまとめます。",
    concepts: ["impl", "operator", "Add / Eq"],
    sourcePath: "examples/playground/11-impl-and-operators.ssrg",
    stdin: "",
    expectedOutput: "Score: 42",
  },
  {
    id: "signal-composition",
    label: "Signalの<$> / <*>",
    level: "上級",
    sequence: 12,
    summary: "SignalのFunctor / Applicative instanceを演算子から利用します。",
    concepts: ["Signal", "<$>", "<*>"],
    sourcePath: "examples/playground/12-signal-composition.ssrg",
    stdin: "",
    expectedOutput: "Signal answer: 42",
  },
  {
    id: "signal-state",
    label: "Signalで状態をまとめる",
    level: "実践",
    sequence: 13,
    summary:
      "複数Signalからderived stateを作り、transactionで一度に更新します。",
    concepts: ["Signal", "derived state", "transaction"],
    sourcePath: "examples/playground/13-signal-state.ssrg",
    stdin: "",
    expectedOutput: "Dashboard total: 52",
  },
  {
    id: "html-components",
    label: "ComponentとStyleでSSR",
    level: "実践",
    sequence: 14,
    summary: "関数componentと再利用可能なStyleからHTMLを組み立て、SSRします。",
    concepts: ["Html", "function component", "reusable Style"],
    sourcePath: "examples/playground/14-html-components.ssrg",
    stdin: "",
    expectedOutput:
      '<main id="app" class="min-h-screen bg-emerald-50 p-8 sm:p-12"><h1 class="mx-auto mb-6 max-w-xl text-3xl font-bold text-emerald-950">Seseragi components</h1><section class="mx-auto max-w-xl" style="--card-shadow: 0 4px 16px #0002; background-color: #fff; box-shadow: var(--card-shadow); border-radius: 16px; padding: 24px"><h2 style="color: #185f50; margin-top: 0">Reusable card</h2><p style="color: #315c53; margin-bottom: 0">Function component from children</p></section></main>',
    outputMode: "html",
  },
  {
    id: "interactive-app",
    label: "Interactive Web App",
    level: "実践",
    sequence: 15,
    summary:
      "pure reducer、typed Msg、Signal、DOM mountを小さなWeb appに統合します。",
    concepts: ["typed Msg", "pure reducer", "dom.app"],
    sourcePath: "examples/playground/15-interactive-app.ssrg",
    stdin: "",
    expectedOutput: "",
    outputMode: "html",
    interactive: true,
  },
]
