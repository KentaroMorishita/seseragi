export type PlaygroundSampleDefinition = {
  readonly id: string
  readonly label: string
  readonly category: "基本" | "アプリ" | "型と抽象化"
  readonly sourcePath: string
  readonly stdin: string
  readonly expectedOutput: string
}

export const sampleCatalog: readonly PlaygroundSampleDefinition[] = [
  {
    id: "hello-world",
    label: "Hello world",
    category: "基本",
    sourcePath: "examples/spec/lessons/01-hello-world.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi!",
  },
  {
    id: "pipeline-application",
    label: "関数適用とパイプライン",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/pipeline-application/main.ssrg",
    stdin: "",
    expectedOutput: "Pipeline answer: 42",
  },
  {
    id: "string-add",
    label: "Stringで招待状",
    category: "基本",
    sourcePath: "examples/spec/artifacts/schema-1/string-add/main.ssrg",
    stdin: "",
    expectedOutput: "Hello, Mio! Welcome to Seseragi Night.",
  },
  {
    id: "template-interpolation",
    label: "型安全なtemplate",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/template-interpolation/main.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi: Active",
  },
  {
    id: "array-scoreboard",
    label: "Arrayスコア集計",
    category: "基本",
    sourcePath: "examples/spec/playground/04-array-scoreboard.ssrg",
    stdin: "",
    expectedOutput: "Score total: 108 — perfect checksum!",
  },
  {
    id: "range-comprehension",
    label: "Rangeと内包表記",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/range-comprehension/main.ssrg",
    stdin: "",
    expectedOutput: "even square total: 220",
  },
  {
    id: "comprehension-pattern-filter",
    label: "Patternで値を選ぶ",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/comprehension-pattern-filter/main.ssrg",
    stdin: "",
    expectedOutput: "pattern-filter totals: 4 / 40",
  },
  {
    id: "rock-paper-scissors",
    label: "型付きじゃんけん",
    category: "アプリ",
    sourcePath:
      "examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
    stdin: "rock\nscissors\n",
    expectedOutput: "Player 1 wins!",
  },
  {
    id: "mini-adventure",
    label: "ミニ冒険",
    category: "アプリ",
    sourcePath: "examples/spec/playground/01-mini-adventure.ssrg",
    stdin: "forest\nrope\n",
    expectedOutput: "You cross the ravine and discover a hidden lake!",
  },
  {
    id: "shipping-advisor",
    label: "送料アドバイザー",
    category: "アプリ",
    sourcePath: "examples/spec/playground/02-shipping-advisor.ssrg",
    stdin: "member\nexpress\n",
    expectedOutput: "Member express shipping: 500 yen",
  },
  {
    id: "seseragi-quiz",
    label: "Seseragiクイズ",
    category: "アプリ",
    sourcePath: "examples/spec/playground/03-seseragi-quiz.ssrg",
    stdin: "rust\n",
    expectedOutput: "Correct! The new compiler is implemented in Rust.",
  },
  {
    id: "trait-badges",
    label: "Traitバッジ",
    category: "型と抽象化",
    sourcePath: "examples/spec/playground/05-trait-badges.ssrg",
    stdin: "",
    expectedOutput: "Status badge: active\nMode badge: automatic",
  },
  {
    id: "generic-instance",
    label: "Generic instance",
    category: "型と抽象化",
    sourcePath: "examples/spec/playground/06-generic-instance.ssrg",
    stdin: "",
    expectedOutput: "Generic Maybe: empty\nGeneric Maybe: present",
  },
  {
    id: "newtype-user-id",
    label: "newtypeでUserId",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/newtype-user-id/main.ssrg",
    stdin: "",
    expectedOutput: "UserId keeps its nominal boundary: 42",
  },
  {
    id: "record-profile",
    label: "Recordでプロフィール",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/record-profile/main.ssrg",
    stdin: "",
    expectedOutput: "Record profile: Mio\nPlayer: Mio",
  },
  {
    id: "struct-profile",
    label: "Structでプロフィール更新",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/struct-profile/main.ssrg",
    stdin: "",
    expectedOutput: "Mio: perfect 42",
  },
  {
    id: "user-add-operator",
    label: "自作の足し算",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/user-add-operator/main.ssrg",
    stdin: "",
    expectedOutput: "User Add: 42",
  },
  {
    id: "user-eq-operator",
    label: "自作の等価比較",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/user-eq-operator/main.ssrg",
    stdin: "",
    expectedOutput: "User Eq: same / different",
  },
  {
    id: "user-iterable-comprehension",
    label: "自作Collectionの流れ",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/user-iterable-comprehension/main.ssrg",
    stdin: "",
    expectedOutput: "user collection totals: 35 / 15",
  },
  {
    id: "list-comprehension",
    label: "永続Listの流れ",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/list-comprehension/main.ssrg",
    stdin: "",
    expectedOutput: "persistent List total: 35",
  },
  {
    id: "partial-functor-value",
    label: "Functorを渡す",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/polymorphic-partial-functor/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42",
  },
  {
    id: "partial-constrained-function",
    label: "制約付き関数を渡す",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/partial-constrained-function/main.ssrg",
    stdin: "",
    expectedOutput: "Badge is ready!",
  },
  {
    id: "applicative-maybe",
    label: "Applicativeの流れ",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/applicative-maybe/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42",
  },
  {
    id: "applicative-validation",
    label: "Validationでエラーを集める",
    category: "型と抽象化",
    sourcePath:
      "examples/spec/artifacts/schema-1/applicative-validation/main.ssrg",
    stdin: "",
    expectedOutput: "NameRequired, AgeMustBePositive\nValid user",
  },
  {
    id: "monad-maybe",
    label: "Monadの流れ",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/monad-maybe/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42\nJust 42\nJust 44\nNothing\nJust 42\nNothing",
  },
  {
    id: "monad-either",
    label: "MonadとEither",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/monad-either/main.ssrg",
    stdin: "",
    expectedOutput: "Right 42\nLeft stopped",
  },
]
