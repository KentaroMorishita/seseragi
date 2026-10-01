type SignatureReading = { en: string; ja: string }

// A signature contains arrows inside callback, record, and generic types.
// Only outer arrows separate this callable's parameters.
export function outerArrows(source: string): string[] {
  const parts: string[] = []
  const stack: string[] = []
  let start = 0
  const closing: Record<string, string> = {
    "(": ")",
    "<": ">",
    "{": "}",
    "[": "]",
  }
  for (let index = 0; index < source.length; index++) {
    if (source.slice(index, index + 2) === "->") {
      if (stack.length === 0) {
        parts.push(source.slice(start, index).trim())
        start = index + 2
      }
      index++
      continue
    }
    const char = source[index]
    if (closing[char]) stack.push(closing[char])
    else if (stack.at(-1) === char) stack.pop()
  }
  parts.push(source.slice(start).trim())
  return parts
}

export function signatureReading(
  signature: string,
  kind: string,
  parameters: string[],
  constraints: string[]
): SignatureReading {
  const suffix = signature.lastIndexOf(" where ")
  const body = suffix === -1 ? signature : signature.slice(0, suffix)
  const parts = outerArrows(body)
  let en: string
  let ja: string
  if (
    parts.length > 1 &&
    ["function", "effect-function", "constructor", "operator"].includes(kind)
  ) {
    const first = parts[0].match(/\s([a-z][A-Za-z0-9_]*):\s(.+)$/u)
    const constructorInput =
      kind === "constructor"
        ? parts[0].match(/^[^:]+:\s(.+)$/u)?.[1]
        : undefined
    const inputs = [
      first?.[2] ?? constructorInput ?? parts[0],
      ...parts.slice(1, -1).map((part) => {
        const input = part.match(/^([a-z][A-Za-z0-9_]*):\s(.+)$/u)
        return input?.[2] ?? part
      }),
    ]
    const result = parts.at(-1)!
    en = `Pass ${inputs.length} argument${inputs.length === 1 ? "" : "s"} in this order: ${inputs.join("; ")}. The result has type ${result}.`
    ja = `引数は${inputs.length}個です。空白で区切り、${inputs.join("、")}の順で渡してください。戻り値の型は${result}です。`
    if (inputs.includes("Unit")) {
      en += " Pass () for an argument of type Unit."
      ja += "Unitの引数には()を渡します。"
    }
    if (inputs.some((input) => input.includes("->"))) {
      en +=
        " An argument containing -> is a function value; pass a named function or lambda with the matching input and result types."
      ja +=
        "->を含む引数の型は関数です。入力と結果の型が合う関数名かラムダを渡します。"
    }
    if (result.startsWith("Effect<")) {
      en +=
        " The returned Effect represents work to execute. Execute it in an effectful entry point or combine it with another Effect to obtain its result."
      ja +=
        "返るEffectは、実行する処理を表す値です。実行入口で実行するか、別のEffectと組み合わせて実行すると結果を得られます。"
      en +=
        " Effect<R, E, A> names the required services R, recoverable failure E, and successful value A."
      ja +=
        "Effect<R, E, A>のRは必要なサービス、Eは回復可能な失敗、Aは成功したときの値の型です。"
    } else if (result.startsWith("Task<")) {
      en +=
        " Task<A> is an Effect requiring no services and having no recoverable failure. Execute the returned computation to obtain an A value."
      ja +=
        "Task<A>は、要求するサービスも回復可能な失敗もないEffectです。Aの値を得るには、返った処理を実行してください。"
    } else if (result.startsWith("Stream<")) {
      en +=
        " Consume the Stream to obtain its elements in sequence. R names the required services, E the recoverable failure type, and A the element type."
      ja +=
        "Streamを消費すると、要素を順に取り出せます。Rは必要なサービス、Eは回復可能な失敗、Aは各要素の型です。"
    } else if (result.startsWith("Signal<")) {
      en +=
        " A Signal manages a current value that can change. Use the signal read operation to obtain the contained value."
      ja +=
        "Signalは、変化する現在値を管理します。中の値を得るには、Signalの読み取り操作を使ってください。"
    } else if (result.startsWith("Ref<")) {
      en +=
        " A Ref is a handle to explicitly managed mutable state. Read or update its value through the Ref operations."
      ja +=
        "Refは、明示的に読み書きする可変状態です。中の値にはRefの読み取り・更新操作を通してアクセスします。"
    } else if (result.startsWith("Maybe<")) {
      en +=
        " Match Just for a present result and Nothing when there is no result."
      ja += "結果に値がある場合はJust、ない場合はNothingをmatchで扱います。"
    } else if (result.startsWith("Either<")) {
      en += " Use match to handle Left for failure and Right for success."
      ja +=
        "失敗はLeft、成功はRightで表します。matchでどちらの場合も扱ってください。"
    }
  } else if (kind === "constructor") {
    en = `This constructor takes no value arguments and creates a value of type ${body}.`
    ja = `このコンストラクターには値の引数がありません。${body}型の値を作ります。`
  } else if (kind === "instance") {
    en =
      "An instance supplies a trait's operations for the types named in this declaration. Call those operations to use the implementation; an instance is not itself a callable function."
    ja =
      "instanceは、この宣言の型でtraitの操作を使うための実装です。traitの操作を呼ぶと、この実装が使われます。instance自体を関数として呼び出すことはできません。"
  } else if (kind === "alias") {
    en =
      "An alias names another type; it does not create a separate type or a value constructor."
    ja =
      "aliasは別の型に名前を付けます。新しい別の型や、値を作るコンストラクターにはなりません。"
  } else if (kind === "trait") {
    en =
      "A trait names a set of shared operations. A where clause requires an implementation for the chosen types."
    ja =
      "traitは型に共通する操作を定めます。where節でこのtraitを指定すると、選んだ型に対する実装が必要になります。"
  } else if (kind === "value") {
    en = `This is a value of type ${body}, not a function call.`
    ja = `これは${body}型の値です。関数の呼び出しではありません。`
  } else if (kind.startsWith("opaque")) {
    en =
      "This type keeps its representation private. Use the owning module's constructors and operations rather than accessing its internals."
    ja =
      "この型の内部の表現は非公開です。値を作ったり取り出したりするには、同じモジュールが提供する関数を使ってください。"
  } else {
    en =
      "This declaration names a type used by the operations in this module. The angle brackets list its type arguments."
    ja =
      "この宣言は、同じモジュールの操作が使う型を示します。山括弧には、その型に渡す型引数を書きます。"
  }
  if (parameters.length) {
    en +=
      parameters.length === 1
        ? ` ${parameters[0]} names a type parameter. At each use it is resolved to a type consistent with the arguments and result; use a type annotation when needed.`
        : ` ${parameters.join(", ")} name type parameters. At each use they are resolved to types consistent with the arguments and result; use a type annotation when needed.`
    ja += `型パラメーターは${parameters.join("、")}です。使用する式で、引数と戻り値に合う具体的な型に決まります。必要なら型注釈で指定してください。`
  }
  if (constraints.length) {
    en += ` The selected types must provide ${constraints.join(", ")}.`
    ja += `選んだ型には${constraints.join("、")}の実装が必要です。`
  }
  return { en, ja }
}
