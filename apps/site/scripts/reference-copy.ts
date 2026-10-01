// Japanese-first reader copy keyed by the exact compiler description.
// Both locales are reviewed here; compiler metadata and signatures stay intact.
type ReferenceDescription = { ja: string; en: string }

const descriptions: Readonly<Record<string, ReferenceDescription>> = {
  "Returns false at the first element rejected by the predicate.": {
    ja: "条件を調べる関数がFalseを返した時点でFalseを返します。",
    en: "Returns False as soon as the condition function returns False.",
  },
  "Returns true at the first element accepted by the predicate.": {
    ja: "条件を調べる関数がTrueを返した時点でTrueを返します。",
    en: "Returns True as soon as the condition function returns True.",
  },
  "Combines a reducible collection using its Monoid instance.": {
    ja: "Monoidが定める空の値から始め、結合操作でコレクションの要素を一つにまとめます。",
    en: "Starts with the empty value defined by Monoid and combines the collection elements into one value.",
  },
  "Runs one Effect for every value exposed by an Iterable instance.": {
    ja: "コレクションから順に値を取り出すIterableの操作を使い、各値に対してEffectを一つずつ実行します。",
    en: "Uses Iterable to obtain the collection values and runs one Effect for each value.",
  },
  "Joins a reducible collection of strings with a separator.": {
    ja: "コレクションの文字列を、指定した区切り文字でつなぎます。",
    en: "Joins a reducible collection of strings with a separator.",
  },
  "Reads the next Iterator value and its remaining Iterator.": {
    ja: "Iteratorから次の値と、残りを読むIteratorを取り出します。",
    en: "Reads the next Iterator value and its remaining Iterator.",
  },
  "Multiplies every element of a reducible collection from one.": {
    ja: "1から始めて、コレクションの全要素を掛け合わせます。",
    en: "Multiplies every element of a reducible collection from one.",
  },
  "Folds a reducible collection into one accumulated value.": {
    ja: "最初の値に各要素を順に組み合わせ、一つの結果を求めます。",
    en: "Combines each element in turn with an accumulated value to produce one result.",
  },
  "Adds every element of a reducible collection from zero.": {
    ja: "0から始めて、コレクションの全要素を足し合わせます。",
    en: "Adds every element of a reducible collection from zero.",
  },
  "Builds a lazy Iterator from an initial state and step function.": {
    ja: "初期状態と次の状態を求める関数から、必要なときに値を計算するIteratorを作ります。",
    en: "Builds a lazy Iterator from an initial state and step function.",
  },
  "Writes text followed by a newline through Console.": {
    ja: "Consoleを使い、文字列の後に改行を付けて出力します。",
    en: "Writes text followed by a newline through Console.",
  },
  "Writes text through Console without adding a newline.": {
    ja: "Consoleを使い、改行を付けずに文字列を出力します。",
    en: "Writes text through Console without adding a newline.",
  },
  "Renders a value through its Show instance and writes it through Console.": {
    ja: "値をShowの実装で文字列にし、Consoleへ出力します。",
    en: "Renders a value through its Show instance and writes it through Console.",
  },
  "Reads one optional line through Stdin.": {
    ja: "Stdinから一行を読みます。行がなければNothingを返します。",
    en: "Reads one optional line through Stdin.",
  },
  "Transparent alias for an Effect with no environment and no recoverable failure.":
    {
      ja: "環境のサービスを必要とせず、回復可能な失敗もないEffectの型別名です。",
      en: "Transparent alias for an Effect with no environment and no recoverable failure.",
    },
  "Represents either a typed failure or a success value.": {
    ja: "失敗の値か成功の値の、どちらか一方を表します。",
    en: "Represents either a typed failure or a success value.",
  },
  "Constructs a present Maybe value.": {
    ja: "値があることを表すJustを作ります。",
    en: "Constructs a present Maybe value.",
  },
  "Constructs the failure side of Either.": {
    ja: "Eitherの失敗側の値であるLeftを作ります。",
    en: "Constructs the failure side of Either.",
  },
  "Represents an optional value as Nothing or Just.": {
    ja: "値がないNothingか、値があるJustのどちらかを表します。",
    en: "Represents an optional value as Nothing or Just.",
  },
  "Constructs an empty Maybe value.": {
    ja: "値がないことを表すNothingを作ります。",
    en: "Constructs an empty Maybe value.",
  },
  "Constructs the success side of Either.": {
    ja: "Eitherの成功側の値であるRightを作ります。",
    en: "Constructs the success side of Either.",
  },
  "Constructs the equal Ordering result.": {
    ja: "二つの値が等しいことを表すEqualを作ります。",
    en: "Constructs the equal Ordering result.",
  },
  "Constructs the greater-than Ordering result.": {
    ja: "左の値が右より大きいことを表すGreaterを作ります。",
    en: "Constructs the greater-than Ordering result.",
  },
  "Constructs the less-than Ordering result.": {
    ja: "左の値が右より小さいことを表すLessを作ります。",
    en: "Constructs the less-than Ordering result.",
  },
  "Represents a total comparison result as Less, Equal, or Greater.": {
    ja: "比較結果をLess、Equal、Greaterのいずれかで表します。",
    en: "Represents a total comparison result as Less, Equal, or Greater.",
  },
  "Selects multiplication through a nominal wrapper; Monoid requires One and homogeneous Mul evidence.":
    {
      ja: "値を別の型で包み、Monoidの結合を乗算として扱います。1を作るOneと、同じ型同士を掛けるMulの実装が必要です。",
      en: "Wraps a value in a separate type so Monoid uses multiplication. Requires One to supply 1 and Mul to multiply values of the same type.",
    },
  "Selects addition through a nominal wrapper; Monoid requires Zero and homogeneous Add evidence.":
    {
      ja: "値を別の型で包み、Monoidの結合を加算として扱います。0を作るZeroと、同じ型同士を足すAddの実装が必要です。",
      en: "Wraps a value in a separate type so Monoid uses addition. Requires Zero to supply 0 and Add to add values of the same type.",
    },
  "Dispatches through the standard trait instance selected by the operand types.":
    {
      ja: "左右の値の型に対応する、標準traitの実装を呼び出します。",
      en: "Dispatches through the standard trait instance selected by the operand types.",
    },
  "Persistent List cons. Precedence 4, right associative; evaluates head then tail once and shares the tail in O(1). The operator section (:) is a curried function.":
    {
      ja: "Listの先頭に要素を追加します。優先順位は4で右結合です。先頭と末尾を一度ずつ計算し、末尾のリストは共有するため追加はO(1)です。(:)はカリー化された関数としても使えます。",
      en: "Persistent List cons. Precedence 4, right associative; evaluates head then tail once and shares the tail in O(1). The operator section (:) is a curried function.",
    },
  "Maybe fallback. Precedence 0, right-associative. Evaluates the left once; evaluates the fallback only for Nothing. Syntax only: no operator section or overload.":
    {
      ja: "Maybeに値がなければ右の式を使います。優先順位は0で右結合です。左は一度だけ計算し、Nothingのときだけ右を計算します。構文専用なので、(??)という関数にしたり再定義したりはできません。",
      en: "Maybe fallback. Precedence 0, right-associative. Evaluates the left once; evaluates the fallback only for Nothing. Syntax only: no operator section or overload.",
    },
  "Operator spelling for a standard trait method.": {
    ja: "標準traitの操作を、演算子の記号で呼び出します。",
    en: "Operator spelling for a standard trait method.",
  },
  "Applies the selected arithmetic trait instance.": {
    ja: "値の型に対応する算術traitの実装を使って計算します。",
    en: "Applies the selected arithmetic trait instance.",
  },
  "Standard type class used for generic dispatch.": {
    ja: "複数の型に共通する操作を定める標準traitです。型ごとの処理はinstanceで定義します。",
    en: "A standard trait defining operations shared by several types. An instance defines the implementation for each type.",
  },
  "Appends the suffix after the collection values.": {
    ja: "コレクションの末尾に指定した値をつなぎます。",
    en: "Appends the suffix after the collection values.",
  },
  "Applies a function inside an Applicative context.": {
    ja: "Applicativeの操作で、包まれた関数を同じ種類の値に適用します。値がない場合や失敗の扱いは、使う型の実装に従います。",
    en: "Uses Applicative to apply a wrapped function to a value of the same kind. The selected type determines how missing values or failures are handled.",
  },
  "Compares two values with the selected total ordering.": {
    ja: "型に対応するOrdの実装で二つの値を比較します。",
    en: "Compares two values with the selected total ordering.",
  },
  "Standard function provided by the compiler-owned library surface.": {
    ja: "標準ライブラリの関数です。渡す引数と戻り値の型は、下の宣言とその読み方で確認してください。",
    en: "A standard-library function. See the declaration and its explanation below for the argument types and result type.",
  },
  "Returns the identity value for a Monoid.": {
    ja: "Monoidで定めた空の値を返します。この値と結合しても、もう一方の値は変わりません。",
    en: "Returns the empty value defined by Monoid. Combining with this value leaves the other value unchanged.",
  },
  "Tests two values with the selected Eq instance.": {
    ja: "型に対応するEqの実装で、二つの値が等しいか調べます。",
    en: "Tests two values with the selected Eq instance.",
  },
  "Transforms each element to a collection and flattens the results.": {
    ja: "各要素からコレクションを作り、その結果を一つにつなぎます。",
    en: "Transforms each element to a collection and flattens the results.",
  },
  "Computes the hash defined by the selected Hash instance.": {
    ja: "型に対応するHashの実装で、ハッシュ値を計算します。",
    en: "Computes the hash defined by the selected Hash instance.",
  },
  "Creates an Iterator over a collection.": {
    ja: "コレクションを順に読むIteratorを作ります。",
    en: "Creates an Iterator over a collection.",
  },
  "Transforms the value inside a Functor without changing its shape.": {
    ja: "Functorの操作で中の値を変換します。外側の種類や構造は保ちます。",
    en: "Uses Functor to transform the contained value while preserving the outer kind and structure.",
  },
  "Returns the numeric one for the selected type.": {
    ja: "指定した数値型の1を返します。",
    en: "Returns the numeric one for the selected type.",
  },
  "Lifts a value into an Applicative context.": {
    ja: "Applicativeの操作で値を包みます。包む型は、使う場所で必要な型に合わせて選ばれます。",
    en: "Uses Applicative to wrap a value in the type required where it is used.",
  },
  "Traverses a Functor with an Applicative effect while preserving its shape.":
    {
      ja: "各要素に対してApplicativeの処理を行います。結果は、元のコレクションの構造を保ってまとめます。",
      en: "Applies an Applicative operation to each element and combines the results while preserving the original collection structure.",
    },
  "Returns the numeric zero for the selected type.": {
    ja: "指定した数値型の0を返します。",
    en: "Returns the numeric zero for the selected type.",
  },
  "Selects an explicit numeric rounding mode.": {
    ja: "数値を丸める方法を、明示的に選ぶための値です。",
    en: "Selects an explicit numeric rounding mode.",
  },
  "Type from the shared numeric rounding surface.": {
    ja: "数値の丸め方を指定するための型です。",
    en: "A type used to choose how a numeric result is rounded.",
  },
  "Parses, formats, or computes with Int under the safe integer contract.": {
    ja: "Intの文字列への変換、文字列からの読み取り、計算を行います。Intで正確に表せる整数の範囲を検査します。",
    en: "Parses, formats, or computes Int values, checking the range of integers that Int can represent exactly.",
  },
  "Type from the checked safe integer surface.": {
    ja: "Intの値域を検査する操作に使う型です。",
    en: "A type used by operations that check whether an integer is within the Int range.",
  },
  "Parses, formats, classifies, or explicitly converts an IEEE 754 Float.": {
    ja: "IEEE 754のFloatを解析・表示・分類し、必要な型変換を明示的に行います。",
    en: "Parses, formats, classifies, or explicitly converts an IEEE 754 Float.",
  },
  "Type from the explicit Float conversion surface.": {
    ja: "Floatを他の型へ明示的に変換する操作に使う型です。",
    en: "A type used by operations that explicitly convert Float to another type.",
  },
  "Pure portable mathematics on Float; angles are radians and atan2 takes y then x.":
    {
      ja: "Floatに対する、副作用のない数学関数です。角度はラジアンで指定します。atan2にはy、xの順で引数を渡してください。",
      en: "Pure mathematical functions on Float. Angles are in radians; pass y before x to atan2.",
    },
  "Compiler-owned standard library symbol.": {
    ja: "標準ライブラリの公開項目です。値、型、関数などの種類と使い方は、下の宣言とその読み方で確認してください。",
    en: "A public standard-library item. See the declaration and its explanation below for its kind and how to use it.",
  },
  "Flattens one level of nested collections in source order.": {
    ja: "入れ子のコレクションを一段だけ平らにし、元の要素順を保ちます。",
    en: "Flattens one level of nested collections in source order.",
  },
  "Returns the elements after skipping the requested leading count.": {
    ja: "指定した個数の先頭要素を除いたコレクションを返します。",
    en: "Returns the elements after skipping the requested leading count.",
  },
  "Transforms collection elements and drops Nothing results.": {
    ja: "各要素を変換し、結果がNothingのものを除きます。",
    en: "Transforms collection elements and drops Nothing results.",
  },
  "Keeps collection elements whose predicate returns True.": {
    ja: "条件を調べる関数がTrueを返す要素だけを残します。",
    en: "Keeps collection elements whose predicate returns True.",
  },
  "Returns the first matching element, or Nothing when none matches.": {
    ja: "条件を満たす最初の要素を返します。該当する要素がなければNothingです。",
    en: "Returns the first matching element, or Nothing when none matches.",
  },
  "Returns the indexed element, or Nothing when the index is invalid.": {
    ja: "指定した添字の要素を返します。添字が範囲外ならNothingです。",
    en: "Returns the indexed element, or Nothing when the index is invalid.",
  },
  "Returns the first element, or Nothing for an empty collection.": {
    ja: "先頭の要素を返します。空ならNothingです。",
    en: "Returns the first element, or Nothing for an empty collection.",
  },
  "Returns whether the collection contains no elements.": {
    ja: "要素が一つもないかどうかを返します。",
    en: "Returns whether the collection contains no elements.",
  },
  "Returns the number of elements in the collection.": {
    ja: "要素の個数を返します。",
    en: "Returns the number of elements in the collection.",
  },
  "Returns the collection elements in reverse order.": {
    ja: "要素を逆順に並べたコレクションを返します。",
    en: "Returns the collection elements in reverse order.",
  },
  "Returns all elements after the first, or Nothing for an empty collection.": {
    ja: "先頭を除いた残りの要素を返します。空ならNothingです。",
    en: "Returns all elements after the first, or Nothing for an empty collection.",
  },
  "Returns at most the requested number of leading elements.": {
    ja: "先頭から、指定した個数までの要素を返します。",
    en: "Returns at most the requested number of leading elements.",
  },
  "Copies the Array elements into a persistent List in source order.": {
    ja: "Arrayの要素を順にコピーし、不変のListを作ります。",
    en: "Copies the Array elements into a persistent List in source order.",
  },
  "Copies the List elements into an Array in source order.": {
    ja: "Listの要素を順にコピーし、Arrayを作ります。",
    en: "Copies the List elements into an Array in source order.",
  },
  "Type or trait from the standard HTML surface.": {
    ja: "型付きHTMLの要素や属性を表すための型・traitです。",
    en: "A type or trait for representing typed HTML elements and attributes.",
  },
  "Creates a self-contained article section.": {
    ja: "一つの記事として独立したarticle要素を作ります。",
    en: "Creates a self-contained article section.",
  },
  "Creates content related to the surrounding section.": {
    ja: "周囲の内容に関連する補足を表すaside要素を作ります。",
    en: "Creates content related to the surrounding section.",
  },
  "Creates a hyperlink with typed navigation props.": {
    ja: "型付きのリンク先の指定を使い、ハイパーリンクを作ります。",
    en: "Creates a hyperlink with typed navigation props.",
  },
  "Validates a custom, data, or ARIA attribute without bypassing escaping.": {
    ja: "独自属性、data属性、ARIA属性を検証します。HTMLのエスケープ処理は省略しません。",
    en: "Validates a custom, data, or ARIA attribute without bypassing escaping.",
  },
  "Validated opaque custom HTML attribute name and value.": {
    ja: "検証済みの独自HTML属性の名前と値を表します。中身を直接変更できない型です。",
    en: "Validated opaque custom HTML attribute name and value.",
  },
  "Creates an audio media element.": {
    ja: "音声を再生するaudio要素を作ります。",
    en: "Creates an audio media element.",
  },
  "Creates a block quotation.": {
    ja: "まとまった引用を表すblockquote要素を作ります。",
    en: "Creates a block quotation.",
  },
  "Creates the document body container.": {
    ja: "文書の本文を囲むbody要素を作ります。",
    en: "Creates the document body container.",
  },
  "Creates a void line-break element without children.": {
    ja: "子要素を持たない改行用のbr要素を作ります。",
    en: "Creates a void line-break element without children.",
  },
  "Creates or renders typed HTML through the standard HTML surface.": {
    ja: "型付きHTMLを作る、または文字列として出力する操作です。",
    en: "Creates typed HTML or renders it as a string.",
  },
  "Creates a caption for a typed data table.": {
    ja: "表の見出しを表すcaption要素を作ります。",
    en: "Creates a caption for a typed data table.",
  },
  "Immutable form-control snapshot containing value and optional checked state: Just for checkbox/radio, Nothing for value controls.":
    {
      ja: "フォーム操作時の値とチェック状態を保持する不変の値です。checkboxとradioはチェック状態がJust、それ以外の値の入力はNothingです。",
      en: "Immutable form-control snapshot containing value and optional checked state: Just for checkbox/radio, Nothing for value controls.",
    },
  "Creates an inline code fragment.": {
    ja: "文中のコードを表すcode要素を作ります。",
    en: "Creates an inline code fragment.",
  },
  "Creates a custom element from a validated Tag.": {
    ja: "検証済みのTagからカスタム要素を作ります。",
    en: "Creates a custom element from a validated Tag.",
  },
  "Validates a custom-element name into an opaque Tag.": {
    ja: "カスタム要素名を検証し、Tagに変換します。",
    en: "Validates a custom-element name into an opaque Tag.",
  },
  "Creates a disclosure element with typed open state.": {
    ja: "開閉状態を指定できるdetails要素を作ります。",
    en: "Creates a disclosure element with typed open state.",
  },
  "Creates a dialog element with typed open state.": {
    ja: "開閉状態を指定できるdialog要素を作ります。",
    en: "Creates a dialog element with typed open state.",
  },
  "Prevents the browser default and stops propagation before dispatching an Action.":
    {
      ja: "ブラウザの既定動作とイベントの伝播を止めてから、Actionを送ります。",
      en: "Prevents the browser default and stops propagation before dispatching an Action.",
    },
  "Prevents the browser default before dispatching an Action.": {
    ja: "ブラウザの既定動作を止めてから、Actionを送ります。",
    en: "Prevents the browser default before dispatching an Action.",
  },
  "Dispatches an Action without controlling the browser event.": {
    ja: "ブラウザの既定動作や伝播は変えず、Actionを送ります。",
    en: "Dispatches an Action without controlling the browser event.",
  },
  "Stops event propagation before dispatching an Action.": {
    ja: "イベントの伝播を止めてから、Actionを送ります。",
    en: "Stops event propagation before dispatching an Action.",
  },
  "Creates emphasized text.": {
    ja: "強調する文言を囲むem要素を作ります。",
    en: "Creates emphasized text.",
  },
  "Explicit event outcome describing dispatch, default prevention, and propagation control.":
    {
      ja: "Actionの送信、既定動作の抑止、伝播の停止を指定するイベント処理の結果です。",
      en: "Explicit event outcome describing dispatch, default prevention, and propagation control.",
    },
  "Groups related controls in a typed form.": {
    ja: "関連するフォームの入力をfieldsetでまとめます。",
    en: "Groups related controls in a typed form.",
  },
  "Creates footer content for a page or section.": {
    ja: "ページや区画の末尾を表すfooter要素を作ります。",
    en: "Creates footer content for a page or section.",
  },
  "Creates a typed form whose onSubmit message prevents native page reload.": {
    ja: "型付きのform要素を作ります。onSubmitはブラウザの通常のページ再読み込みを防ぎます。",
    en: "Creates a typed form whose onSubmit message prevents native page reload.",
  },
  "Creates a typed document heading.": {
    ja: "文書の見出しを作ります。",
    en: "Creates a typed document heading.",
  },
  "Creates introductory content for a page or section.": {
    ja: "ページや区画の導入を表すheader要素を作ります。",
    en: "Creates introductory content for a page or section.",
  },
  "Creates the metadata container for a typed document.": {
    ja: "文書のメタ情報を囲むhead要素を作ります。",
    en: "Creates the metadata container for a typed document.",
  },
  "Creates a void thematic-break element without children.": {
    ja: "子要素を持たない、区切りを表すhr要素を作ります。",
    en: "Creates a void thematic-break element without children.",
  },
  "Describes a rejected custom HTML value or unsafe Web URL.": {
    ja: "独自のHTML値や安全でないWeb URLを拒否した理由を表します。",
    en: "Describes a rejected custom HTML value or unsafe Web URL.",
  },
  "Creates the root html element for a typed document.": {
    ja: "文書全体のルートとなるhtml要素を作ります。",
    en: "Creates the root html element for a typed document.",
  },
  "Ignores an event without dispatch or browser control.": {
    ja: "Actionを送らず、ブラウザの動作も変えずにイベントを無視します。",
    en: "Ignores an event without dispatch or browser control.",
  },
  "Creates a void image element with required source and alt text.": {
    ja: "画像のURLと代替テキストを必須とするimg要素を作ります。子要素はありません。",
    en: "Creates a void image element with required source and alt text.",
  },
  "Immutable text-input snapshot containing only the current String value.": {
    ja: "文字入力時点のStringだけを保持する不変の値です。",
    en: "Immutable text-input snapshot containing only the current String value.",
  },
  "Creates a controlled input with typed input and change event snapshots.": {
    ja: "値をアプリケーション側で管理するinput要素を作ります。入力・変更イベントの値には型が付きます。",
    en: "Creates a controlled input with typed input and change event snapshots.",
  },
  "Reports an invalid custom attribute spelling.": {
    ja: "独自属性名の書き方が不正であることを表します。",
    en: "Reports an invalid custom attribute spelling.",
  },
  "Reports a rejected custom-element name.": {
    ja: "カスタム要素名を受け付けられないことを表します。",
    en: "Reports a rejected custom-element name.",
  },
  "Immutable keyboard snapshot containing key identity, repeat state, and modifier keys.":
    {
      ja: "キー、押し続けた状態、修飾キーを保持する、キー操作時点の不変の値です。",
      en: "Immutable keyboard snapshot containing key identity, repeat state, and modifier keys.",
    },
  "Creates a label connected through the htmlFor prop.": {
    ja: "htmlForで入力と対応付けるlabel要素を作ります。",
    en: "Creates a label connected through the htmlFor prop.",
  },
  "Creates a caption for a fieldset.": {
    ja: "fieldsetの見出しとなるlegend要素を作ります。",
    en: "Creates a caption for a fieldset.",
  },
  "Creates a void external-resource link element without children.": {
    ja: "外部リソースを参照するlink要素を作ります。子要素はありません。",
    en: "Creates a void external-resource link element without children.",
  },
  "Creates a list item.": {
    ja: "リストの項目を表すli要素を作ります。",
    en: "Creates a list item.",
  },
  "Creates a void metadata element without children.": {
    ja: "メタ情報を表すmeta要素を作ります。子要素はありません。",
    en: "Creates a void metadata element without children.",
  },
  "Immutable mouse snapshot containing button, coordinates, and modifier keys.":
    {
      ja: "ボタン、座標、修飾キーを保持する、マウス操作時点の不変の値です。",
      en: "Immutable mouse snapshot containing button, coordinates, and modifier keys.",
    },
  "Creates a navigation section.": {
    ja: "ナビゲーションを表すnav要素を作ります。",
    en: "Creates a navigation section.",
  },
  "Creates an ordered list.": {
    ja: "順序付きリストのol要素を作ります。",
    en: "Creates an ordered list.",
  },
  "Creates an option for a typed selection control.": {
    ja: "選択式入力の項目を表すoption要素を作ります。",
    en: "Creates an option for a typed selection control.",
  },
  "Validates a relative or allowlisted Web URL into an opaque value.": {
    ja: "相対URL、または許可されたWeb URLを検証して専用の型に変換します。",
    en: "Validates a relative or allowlisted Web URL into an opaque value.",
  },
  "Creates a responsive image container.": {
    ja: "表示条件に応じた画像をまとめるpicture要素を作ります。",
    en: "Creates a responsive image container.",
  },
  "Immutable pointer snapshot distinguishing mouse, touch, and pen input.": {
    ja: "マウス・タッチ・ペンの種類を区別して保持する、ポインター操作時点の不変の値です。",
    en: "Immutable pointer snapshot distinguishing mouse, touch, and pen input.",
  },
  "Creates preformatted text.": {
    ja: "空白と改行を保つpre要素を作ります。",
    en: "Creates preformatted text.",
  },
  "Renders typed HTML as a complete document string.": {
    ja: "型付きHTMLを完全な文書の文字列へ変換します。",
    en: "Renders typed HTML as a complete document string.",
  },
  "Renders typed HTML to an escaped fragment string.": {
    ja: "型付きHTMLを、必要なエスケープを施した断片の文字列へ変換します。",
    en: "Renders typed HTML to an escaped fragment string.",
  },
  "Reports a custom attribute that collides with typed or runtime-owned props.":
    {
      ja: "独自属性が、型付きの属性や実行基盤が使う属性と衝突したことを表します。",
      en: "Reports a custom attribute that collides with typed or runtime-owned props.",
    },
  "Immutable scroll snapshot containing the current element offsets.": {
    ja: "要素のスクロール位置を保持する、操作時点の不変の値です。",
    en: "Immutable scroll snapshot containing the current element offsets.",
  },
  "Creates a typed selection control using the shared change snapshot.": {
    ja: "共通の変更イベントの値を使い、型付きのselect要素を作ります。",
    en: "Creates a typed selection control using the shared change snapshot.",
  },
  "Creates secondary or fine-print text.": {
    ja: "補足や注記を表すsmall要素を作ります。",
    en: "Creates secondary or fine-print text.",
  },
  "Creates a void media source element without children.": {
    ja: "メディアの読み込み元を表すsource要素を作ります。子要素はありません。",
    en: "Creates a void media source element without children.",
  },
  "Creates strongly emphasized text.": {
    ja: "重要な文言を囲むstrong要素を作ります。",
    en: "Creates strongly emphasized text.",
  },
  "Validates and converts a style record into inline Style.": {
    ja: "スタイルのレコードを検証し、インラインStyleに変換します。",
    en: "Validates and converts a style record into inline Style.",
  },
  "Creates the visible summary for a disclosure element.": {
    ja: "detailsの開閉に使う見出しsummaryを作ります。",
    en: "Creates the visible summary for a disclosure element.",
  },
  "Creates a typed data table.": {
    ja: "型付きのデータ表を作ります。",
    en: "Creates a typed data table.",
  },
  "Validated opaque name for a custom HTML element.": {
    ja: "検証済みのカスタムHTML要素名です。直接中身を変更できない型です。",
    en: "Validated opaque name for a custom HTML element.",
  },
  "Groups body rows in a typed data table.": {
    ja: "表の本体の行をtbodyでまとめます。",
    en: "Groups body rows in a typed data table.",
  },
  "Creates a data cell with typed span props.": {
    ja: "列や行にまたがる指定を持つ、表のデータセルtdを作ります。",
    en: "Creates a data cell with typed span props.",
  },
  "Creates a controlled textarea with typed input and change snapshots.": {
    ja: "値をアプリケーション側で管理するtextareaを作ります。入力・変更時の値には型が付きます。",
    en: "Creates a controlled textarea with typed input and change snapshots.",
  },
  "Groups footer rows in a typed data table.": {
    ja: "表の末尾の行をtfootでまとめます。",
    en: "Groups footer rows in a typed data table.",
  },
  "Groups header rows in a typed data table.": {
    ja: "表の先頭の行をtheadでまとめます。",
    en: "Groups header rows in a typed data table.",
  },
  "Creates a header cell with typed span props.": {
    ja: "列や行にまたがる指定を持つ、表の見出しセルthを作ります。",
    en: "Creates a header cell with typed span props.",
  },
  "Creates the document title element.": {
    ja: "文書のタイトルを表すtitle要素を作ります。",
    en: "Creates the document title element.",
  },
  "Creates a row in a typed data table.": {
    ja: "表の行trを作ります。",
    en: "Creates a row in a typed data table.",
  },
  "Creates an unordered list.": {
    ja: "順序なしリストのul要素を作ります。",
    en: "Creates an unordered list.",
  },
  "Reports a URL rejected for its scheme, credentials, or control characters.":
    {
      ja: "URLの方式、認証情報、制御文字が原因で拒否されたことを表します。",
      en: "Reports a URL rejected for its scheme, credentials, or control characters.",
    },
  "Creates a video media element.": {
    ja: "動画を再生するvideo要素を作ります。",
    en: "Creates a video media element.",
  },
  "Validated opaque URL for security-sensitive HTML attributes.": {
    ja: "安全性が必要なHTML属性に使う、検証済みのURLです。",
    en: "Validated opaque URL for security-sensitive HTML attributes.",
  },
  "Immutable wheel snapshot containing deltas, mode, coordinates, and modifiers.":
    {
      ja: "移動量、単位、座標、修飾キーを保持する、ホイール操作時点の不変の値です。",
      en: "Immutable wheel snapshot containing deltas, mode, coordinates, and modifiers.",
    },
  "Creates a pure namespaced SVG scene or explicitly bridges it to Html.": {
    ja: "SVG専用の名前空間を使う描画を純粋な値として作ります。Htmlへ組み込むときは明示的に変換します。",
    en: "Creates a pure namespaced SVG scene or explicitly bridges it to Html.",
  },
  "Type from the standard SVG scene surface.": {
    ja: "型付きSVGの図形や属性を表す型です。",
    en: "A type for representing typed SVG shapes or attributes.",
  },
  "Mounts a typed state-update-view application into a DOM target.": {
    ja: "状態・更新関数・表示関数を持つ型付きアプリケーションを、指定したDOMの場所に配置します。",
    en: "Mounts a typed state-update-view application into a DOM target.",
  },
  "Runs typed browser DOM behavior through the standard DOM surface.": {
    ja: "DOMのサービスを使い、ブラウザの要素を操作します。",
    en: "Uses a DOM service to operate on browser elements.",
  },
  "Type from the standard DOM capability surface.": {
    ja: "DOMを操作するサービスに使う型です。",
    en: "A type used by the service that operates on DOM elements.",
  },
  "Captures a live pointer on an explicit DOM target through Effect.": {
    ja: "指定したDOM要素で、操作中のポインターを捕捉するEffectです。",
    en: "Captures a live pointer on an explicit DOM target through Effect.",
  },
  "Snapshots an element bounding rectangle through the DOM capability.": {
    ja: "DOMのサービスを使い、要素を囲む長方形の位置と大きさを取得します。",
    en: "Uses the DOM service to read the position and size of the rectangle enclosing an element.",
  },
  "Observes element geometry with an explicit disposable resource lifecycle.": {
    ja: "要素の位置と大きさを監視します。監視を終了するときは、取得したリソースを解放してください。",
    en: "Observes an element's position and size. Release the returned resource when you finish observing.",
  },
  "Releases a captured pointer on an explicit DOM target through Effect.": {
    ja: "指定したDOM要素のポインター捕捉を解放するEffectです。",
    en: "Releases a captured pointer on an explicit DOM target through Effect.",
  },
  "Creates or transforms reactive values through the standard Signal surface.":
    {
      ja: "変化する現在値を表すSignalを作る、または変換します。",
      en: "Creates or transforms a Signal representing a current value that can change.",
    },
  "Creates a mutable Signal inside Effect.": {
    ja: "Effectの中で、更新できるSignalを作ります。",
    en: "Creates a mutable Signal inside Effect.",
  },
  "Derives a Signal by transforming each current value.": {
    ja: "現在値を変換して、元の値の変化に追従するSignalを作ります。",
    en: "Derives a Signal by transforming each current value.",
  },
  "Type from the standard Signal surface.": {
    ja: "変化する現在値を表すSignalの操作に使う型です。",
    en: "A type used by operations on Signal values.",
  },
  "Type from the standard Clock capability surface.": {
    ja: "時刻の取得や待機を行うClockサービスの型です。",
    en: "A type used by the Clock service to read time or wait.",
  },
  "Reads the current monotonic clock instant through Clock Effect.": {
    ja: "ClockのEffectで、後戻りしない時計の現在時点を読みます。",
    en: "Reads the current instant of a monotonic clock through a Clock Effect.",
  },
  "Waits for a Duration through the Clock Effect capability.": {
    ja: "ClockのEffectで、指定したDurationの間待ちます。",
    en: "Waits for a Duration through the Clock Effect capability.",
  },
  "Adds two Duration values with range validation.": {
    ja: "二つのDurationを加算し、結果の範囲を検証します。",
    en: "Adds two Duration values with range validation.",
  },
  "Constructs or computes with standard time values.": {
    ja: "標準の時刻や時間の長さを作る、または計算します。",
    en: "Constructs or computes with standard time values.",
  },
  "Type from the standard time surface.": {
    ja: "時刻や時間の長さを表す型です。",
    en: "A type representing an instant or a duration.",
  },
  "Converts exact hours into Duration.": {
    ja: "時間の整数値をDurationに変換します。",
    en: "Converts exact hours into Duration.",
  },
  "Converts exact milliseconds into Duration.": {
    ja: "ミリ秒の整数値をDurationに変換します。",
    en: "Converts exact milliseconds into Duration.",
  },
  "Converts exact minutes into Duration.": {
    ja: "分の整数値をDurationに変換します。",
    en: "Converts exact minutes into Duration.",
  },
  "Validates an Int as a nanosecond Duration.": {
    ja: "Intをナノ秒のDurationとして検証します。",
    en: "Validates an Int as a nanosecond Duration.",
  },
  "Converts exact seconds into Duration.": {
    ja: "秒の整数値をDurationに変換します。",
    en: "Converts exact seconds into Duration.",
  },
  "Returns a Duration's integer nanosecond count.": {
    ja: "Durationのナノ秒数をIntで返します。",
    en: "Returns a Duration's integer nanosecond count.",
  },
  "Returns the exact zero-nanosecond Duration.": {
    ja: "0ナノ秒のDurationを返します。",
    en: "Returns the exact zero-nanosecond Duration.",
  },
  "Validates and appends one HTTP header field.": {
    ja: "HTTPヘッダー一項目を検証し、末尾に追加します。",
    en: "Validates and appends one HTTP header field.",
  },
  "Performs HTTP client operations through the standard HttpClient capability.":
    {
      ja: "HttpClientサービスを使い、HTTPリクエストを送ります。",
      en: "Uses the HttpClient service to send HTTP requests.",
    },
  "Type from the standard HTTP client capability surface.": {
    ja: "HTTPクライアントのサービスに使う型です。",
    en: "A type used by the HTTP client service.",
  },
  "The standard CONNECT Method value.": {
    ja: "CONNECTメソッドを表す標準のMethod値です。",
    en: "The standard CONNECT Method value.",
  },
  "Validates an uppercase custom HTTP method token.": {
    ja: "大文字の独自HTTPメソッド名を検証します。",
    en: "Validates an uppercase custom HTTP method token.",
  },
  "The standard DELETE Method value.": {
    ja: "DELETEメソッドを表す標準のMethod値です。",
    en: "The standard DELETE Method value.",
  },
  "The empty immutable Headers value.": {
    ja: "空の不変のHeaders値です。",
    en: "The empty immutable Headers value.",
  },
  "Reads the message carried by an HTTP client failure.": {
    ja: "HTTPクライアントの失敗が持つメッセージを読みます。",
    en: "Reads the message carried by an HTTP client failure.",
  },
  "The standard GET Method value.": {
    ja: "GETメソッドを表す標準のMethod値です。",
    en: "The standard GET Method value.",
  },
  "Immutable ordered HTTP header collection.": {
    ja: "順序を保つ不変のHTTPヘッダーの集まりです。",
    en: "Immutable ordered HTTP header collection.",
  },
  "The standard HEAD Method value.": {
    ja: "HEADメソッドを表す標準のMethod値です。",
    en: "The standard HEAD Method value.",
  },
  "Positive response body byte limit.": {
    ja: "レスポンス本文の最大バイト数を表す正の値です。",
    en: "Positive response body byte limit.",
  },
  "Typed validation failure while building an HTTP value.": {
    ja: "HTTPの値を作る際の検証の失敗を表します。",
    en: "Typed validation failure while building an HTTP value.",
  },
  "Typed HTTP transport, protocol, or body failure.": {
    ja: "HTTPの通信、プロトコル、本文の読み取りの失敗を表します。",
    en: "Typed HTTP transport, protocol, or body failure.",
  },
  "Normalized absolute HTTP or HTTPS URL.": {
    ja: "正規化済みの絶対HTTPまたはHTTPS URLです。",
    en: "Normalized absolute HTTP or HTTPS URL.",
  },
  "Opaque validated HTTP request method.": {
    ja: "検証済みのHTTPリクエストメソッドです。",
    en: "Opaque validated HTTP request method.",
  },
  "Reads the text of a validated HTTP Method.": {
    ja: "検証済みのMethodの文字列を読みます。",
    en: "Reads the text of a validated HTTP Method.",
  },
  "The standard OPTIONS Method value.": {
    ja: "OPTIONSメソッドを表す標準のMethod値です。",
    en: "The standard OPTIONS Method value.",
  },
  "Parses and normalizes an absolute HTTP or HTTPS URL.": {
    ja: "絶対HTTPまたはHTTPS URLを解析し、正規化します。",
    en: "Parses and normalizes an absolute HTTP or HTTPS URL.",
  },
  "The standard PATCH Method value.": {
    ja: "PATCHメソッドを表す標準のMethod値です。",
    en: "The standard PATCH Method value.",
  },
  "The standard POST Method value.": {
    ja: "POSTメソッドを表す標準のMethod値です。",
    en: "The standard POST Method value.",
  },
  "The standard PUT Method value.": {
    ja: "PUTメソッドを表す標準のMethod値です。",
    en: "The standard PUT Method value.",
  },
  "Removes every case-insensitive occurrence of a header.": {
    ja: "大文字・小文字を区別せず、指定したヘッダーをすべて削除します。",
    en: "Removes every case-insensitive occurrence of a header.",
  },
  "Renders a normalized HttpUrl.": {
    ja: "正規化済みのHttpUrlを文字列にします。",
    en: "Renders a normalized HttpUrl.",
  },
  "Builds an immutable request head from Method and HttpUrl.": {
    ja: "MethodとHttpUrlから、不変のリクエストの先頭部分を作ります。",
    en: "Builds an immutable request head from Method and HttpUrl.",
  },
  "Immutable HTTP request head without a body.": {
    ja: "本文を含まない、不変のHTTPリクエストの先頭部分です。",
    en: "Immutable HTTP request head without a body.",
  },
  "Copies the immutable Bytes body of a small response.": {
    ja: "小さいレスポンスの、不変のBytes本文をコピーして返します。",
    en: "Copies the immutable Bytes body of a small response.",
  },
  "Copies the immutable headers of a small response.": {
    ja: "小さいレスポンスの、不変のヘッダーをコピーして返します。",
    en: "Copies the immutable headers of a small response.",
  },
  "Reads the validated status of a small response.": {
    ja: "小さいレスポンスの、検証済みステータスを読みます。",
    en: "Reads the validated status of a small response.",
  },
  "Small HTTP response with an immutable Bytes body.": {
    ja: "不変のBytes本文を持つ、小さいHTTPレスポンスです。",
    en: "Small HTTP response with an immutable Bytes body.",
  },
  "Sends an explicit Bytes body through HttpClient.": {
    ja: "HttpClientを使い、指定したBytesを本文として送ります。",
    en: "Sends an explicit Bytes body through HttpClient.",
  },
  "Sends an explicit empty body through HttpClient.": {
    ja: "HttpClientを使い、空の本文を明示して送ります。",
    en: "Sends an explicit empty body through HttpClient.",
  },
  "Replaces a header field at its first ordered position.": {
    ja: "ヘッダーを、元の最初の位置で置き換えます。",
    en: "Replaces a header field at its first ordered position.",
  },
  "Reads the Int code of a validated HTTP Status.": {
    ja: "検証済みStatusの数値をIntで読みます。",
    en: "Reads the Int code of a validated HTTP Status.",
  },
  "Validates an HTTP status code from 100 through 999.": {
    ja: "100から999のHTTPステータスコードを検証します。",
    en: "Validates an HTTP status code from 100 through 999.",
  },
  "Opaque validated HTTP response status.": {
    ja: "検証済みのHTTPレスポンスのステータスです。",
    en: "Opaque validated HTTP response status.",
  },
  "The standard TRACE Method value.": {
    ja: "TRACEメソッドを表す標準のMethod値です。",
    en: "The standard TRACE Method value.",
  },
  "Returns the exact non-negative magnitude of a BigInt.": {
    ja: "BigIntの符号を除いた、正確な絶対値を返します。",
    en: "Returns the exact non-negative magnitude of a BigInt.",
  },
  "Describes a BigInt that cannot be narrowed to Int exactly.": {
    ja: "BigIntを正確にIntへ変換できなかった理由を表します。",
    en: "Describes a BigInt that cannot be narrowed to Int exactly.",
  },
  "Reports a zero BigInt divisor.": {
    ja: "BigIntの除数が0であることを表します。",
    en: "Reports a zero BigInt divisor.",
  },
  "Describes checked BigInt division or remainder by zero.": {
    ja: "BigIntの除算や余りの計算で、除数が0の場合の失敗を表します。",
    en: "Describes checked BigInt division or remainder by zero.",
  },
  "Reports a BigInt outside the safe Int range.": {
    ja: "BigIntがIntで表せる範囲外であることを表します。",
    en: "Reports a BigInt outside the safe Int range.",
  },
  "Describes an invalid BigInt text or radix without a magnitude limit.": {
    ja: "BigIntの文字列や基数が不正であることを表します。値の大きさには上限を設けません。",
    en: "Describes an invalid BigInt text or radix without a magnitude limit.",
  },
  "Describes a checked negative BigInt exponent.": {
    ja: "BigIntの指数に負の整数を指定した失敗を表します。",
    en: "Describes a checked negative BigInt exponent.",
  },
  "Opaque arbitrary-precision signed integer value.": {
    ja: "任意精度の符号付き整数です。",
    en: "Opaque arbitrary-precision signed integer value.",
  },
  "Divides exactly represented BigInts or returns a typed zero-divisor failure.":
    {
      ja: "BigInt同士を除算します。除数が0なら回復可能な失敗を返します。",
      en: "Divides exactly represented BigInts or returns a typed zero-divisor failure.",
    },
  "Raises a BigInt by a non-negative Int using exact exponentiation.": {
    ja: "BigIntを、0以上のIntで指定した回数だけ正確に累乗します。",
    en: "Raises a BigInt by a non-negative Int using exact exponentiation.",
  },
  "Computes truncating BigInt remainder or returns a typed zero-divisor failure.":
    {
      ja: "BigInt同士の余りを計算します。除数が0なら回復可能な失敗を返します。商は0に向けて切り捨てます。",
      en: "Computes truncating BigInt remainder or returns a typed zero-divisor failure.",
    },
  "Reports an empty BigInt input.": {
    ja: "BigIntの入力文字列が空であることを表します。",
    en: "Reports an empty BigInt input.",
  },
  "Formats a BigInt in radix 2 through 36.": {
    ja: "BigIntを、2から36で指定した基数の文字列にします。",
    en: "Formats a BigInt in radix 2 through 36.",
  },
  "Formats a BigInt as canonical decimal text.": {
    ja: "BigIntを、標準の十進文字列にします。",
    en: "Formats a BigInt as canonical decimal text.",
  },
  "Converts Int to BigInt exactly.": {
    ja: "Intを正確にBigIntへ変換します。",
    en: "Converts Int to BigInt exactly.",
  },
  "Reports the first invalid BigInt digit at a UTF-8 byte offset.": {
    ja: "BigInt文字列の最初の不正な数字の位置を、UTF-8のバイト位置で表します。",
    en: "Reports the first invalid BigInt digit at a UTF-8 byte offset.",
  },
  "Reports a BigInt radix outside the inclusive range 2 through 36.": {
    ja: "BigIntの基数が2から36の範囲外であることを表します。",
    en: "Reports a BigInt radix outside the inclusive range 2 through 36.",
  },
  "Reports a negative Int exponent.": {
    ja: "Intの指数が負であることを表します。",
    en: "Reports a negative Int exponent.",
  },
  "Parses signed BigInt text in radix 2 through 36.": {
    ja: "2から36で指定した基数で、符号付きBigIntの文字列を解析します。",
    en: "Parses signed BigInt text in radix 2 through 36.",
  },
  "Parses canonical decimal text into an exact BigInt.": {
    ja: "十進文字列を、正確なBigIntへ解析します。",
    en: "Parses canonical decimal text into an exact BigInt.",
  },
  "Returns minus one, zero, or one for a BigInt.": {
    ja: "BigIntの符号を-1、0、1のいずれかで返します。",
    en: "Returns minus one, zero, or one for a BigInt.",
  },
  "Narrows BigInt to Int with a typed range failure.": {
    ja: "BigIntをIntへ変換します。範囲外なら回復可能な失敗を返します。",
    en: "Narrows BigInt to Int with a typed range failure.",
  },
  "Describes an Int that cannot be represented as a Byte.": {
    ja: "IntがByteで表せない値であることを表します。",
    en: "Describes an Int that cannot be represented as a Byte.",
  },
  "Describes an invalid half-open Bytes slice range.": {
    ja: "Bytesの切り出し範囲が不正であることを表します。範囲は開始位置を含み、終了位置を含みません。",
    en: "Describes an invalid half-open Bytes slice range.",
  },
  "Immutable sequence of bytes with explicit copy boundaries.": {
    ja: "不変のバイト列です。データをコピーする操作かどうかは、各関数の説明で確認できます。",
    en: "An immutable byte sequence. Each operation specifies whether it copies the data.",
  },
  "Validates an Int and converts it to an opaque Byte.": {
    ja: "Intを検証してByteへ変換します。",
    en: "Validates an Int and converts it to an opaque Byte.",
  },
  "Opaque unsigned 8-bit value in the inclusive range 0 through 255.": {
    ja: "0から255の符号なし8ビット値です。",
    en: "Opaque unsigned 8-bit value in the inclusive range 0 through 255.",
  },
  "Creates an independent copy of Bytes.": {
    ja: "元の値と独立したBytesのコピーを作ります。",
    en: "Creates an independent copy of Bytes.",
  },
  "Creates an empty immutable Bytes value.": {
    ja: "空の不変のBytesを作ります。",
    en: "Creates an empty immutable Bytes value.",
  },
  "Copies an Array of Byte values into immutable Bytes.": {
    ja: "ByteのArrayをコピーして、不変のBytesを作ります。",
    en: "Copies an Array of Byte values into immutable Bytes.",
  },
  "Validates Int values in order and copies them into Bytes.": {
    ja: "Intを順に検証し、コピーしてBytesを作ります。",
    en: "Validates Int values in order and copies them into Bytes.",
  },
  "Creates Bytes containing exactly one Byte.": {
    ja: "Byte一個だけを含むBytesを作ります。",
    en: "Creates Bytes containing exactly one Byte.",
  },
  "Returns a validated half-open slice without exposing mutation.": {
    ja: "開始を含み終了を含まない範囲を検証し、切り出したBytesを返します。変更可能な内部データは公開しません。",
    en: "Returns a validated half-open slice without exposing mutation.",
  },
  "Copies Bytes into an Array of Byte values.": {
    ja: "BytesをコピーしてByteのArrayを作ります。",
    en: "Copies Bytes into an Array of Byte values.",
  },
  "Copies Bytes into an Array of Int values.": {
    ja: "BytesをコピーしてIntのArrayを作ります。",
    en: "Copies Bytes into an Array of Int values.",
  },
  "Converts a Byte to its Int value.": {
    ja: "Byteの値をIntへ変換します。",
    en: "Converts a Byte to its Int value.",
  },
  "Describes a strict RFC 4648 Base64 decoding failure.": {
    ja: "RFC 4648のBase64を厳密に復号した際の失敗を表します。",
    en: "Describes a strict RFC 4648 Base64 decoding failure.",
  },
  "Strictly decodes canonical padded RFC 4648 Base64 text.": {
    ja: "RFC 4648の、末尾の埋め合わせを持つ標準のBase64文字列を厳密に復号します。",
    en: "Strictly decodes canonical padded RFC 4648 Base64 text.",
  },
  "Strictly decodes canonical unpadded URL-safe Base64 text.": {
    ja: "末尾の埋め合わせを持たない、URL向けBase64文字列を厳密に復号します。",
    en: "Strictly decodes canonical unpadded URL-safe Base64 text.",
  },
  "Encodes Bytes as canonical padded RFC 4648 Base64 text.": {
    ja: "Bytesを、RFC 4648の末尾の埋め合わせを持つBase64文字列にします。",
    en: "Encodes Bytes as canonical padded RFC 4648 Base64 text.",
  },
  "Encodes Bytes as canonical unpadded URL-safe Base64 text.": {
    ja: "Bytesを、末尾の埋め合わせを持たないURL向けBase64文字列にします。",
    en: "Encodes Bytes as canonical unpadded URL-safe Base64 text.",
  },
  "Reports the UTF-8 byte offset of the first invalid Base64 digit.": {
    ja: "最初の不正なBase64文字の位置を、UTF-8のバイト位置で表します。",
    en: "Reports the UTF-8 byte offset of the first invalid Base64 digit.",
  },
  "Reports an invalid Base64 input length measured in UTF-8 bytes.": {
    ja: "Base64入力の長さが不正であることを表します。長さの単位はUTF-8バイトです。",
    en: "Reports an invalid Base64 input length measured in UTF-8 bytes.",
  },
  "Reports the UTF-8 byte offset of invalid Base64 padding.": {
    ja: "Base64の埋め合わせが不正な位置を、UTF-8のバイト位置で表します。",
    en: "Reports the UTF-8 byte offset of invalid Base64 padding.",
  },
  "Reports non-zero unused bits in the final Base64 sextet.": {
    ja: "Base64の最後の6ビットのうち、使われていない部分が0でないことを表します。",
    en: "Reports non-zero unused bits in the final Base64 sextet.",
  },
  "Decodes ASCII hexadecimal text or reports the first typed failure.": {
    ja: "ASCIIの十六進文字列を復号します。不正な場合は最初の失敗を返します。",
    en: "Decodes ASCII hexadecimal text or reports the first typed failure.",
  },
  "Encodes Bytes as canonical lowercase hexadecimal text.": {
    ja: "Bytesを、標準の小文字の十六進文字列にします。",
    en: "Encodes Bytes as canonical lowercase hexadecimal text.",
  },
  "Describes an invalid hexadecimal text input with a UTF-8 byte position.": {
    ja: "十六進文字列の不正な入力を、UTF-8のバイト位置とともに表します。",
    en: "Describes an invalid hexadecimal text input with a UTF-8 byte position.",
  },
  "Reports the UTF-8 byte offset of the first non-hexadecimal digit.": {
    ja: "最初の十六進数字でない文字の位置を、UTF-8のバイト位置で表します。",
    en: "Reports the UTF-8 byte offset of the first non-hexadecimal digit.",
  },
  "Reports an odd hexadecimal input length measured in UTF-8 bytes.": {
    ja: "十六進文字列の長さが奇数であることを表します。長さの単位はUTF-8バイトです。",
    en: "Reports an odd hexadecimal input length measured in UTF-8 bytes.",
  },
  "Stops reduction immediately with the final result.": {
    ja: "最終結果を返し、要素をまとめる処理を直ちに止めます。",
    en: "Stops reduction immediately with the final result.",
  },
  "Continues reduction with the new accumulator.": {
    ja: "新しい蓄積値を使い、要素をまとめる処理を続けます。",
    en: "Continues reduction with the new accumulator.",
  },
  "Pure reduction control: Next continues and Done stops.": {
    ja: "集計を続けるか終了するかを表す値です。Nextは次の要素へ進み、Doneは結果を返して終了します。",
    en: "A value controlling whether reduction continues. Next proceeds to the next element; Done returns the result and stops.",
  },
  "Folds an Iterable in source order until Done; does not pull the remaining elements.":
    {
      ja: "Iterableを順に処理し、Doneで止めます。終了後の要素は取り出しません。",
      en: "Folds an Iterable in source order until Done; does not pull the remaining elements.",
    },
  "Validates a positive precision and constructs a DecimalContext.": {
    ja: "精度が正であることを検証し、DecimalContextを作ります。",
    en: "Validates a positive precision and constructs a DecimalContext.",
  },
  "Describes a checked decimal division failure.": {
    ja: "Decimalの除算に失敗した理由を表します。",
    en: "Describes why Decimal division failed.",
  },
  "Describes an invalid explicit decimal arithmetic context.": {
    ja: "Decimalの計算方法を指定する値が不正であることを表します。",
    en: "Describes an invalid explicit decimal arithmetic context.",
  },
  "Explicit positive precision and rounding policy for decimal operations.": {
    ja: "Decimalの計算で使う、有効桁数と丸め方の明示的な指定です。有効桁数は正です。",
    en: "Explicit positive precision and rounding policy for decimal operations.",
  },
  "Describes a failed explicit Int or Float conversion.": {
    ja: "IntやFloatへの明示的な変換の失敗を表します。",
    en: "Describes a failed explicit Int or Float conversion.",
  },
  "Reports a zero Decimal divisor.": {
    ja: "Decimalの除数が0であることを表します。",
    en: "Reports a zero Decimal divisor.",
  },
  "Reports a Decimal with a nonzero fractional part.": {
    ja: "Decimalに0でない小数部分があり、整数へ変換できないことを表します。",
    en: "Reports a Decimal with a nonzero fractional part.",
  },
  "Reports a Decimal outside the finite binary64 range.": {
    ja: "Decimalが有限の64ビットFloatの範囲外であることを表します。",
    en: "Reports a Decimal outside the finite binary64 range.",
  },
  "Reports an integral Decimal outside the safe Int range.": {
    ja: "整数であるDecimalが、Intで表せる範囲外であることを表します。",
    en: "Reports an integral Decimal outside the safe Int range.",
  },
  "Describes invalid decimal text at a UTF-8 byte offset.": {
    ja: "Decimal文字列が不正な位置を、UTF-8のバイト位置で表します。",
    en: "Describes invalid decimal text at a UTF-8 byte offset.",
  },
  "Opaque finite arbitrary-precision decimal value.": {
    ja: "有限の任意精度の十進数です。",
    en: "Opaque finite arbitrary-precision decimal value.",
  },
  "Divides Decimals only when the exact result has a finite decimal expansion.":
    {
      ja: "正確な結果が有限桁の十進数になる場合にだけ、Decimal同士を除算します。",
      en: "Divides Decimals only when the exact result has a finite decimal expansion.",
    },
  "Divides Decimals using explicit significant-digit precision and rounding.": {
    ja: "有効桁数と丸め方を指定して、Decimal同士を除算します。",
    en: "Divides Decimals using explicit significant-digit precision and rounding.",
  },
  "Reports NaN or infinity at the explicit Float conversion boundary.": {
    ja: "Floatからの変換にNaNまたは無限大が渡されたことを表します。",
    en: "Reports NaN or infinity at the explicit Float conversion boundary.",
  },
  "Converts the exact finite binary64 value under an explicit DecimalContext.":
    {
      ja: "DecimalContextを指定し、有限の64ビットFloatの正確な値を変換します。",
      en: "Converts the exact finite binary64 value under an explicit DecimalContext.",
    },
  "Converts Int to Decimal exactly.": {
    ja: "Intを正確にDecimalへ変換します。",
    en: "Converts Int to Decimal exactly.",
  },
  "Reports the first byte where decimal parsing becomes invalid.": {
    ja: "Decimalの解析で最初に不正となるバイト位置を表します。",
    en: "Reports the first byte where decimal parsing becomes invalid.",
  },
  "Reports a decimal precision that is zero or negative.": {
    ja: "Decimalの有効桁数が0または負であることを表します。",
    en: "Reports a decimal precision that is zero or negative.",
  },
  "Reports an exact quotient with a non-terminating decimal expansion.": {
    ja: "正確な商が、有限桁の十進数にならないことを表します。",
    en: "Reports an exact quotient with a non-terminating decimal expansion.",
  },
  "Parses exact Decimal text without binary floating point.": {
    ja: "二進の浮動小数点数を経由せず、十進文字列を正確なDecimalへ解析します。",
    en: "Parses exact Decimal text without binary floating point.",
  },
  "Returns the significant-digit precision of a context.": {
    ja: "DecimalContextの有効桁数を返します。",
    en: "Returns the significant-digit precision of a context.",
  },
  "Rounds a Decimal to an explicit decimal scale and rounding mode.": {
    ja: "小数点以下の桁数と丸め方を指定し、Decimalを丸めます。",
    en: "Rounds a Decimal to an explicit decimal scale and rounding mode.",
  },
  "Returns the rounding mode of a context.": {
    ja: "DecimalContextの丸め方を返します。",
    en: "Returns the rounding mode of a context.",
  },
  "Rounds Decimal explicitly to the nearest finite binary64 value.": {
    ja: "Decimalを明示的に丸め、最も近い有限の64ビットFloatへ変換します。",
    en: "Rounds Decimal explicitly to the nearest finite binary64 value.",
  },
  "Converts an integral in-range Decimal to Int without rounding.": {
    ja: "範囲内の整数であるDecimalを、丸めずにIntへ変換します。",
    en: "Converts an integral in-range Decimal to Int without rounding.",
  },
  "Builds or transforms a cold standard Effect value.": {
    ja: "実行するまで処理が始まらないEffectを作る、または変換します。",
    en: "Builds or transforms a cold standard Effect value.",
  },
  "Moves typed failure into Either while preserving defects and cancellation.":
    {
      ja: "回復可能な失敗をEitherのLeftとして返します。処理の不具合やキャンセルは捕捉しません。",
      en: "Returns a recoverable failure as Left in Either. Defects and cancellation are not caught.",
    },
  "Defers construction of a cold Effect until each execution.": {
    ja: "Effectを実行するたびに、その処理を作る関数を呼び出します。",
    en: "Defers construction of a cold Effect until each execution.",
  },
  "Creates an Effect that fails with a typed error.": {
    ja: "指定した型のエラーで失敗するEffectを作ります。",
    en: "Creates an Effect that fails with a typed error.",
  },
  "Type from the standard Effect control surface.": {
    ja: "Effectの実行や継続を制御する型です。",
    en: "A type used to control Effect execution or continuation.",
  },
  "Runs cold actions sequentially in Iterable order, stopping successfully at Break without pulling the next element.":
    {
      ja: "Iterableの順にEffectを実行します。Breakなら正常に終了し、次の要素は取り出しません。",
      en: "Runs cold actions sequentially in Iterable order, stopping successfully at Break without pulling the next element.",
    },
  "Lifts Either into Effect without performing an external operation.": {
    ja: "EitherをEffectへ変換します。この操作自体は外部への入出力を行いません。",
    en: "Lifts Either into Effect without performing an external operation.",
  },
  "Lifts Maybe into Effect with an explicit missing-value error.": {
    ja: "MaybeをEffectへ変換します。値がない場合に使うエラーを明示的に渡します。",
    en: "Lifts Maybe into Effect with an explicit missing-value error.",
  },
  "Normal-success control for sequential Effect traversal: Continue or Break.":
    {
      ja: "Effectで要素を順に処理する際、継続するか正常に終了するかを表します。Continueは次へ進み、Breakは終了します。",
      en: "Controls whether sequential Effect processing continues or finishes successfully. Continue proceeds; Break stops.",
    },
  "Transforms an Effect failure while preserving its success value.": {
    ja: "Effectの失敗値を変換します。成功値は変えません。",
    en: "Transforms an Effect failure while preserving its success value.",
  },
  "Projects an outer environment into the environment required by an Effect.": {
    ja: "外側の環境から、そのEffectに必要な環境を取り出す関数を指定します。",
    en: "Projects an outer environment into the environment required by an Effect.",
  },
  "Runs an Effect with a fixed environment.": {
    ja: "固定した環境を渡してEffectを実行します。",
    en: "Runs an Effect with a fixed environment.",
  },
  "Recovers a typed Effect failure without catching defects or cancellation.": {
    ja: "回復可能な失敗が起きたら、指定した別の処理を実行します。処理の不具合やキャンセルは捕捉しません。",
    en: "Runs the supplied recovery computation after a recoverable failure. Defects and cancellation are not caught.",
  },
  "Builds a zero-delay Schedule with a bounded number of reruns.": {
    ja: "指定した回数まで、待たずに再実行するScheduleを作ります。",
    en: "Builds a zero-delay Schedule with a bounded number of reruns.",
  },
  "Repeats successes according to a Schedule and Clock.": {
    ja: "ScheduleとClockに従い、成功した処理を繰り返します。",
    en: "Repeats successes according to a Schedule and Clock.",
  },
  "Retries typed failures according to a Schedule and Clock.": {
    ja: "Scheduleで定めた回数や待ち時間に従い、回復可能な失敗が起きた処理を再試行します。待機にはClockを使います。",
    en: "Retries recoverable failures according to the Schedule, using Clock for any waits.",
  },
  "Selects one value from the execution environment.": {
    ja: "実行環境から一つの値を取り出します。",
    en: "Selects one value from the execution environment.",
  },
  "Builds a fixed-delay Schedule with bounded reruns.": {
    ja: "一定の待ち時間を挟み、指定した回数まで再実行するScheduleを作ります。",
    en: "Builds a fixed-delay Schedule with bounded reruns.",
  },
  "Creates an Effect that succeeds with a value.": {
    ja: "指定した値で成功するEffectを作ります。",
    en: "Creates an Effect that succeeds with a value.",
  },
  "Applies a Clock-backed timeout with an explicit typed failure on expiry.": {
    ja: "Clockを使って制限時間を設けます。時間切れでは、引数で渡したエラーを返します。",
    en: "Uses Clock to set a time limit. On expiry, fails with the error supplied as an argument.",
  },
  "Applies a Clock-backed timeout and returns Maybe on expiry.": {
    ja: "Clockを使って制限時間を設け、時間切れではNothingを返します。",
    en: "Applies a Clock-backed timeout and returns Maybe on expiry.",
  },
  "Builds a Schedule that continues while a predicate is true.": {
    ja: "条件がTrueの間、再実行を続けるScheduleを作ります。",
    en: "Builds a Schedule that continues while a predicate is true.",
  },
  "Transforms the selected Left or Right branch once.": {
    ja: "選ばれたLeftまたはRightの中身を一度だけ変換します。",
    en: "Transforms the selected Left or Right branch once.",
  },
  "Eliminates Either by calling only the selected branch function.": {
    ja: "LeftかRightに対応する関数だけを呼び出し、Eitherの中身を結果に変換します。",
    en: "Eliminates Either by calling only the selected branch function.",
  },
  "Transforms only the Left payload.": {
    ja: "Leftの中身だけを変換します。",
    en: "Transforms only the Left payload.",
  },
  "Transforms only the Right payload using the standard Functor instance.": {
    ja: "標準Functorの実装を使い、Rightの中身だけを変換します。",
    en: "Transforms only the Right payload using the standard Functor instance.",
  },
  "Traverses wrapped values with identity, preserving source shape and order.":
    {
      ja: "包まれた各値から結果を取り出し、元のコレクションの構造と順序を保ってまとめます。",
      en: "Collects the results from wrapped values while preserving the source collection's shape and order.",
    },
  "Exchanges Left and Right without changing the payload.": {
    ja: "中身を変えず、LeftとRightを入れ替えます。",
    en: "Exchanges Left and Right without changing the payload.",
  },
  "Uses the source Traversable and the module's Applicative without a separate traversal implementation.":
    {
      ja: "コレクションを順にたどるTraversableと、このモジュールのApplicativeを使って、各要素の処理結果をまとめます。",
      en: "Uses the collection's Traversable and this module's Applicative to combine the results of processing its elements.",
    },
  "Decodes every JSON array element in source order.": {
    ja: "JSON配列の全要素を順に読み取ります。",
    en: "Decodes every JSON array element in source order.",
  },
  "Reports the root-to-leaf path and kind of a decode failure.": {
    ja: "値の読み取りに失敗した理由と、対象のフィールドや配列要素までの位置を返します。",
    en: "Reports the kind of decoding failure and the path to the affected field or array element.",
  },
  "Pure Json decoder that reports a typed path-aware DecodeError.": {
    ja: "Jsonを値に変換する純粋な関数です。失敗では位置を持つDecodeErrorを返します。",
    en: "Pure Json decoder that reports a typed path-aware DecodeError.",
  },
  "Parses and decodes text through its selected JsonDecode dictionary.": {
    ja: "文字列を解析し、その型のJsonDecodeの実装で値に変換します。",
    en: "Parses and decodes text through its selected JsonDecode dictionary.",
  },
  "Pure function that encodes a value as Json.": {
    ja: "値をJsonへ変換する純粋な関数です。",
    en: "Pure function that encodes a value as Json.",
  },
  "Encodes a value through its selected JsonEncode dictionary.": {
    ja: "その型のJsonEncodeの実装で、値をJsonへ変換します。",
    en: "Encodes a value through its selected JsonEncode dictionary.",
  },
  "Decodes a required object field and prepends its path segment on failure.": {
    ja: "必須のオブジェクトフィールドを読み取ります。失敗には、そのフィールドの位置を加えます。",
    en: "Decodes a required object field and prepends its path segment on failure.",
  },
  "Selects a following decoder from a successful decoder result.": {
    ja: "読み取りに成功した値から、次に使う読み取り関数を選びます。",
    en: "Selects a following decoder from a successful decoder result.",
  },
  "Decodes one array position and prepends its index on failure.": {
    ja: "配列の一つの位置を読み取ります。失敗には、その添字を加えます。",
    en: "Decodes one array position and prepends its index on failure.",
  },
  "Reports invalid syntax or a duplicate object field.": {
    ja: "構文が不正、またはオブジェクトのフィールド名が重複していることを表します。",
    en: "Reports invalid syntax or a duplicate object field.",
  },
  "Distinguishes JSON syntax failure from value decode failure.": {
    ja: "JSON構文の失敗と、値への変換の失敗を区別します。",
    en: "Distinguishes JSON syntax failure from value decode failure.",
  },
  "Exact JSON value with Decimal numbers and ordered object fields.": {
    ja: "数値には正確なDecimalを使い、オブジェクトのフィールド順を保つJSONの値です。",
    en: "Exact JSON value with Decimal numbers and ordered object fields.",
  },
  "Transforms a successful decoder result.": {
    ja: "読み取りに成功した値を変換します。",
    en: "Transforms a successful decoder result.",
  },
  "Tries decoders in order and returns the first successful result.": {
    ja: "読み取り関数を順に試し、最初に成功した結果を返します。",
    en: "Tries decoders in order and returns the first successful result.",
  },
  "Decodes an object field as Maybe and accepts only a missing field as Nothing.":
    {
      ja: "フィールドをMaybeとして読みます。フィールドがない場合だけNothingとし、不正な値は成功扱いにしません。",
      en: "Decodes an object field as Maybe and accepts only a missing field as Nothing.",
    },
  "Parses RFC 8259 JSON with exact Decimal numbers and duplicate-field rejection.":
    {
      ja: "RFC 8259のJSONを解析します。数値は正確なDecimalで扱い、フィールド名の重複を拒否します。",
      en: "Parses RFC 8259 JSON with exact Decimal numbers and duplicate-field rejection.",
    },
  "Decodes a declared homogeneous object field set and rejects unknown fields.":
    {
      ja: "指定した同じ型のフィールドの集合を読み取ります。不明なフィールドを拒否します。",
      en: "Decodes a declared homogeneous object field set and rejects unknown fields.",
    },
  "Writes compact deterministic JSON with minimal escaping.": {
    ja: "必要最小限のエスケープを施し、一定の形式の短いJSON文字列を作ります。",
    en: "Writes compact deterministic JSON with minimal escaping.",
  },
  "Selects the fallback Maybe for Nothing, preserving an existing Just.": {
    ja: "Nothingなら代わりのMaybeを返し、Justなら元の値を保ちます。",
    en: "Selects the fallback Maybe for Nothing, preserving an existing Just.",
  },
  "Selects the fallback for Nothing, or the Just payload. Arguments are strict.":
    {
      ja: "Nothingなら代わりの値、Justならその中身を返します。関数なので、引数は呼び出し前に計算されます。",
      en: "Selects the fallback for Nothing, or the Just payload. Arguments are strict.",
    },
  "Creates a fresh Effect-local mutable Ref.": {
    ja: "Effectの実行ごとに新しいRefを作ります。Refの中身は明示的な操作で更新できます。",
    en: "Creates a fresh Effect-local mutable Ref.",
  },
  "Returns a result and updates a Ref in one atomic callback.": {
    ja: "一つの関数で結果と新しい値を求め、Refの値の更新と結果の取得をまとめて行います。",
    en: "Returns a result and updates a Ref in one atomic callback.",
  },
  "Type from the standard mutable Ref surface.": {
    ja: "明示的に読み書きできるRefの操作に使う型です。",
    en: "A type used by explicit read and update operations on Ref.",
  },
  "Replaces the current Ref value atomically.": {
    ja: "Refの現在値を、一つの操作で置き換えます。",
    en: "Replaces the current Ref value atomically.",
  },
  "Updates a Ref with one pure atomic callback.": {
    ja: "純粋な関数を一度呼び、その結果でRefを更新します。読み取りと更新は一つの操作です。",
    en: "Updates a Ref with one pure atomic callback.",
  },
  "Compiles a portable pattern or returns a typed error without throwing.": {
    ja: "移植可能な正規表現をコンパイルします。不正なら例外を投げず、型付きのエラーを返します。",
    en: "Compiles a portable pattern or returns a typed error without throwing.",
  },
  "Compiles a portable pattern using explicit RegexOptions.": {
    ja: "RegexOptionsを明示して、移植可能な正規表現をコンパイルします。",
    en: "Compiles a portable pattern using explicit RegexOptions.",
  },
  "Returns the portable Regex defaults with every option disabled.": {
    ja: "すべての選択肢が無効の、標準RegexOptionsを返します。",
    en: "Returns the portable Regex defaults with every option disabled.",
  },
  "Reports a repeated named-capture identifier.": {
    ja: "同じ名前のキャプチャーが繰り返し指定されていることを表します。",
    en: "Reports a repeated named-capture identifier.",
  },
  "Escapes text so it denotes a literal portable Regex fragment.": {
    ja: "文字列を、正規表現の記号ではなく文字そのものとして扱えるようにエスケープします。",
    en: "Escapes text so it denotes a literal portable Regex fragment.",
  },
  "Returns non-overlapping matches and advances one scalar after an empty match.":
    {
      ja: "重複しない一致をすべて返します。空の一致の後はUnicodeスカラー値一個分進みます。",
      en: "Returns non-overlapping matches and advances one scalar after an empty match.",
    },
  "Reports an invalid or incomplete regular-expression escape.": {
    ja: "正規表現のエスケープが不正、または未完成であることを表します。",
    en: "Reports an invalid or incomplete regular-expression escape.",
  },
  "Reports a malformed or misplaced quantifier.": {
    ja: "繰り返しの指定が不正、または位置が誤っていることを表します。",
    en: "Reports a malformed or misplaced quantifier.",
  },
  "Reports an invalid character-class range.": {
    ja: "文字クラスの範囲指定が不正であることを表します。",
    en: "Reports an invalid character-class range.",
  },
  "Reports whether the leftmost-first engine finds a match.": {
    ja: "文字列に一致があるかどうかを返します。複数の候補があれば、最も左の位置から最初の候補を選びます。",
    en: "Reports whether the text contains a match. When there are several candidates, selects the first candidate at the leftmost position.",
  },
  "Captured source text together with its UTF-8 byte span.": {
    ja: "キャプチャーした文字列と、その開始を含み終了を含まないUTF-8バイト範囲です。",
    en: "Captured source text together with its UTF-8 byte span.",
  },
  "Classifies a portable regular-expression compile failure.": {
    ja: "正規表現のコンパイル失敗の種類を表します。",
    en: "Classifies a portable regular-expression compile failure.",
  },
  "Reports a compile failure at a UTF-8 byte offset in the pattern.": {
    ja: "正規表現のコンパイル失敗を、パターン内のUTF-8バイト位置とともに表します。",
    en: "Reports a compile failure at a UTF-8 byte offset in the pattern.",
  },
  "A leftmost match with ordered and named captures.": {
    ja: "最も左の一致です。順序付き・名前付きのキャプチャーを持ちます。",
    en: "A leftmost match with ordered and named captures.",
  },
  "Controls case folding, multiline anchors, and dot-newline matching.": {
    ja: "大文字・小文字の扱い、複数行の先頭・末尾、ドットと改行の一致を指定します。",
    en: "Controls case folding, multiline anchors, and dot-newline matching.",
  },
  "Half-open UTF-8 byte span in the searched text.": {
    ja: "対象文字列の、開始を含み終了を含まないUTF-8バイト範囲です。",
    en: "Half-open UTF-8 byte span in the searched text.",
  },
  "Opaque compiled portable regular expression with pinned Unicode semantics.":
    {
      ja: "コンパイル済みの正規表現です。実行環境が変わっても、定められたUnicodeの規則に従って検索します。",
      en: "A compiled portable regular expression. It searches using the specified Unicode rules across execution environments.",
    },
  "Replaces every match with literal text; capture markers are not expanded.": {
    ja: "すべての一致を指定した文字列に置き換えます。キャプチャー参照の記号は展開しません。",
    en: "Replaces every match with literal text; capture markers are not expanded.",
  },
  "Replaces matches by invoking a callback in source order.": {
    ja: "一致ごとに関数を呼び、その結果で順に置き換えます。",
    en: "Replaces matches by invoking a callback in source order.",
  },
  "Splits text at non-overlapping portable Regex matches.": {
    ja: "重複しない正規表現の一致を区切りとして、文字列を分割します。",
    en: "Splits text at non-overlapping portable Regex matches.",
  },
  "Reports a pattern that ends before its syntax is complete.": {
    ja: "正規表現の構文が完成する前に、パターンが終わっていることを表します。",
    en: "Reports a pattern that ends before its syntax is complete.",
  },
  "Reports an unexpected pattern character.": {
    ja: "パターンに想定外の文字があることを表します。",
    en: "Reports an unexpected pattern character.",
  },
  "Reports syntax intentionally excluded from the portable Regex contract.": {
    ja: "移植可能な正規表現では意図的に使えない構文を表します。",
    en: "Reports syntax intentionally excluded from the portable Regex contract.",
  },
  "Decodes UTF-8 and replaces invalid sequences.": {
    ja: "UTF-8を復号します。不正な並びは代替文字に置き換えます。",
    en: "Decodes UTF-8 and replaces invalid sequences.",
  },
  "Strictly decodes UTF-8 or reports the first invalid byte offset.": {
    ja: "UTF-8を厳密に復号します。不正なら最初のバイト位置を返します。",
    en: "Strictly decodes UTF-8 or reports the first invalid byte offset.",
  },
  "Encodes String as UTF-8 Bytes.": {
    ja: "StringをUTF-8のBytesへ変換します。",
    en: "Encodes String as UTF-8 Bytes.",
  },
  "Reports the byte offset of the first invalid UTF-8 sequence.": {
    ja: "最初の不正なUTF-8の並びの、バイト位置を表します。",
    en: "Reports the byte offset of the first invalid UTF-8 sequence.",
  },
  "Explicitly converts Left to one error and Right to Valid.": {
    ja: "Leftを一個のエラーを持つInvalid、RightをValidへ明示的に変換します。",
    en: "Explicitly converts Left to one error and Right to Valid.",
  },
  "Constructs an invalid result from a NonEmptyList, preserving error order.": {
    ja: "NonEmptyListからInvalidを作り、エラーの順序を保ちます。",
    en: "Constructs an invalid result from a NonEmptyList, preserving error order.",
  },
  "Constructs an invalid result containing one error.": {
    ja: "エラー一個を含むInvalidを作ります。",
    en: "Constructs an invalid result containing one error.",
  },
  "Explicitly converts Invalid to Left containing all errors, or Valid to Right.":
    {
      ja: "Invalidを全エラーを持つLeft、ValidをRightへ明示的に変換します。",
      en: "Explicitly converts Invalid to Left containing all errors, or Valid to Right.",
    },
  "Independent validation with a Valid value or a non-empty, source-ordered collection of errors.":
    {
      ja: "独立した検証の結果です。成功はValidの値、失敗は元の順序を保つ一個以上のエラーで表します。",
      en: "Independent validation with a Valid value or a non-empty, source-ordered collection of errors.",
    },
  "Constructs a successful Validation value.": {
    ja: "検証に成功した値を作ります。",
    en: "Constructs a successful Validation value.",
  },
}

export function referenceDescription(
  description: string
): ReferenceDescription {
  const translated = descriptions[description]
  if (translated === undefined) {
    throw new Error(`Missing bilingual API explanation: ${description}`)
  }
  return translated
}
