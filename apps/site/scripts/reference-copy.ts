// Site-owned Japanese explanations. Keys are exact compiler descriptions;
// canonical API metadata and signatures remain compiler-owned.
const japaneseDescriptions: Readonly<Record<string, string>> = {
  "Returns false at the first element rejected by the predicate.":
    "条件を満たさない最初の要素でFalseを返します。",
  "Returns true at the first element accepted by the predicate.":
    "条件を満たす最初の要素でTrueを返します。",
  "Combines a reducible collection using its Monoid instance.":
    "Monoidで定めた空の値と結合操作を使い、コレクションの要素を一つにまとめます。",
  "Runs one Effect for every value exposed by an Iterable instance.":
    "Iterableから取り出す各値について、一つずつEffectを実行します。",
  "Joins a reducible collection of strings with a separator.":
    "コレクションの文字列を、指定した区切り文字でつなぎます。",
  "Reads the next Iterator value and its remaining Iterator.":
    "Iteratorから次の値と、残りを読むIteratorを取り出します。",
  "Multiplies every element of a reducible collection from one.":
    "1から始めて、コレクションの全要素を掛け合わせます。",
  "Folds a reducible collection into one accumulated value.":
    "コレクションの要素を順に処理し、一つの蓄積値にまとめます。",
  "Adds every element of a reducible collection from zero.":
    "0から始めて、コレクションの全要素を足し合わせます。",
  "Builds a lazy Iterator from an initial state and step function.":
    "初期状態と次の状態を求める関数から、必要なときに値を計算するIteratorを作ります。",
  "Writes text followed by a newline through Console.":
    "Consoleを使い、文字列の後に改行を付けて出力します。",
  "Writes text through Console without adding a newline.":
    "Consoleを使い、改行を付けずに文字列を出力します。",
  "Renders a value through its Show instance and writes it through Console.":
    "値をShowの実装で文字列にし、Consoleへ出力します。",
  "Reads one optional line through Stdin.":
    "Stdinから一行を読みます。行がなければNothingを返します。",
  "Transparent alias for an Effect with no environment and no recoverable failure.":
    "環境のサービスを必要とせず、回復可能な失敗もないEffectの型別名です。",
  "Represents either a typed failure or a success value.":
    "失敗の値か成功の値の、どちらか一方を表します。",
  "Constructs a present Maybe value.": "値があることを表すJustを作ります。",
  "Constructs the failure side of Either.":
    "Eitherの失敗側の値であるLeftを作ります。",
  "Represents an optional value as Nothing or Just.":
    "値がないNothingか、値があるJustのどちらかを表します。",
  "Constructs an empty Maybe value.": "値がないことを表すNothingを作ります。",
  "Constructs the success side of Either.":
    "Eitherの成功側の値であるRightを作ります。",
  "Constructs the equal Ordering result.":
    "二つの値が等しいことを表すEqualを作ります。",
  "Constructs the greater-than Ordering result.":
    "左の値が右より大きいことを表すGreaterを作ります。",
  "Constructs the less-than Ordering result.":
    "左の値が右より小さいことを表すLessを作ります。",
  "Represents a total comparison result as Less, Equal, or Greater.":
    "比較結果をLess、Equal、Greaterのいずれかで表します。",
  "Selects multiplication through a nominal wrapper; Monoid requires One and homogeneous Mul evidence.":
    "値を別の型で包み、Monoidの結合を乗算として扱います。Oneと、同じ型同士のMulの実装が必要です。",
  "Selects addition through a nominal wrapper; Monoid requires Zero and homogeneous Add evidence.":
    "値を別の型で包み、Monoidの結合を加算として扱います。Zeroと、同じ型同士のAddの実装が必要です。",
  "Dispatches through the standard trait instance selected by the operand types.":
    "左右の値の型に対応する、標準traitの実装を呼び出します。",
  "Persistent List cons. Precedence 4, right associative; evaluates head then tail once and shares the tail in O(1). The operator section (:) is a curried function.":
    "Listの先頭に要素を追加します。優先順位は4で右結合です。先頭と末尾を一度ずつ計算し、末尾のリストは共有するため追加はO(1)です。(:)はカリー化された関数としても使えます。",
  "Maybe fallback. Precedence 0, right-associative. Evaluates the left once; evaluates the fallback only for Nothing. Syntax only: no operator section or overload.":
    "Maybeに値がなければ右の式を使います。優先順位は0で右結合です。左は一度だけ計算し、Nothingのときだけ右を計算します。構文専用なので、(??)という関数にしたり再定義したりはできません。",
  "Operator spelling for a standard trait method.":
    "標準traitの操作を、演算子の記号で呼び出します。",
  "Applies the selected arithmetic trait instance.":
    "値の型に対応する算術traitの実装を使って計算します。",
  "Standard type class used for generic dispatch.":
    "複数の型に共通の操作を定める標準traitです。型ごとの処理はinstanceで提供します。",
  "Appends the suffix after the collection values.":
    "コレクションの末尾に指定した値をつなぎます。",
  "Applies a function inside an Applicative context.":
    "Applicativeに包まれた関数を、同じ種類の値に適用します。",
  "Compares two values with the selected total ordering.":
    "型に対応するOrdの実装で二つの値を比較します。",
  "Standard function provided by the compiler-owned library surface.":
    "コンパイラが提供する標準関数です。受け取る型と戻り値は下の署名で確認できます。",
  "Returns the identity value for a Monoid.":
    "Monoidの結合で値を変えない、空の値を返します。",
  "Tests two values with the selected Eq instance.":
    "型に対応するEqの実装で、二つの値が等しいか調べます。",
  "Transforms each element to a collection and flattens the results.":
    "各要素からコレクションを作り、その結果を一つにつなぎます。",
  "Computes the hash defined by the selected Hash instance.":
    "型に対応するHashの実装で、ハッシュ値を計算します。",
  "Creates an Iterator over a collection.":
    "コレクションを順に読むIteratorを作ります。",
  "Transforms the value inside a Functor without changing its shape.":
    "Functorの中の値を変換します。外側の種類や構造は保ちます。",
  "Returns the numeric one for the selected type.":
    "指定した数値型の1を返します。",
  "Lifts a value into an Applicative context.":
    "値をApplicativeの中に包みます。",
  "Traverses a Functor with an Applicative effect while preserving its shape.":
    "各要素にApplicativeの処理を行い、元のFunctorの構造を保ってまとめます。",
  "Returns the numeric zero for the selected type.":
    "指定した数値型の0を返します。",
  "Selects an explicit numeric rounding mode.":
    "数値を丸める方法を、明示的に選ぶための値です。",
  "Type from the shared numeric rounding surface.":
    "数値の丸め方を指定するための型です。",
  "Parses, formats, or computes with Int under the safe integer contract.":
    "Intの解析、表示、計算を行います。Intで正確に表せる整数の範囲を検査します。",
  "Type from the checked safe integer surface.":
    "Intの値域を検査する操作に使う型です。",
  "Parses, formats, classifies, or explicitly converts an IEEE 754 Float.":
    "IEEE 754のFloatを解析・表示・分類し、必要な型変換を明示的に行います。",
  "Type from the explicit Float conversion surface.":
    "Floatを他の型へ明示的に変換する操作に使う型です。",
  "Pure portable mathematics on Float; angles are radians and atan2 takes y then x.":
    "Floatの純粋な数学関数です。角度はラジアンで指定します。atan2にはy、xの順で渡します。",
  "Compiler-owned standard library symbol.":
    "標準ライブラリの公開項目です。種類と型は下の署名で確認できます。",
  "Flattens one level of nested collections in source order.":
    "入れ子のコレクションを一段だけ平らにし、元の要素順を保ちます。",
  "Returns the elements after skipping the requested leading count.":
    "指定した個数の先頭要素を除いたコレクションを返します。",
  "Transforms collection elements and drops Nothing results.":
    "各要素を変換し、結果がNothingのものを除きます。",
  "Keeps collection elements whose predicate returns True.":
    "条件を調べる関数がTrueを返す要素だけを残します。",
  "Returns the first matching element, or Nothing when none matches.":
    "条件を満たす最初の要素を返します。該当する要素がなければNothingです。",
  "Returns the indexed element, or Nothing when the index is invalid.":
    "指定した添字の要素を返します。添字が範囲外ならNothingです。",
  "Returns the first element, or Nothing for an empty collection.":
    "先頭の要素を返します。空ならNothingです。",
  "Returns whether the collection contains no elements.":
    "要素が一つもないかどうかを返します。",
  "Returns the number of elements in the collection.": "要素の個数を返します。",
  "Returns the collection elements in reverse order.":
    "要素を逆順に並べたコレクションを返します。",
  "Returns all elements after the first, or Nothing for an empty collection.":
    "先頭を除いた残りの要素を返します。空ならNothingです。",
  "Returns at most the requested number of leading elements.":
    "先頭から、指定した個数までの要素を返します。",
  "Copies the Array elements into a persistent List in source order.":
    "Arrayの要素を順にコピーし、不変のListを作ります。",
  "Copies the List elements into an Array in source order.":
    "Listの要素を順にコピーし、Arrayを作ります。",
  "Type or trait from the standard HTML surface.":
    "型付きHTMLの要素や属性を表すための型・traitです。",
  "Creates a self-contained article section.":
    "一つの記事として独立したarticle要素を作ります。",
  "Creates content related to the surrounding section.":
    "周囲の内容に関連する補足を表すaside要素を作ります。",
  "Creates a hyperlink with typed navigation props.":
    "型付きのリンク先の指定を使い、ハイパーリンクを作ります。",
  "Validates a custom, data, or ARIA attribute without bypassing escaping.":
    "独自属性、data属性、ARIA属性を検証します。HTMLのエスケープ処理は省略しません。",
  "Validated opaque custom HTML attribute name and value.":
    "検証済みの独自HTML属性の名前と値を表します。中身を直接変更できない型です。",
  "Creates an audio media element.": "音声を再生するaudio要素を作ります。",
  "Creates a block quotation.":
    "まとまった引用を表すblockquote要素を作ります。",
  "Creates the document body container.":
    "文書の本文を囲むbody要素を作ります。",
  "Creates a void line-break element without children.":
    "子要素を持たない改行用のbr要素を作ります。",
  "Creates or renders typed HTML through the standard HTML surface.":
    "型付きHTMLを作る、または文字列として出力する操作です。",
  "Creates a caption for a typed data table.":
    "表の見出しを表すcaption要素を作ります。",
  "Immutable form-control snapshot containing value and optional checked state: Just for checkbox/radio, Nothing for value controls.":
    "フォーム操作時の値とチェック状態を保持する不変の値です。checkboxとradioはチェック状態がJust、それ以外の値の入力はNothingです。",
  "Creates an inline code fragment.": "文中のコードを表すcode要素を作ります。",
  "Creates a custom element from a validated Tag.":
    "検証済みのTagからカスタム要素を作ります。",
  "Validates a custom-element name into an opaque Tag.":
    "カスタム要素名を検証し、Tagに変換します。",
  "Creates a disclosure element with typed open state.":
    "開閉状態を指定できるdetails要素を作ります。",
  "Creates a dialog element with typed open state.":
    "開閉状態を指定できるdialog要素を作ります。",
  "Prevents the browser default and stops propagation before dispatching an Action.":
    "ブラウザの既定動作とイベントの伝播を止めてから、Actionを送ります。",
  "Prevents the browser default before dispatching an Action.":
    "ブラウザの既定動作を止めてから、Actionを送ります。",
  "Dispatches an Action without controlling the browser event.":
    "ブラウザの既定動作や伝播は変えず、Actionを送ります。",
  "Stops event propagation before dispatching an Action.":
    "イベントの伝播を止めてから、Actionを送ります。",
  "Creates emphasized text.": "強調する文言を囲むem要素を作ります。",
  "Explicit event outcome describing dispatch, default prevention, and propagation control.":
    "Actionの送信、既定動作の抑止、伝播の停止を指定するイベント処理の結果です。",
  "Groups related controls in a typed form.":
    "関連するフォームの入力をfieldsetでまとめます。",
  "Creates footer content for a page or section.":
    "ページや区画の末尾を表すfooter要素を作ります。",
  "Creates a typed form whose onSubmit message prevents native page reload.":
    "型付きのform要素を作ります。onSubmitはブラウザの通常のページ再読み込みを防ぎます。",
  "Creates a typed document heading.": "文書の見出しを作ります。",
  "Creates introductory content for a page or section.":
    "ページや区画の導入を表すheader要素を作ります。",
  "Creates the metadata container for a typed document.":
    "文書のメタ情報を囲むhead要素を作ります。",
  "Creates a void thematic-break element without children.":
    "子要素を持たない、区切りを表すhr要素を作ります。",
  "Describes a rejected custom HTML value or unsafe Web URL.":
    "独自のHTML値や安全でないWeb URLを拒否した理由を表します。",
  "Creates the root html element for a typed document.":
    "文書全体のルートとなるhtml要素を作ります。",
  "Ignores an event without dispatch or browser control.":
    "Actionを送らず、ブラウザの動作も変えずにイベントを無視します。",
  "Creates a void image element with required source and alt text.":
    "画像のURLと代替テキストを必須とするimg要素を作ります。子要素はありません。",
  "Immutable text-input snapshot containing only the current String value.":
    "文字入力時点のStringだけを保持する不変の値です。",
  "Creates a controlled input with typed input and change event snapshots.":
    "値をアプリケーション側で管理するinput要素を作ります。入力・変更イベントの値には型が付きます。",
  "Reports an invalid custom attribute spelling.":
    "独自属性名の書き方が不正であることを表します。",
  "Reports a rejected custom-element name.":
    "カスタム要素名を受け付けられないことを表します。",
  "Immutable keyboard snapshot containing key identity, repeat state, and modifier keys.":
    "キー、押し続けた状態、修飾キーを保持する、キー操作時点の不変の値です。",
  "Creates a label connected through the htmlFor prop.":
    "htmlForで入力と対応付けるlabel要素を作ります。",
  "Creates a caption for a fieldset.":
    "fieldsetの見出しとなるlegend要素を作ります。",
  "Creates a void external-resource link element without children.":
    "外部リソースを参照するlink要素を作ります。子要素はありません。",
  "Creates a list item.": "リストの項目を表すli要素を作ります。",
  "Creates a void metadata element without children.":
    "メタ情報を表すmeta要素を作ります。子要素はありません。",
  "Immutable mouse snapshot containing button, coordinates, and modifier keys.":
    "ボタン、座標、修飾キーを保持する、マウス操作時点の不変の値です。",
  "Creates a navigation section.": "ナビゲーションを表すnav要素を作ります。",
  "Creates an ordered list.": "順序付きリストのol要素を作ります。",
  "Creates an option for a typed selection control.":
    "選択式入力の項目を表すoption要素を作ります。",
  "Validates a relative or allowlisted Web URL into an opaque value.":
    "相対URL、または許可されたWeb URLを検証して専用の型に変換します。",
  "Creates a responsive image container.":
    "表示条件に応じた画像をまとめるpicture要素を作ります。",
  "Immutable pointer snapshot distinguishing mouse, touch, and pen input.":
    "マウス・タッチ・ペンの種類を区別して保持する、ポインター操作時点の不変の値です。",
  "Creates preformatted text.": "空白と改行を保つpre要素を作ります。",
  "Renders typed HTML as a complete document string.":
    "型付きHTMLを完全な文書の文字列へ変換します。",
  "Renders typed HTML to an escaped fragment string.":
    "型付きHTMLを、必要なエスケープを施した断片の文字列へ変換します。",
  "Reports a custom attribute that collides with typed or runtime-owned props.":
    "独自属性が、型付きの属性や実行基盤が使う属性と衝突したことを表します。",
  "Immutable scroll snapshot containing the current element offsets.":
    "要素のスクロール位置を保持する、操作時点の不変の値です。",
  "Creates a typed selection control using the shared change snapshot.":
    "共通の変更イベントの値を使い、型付きのselect要素を作ります。",
  "Creates secondary or fine-print text.":
    "補足や注記を表すsmall要素を作ります。",
  "Creates a void media source element without children.":
    "メディアの読み込み元を表すsource要素を作ります。子要素はありません。",
  "Creates strongly emphasized text.": "重要な文言を囲むstrong要素を作ります。",
  "Validates and converts a style record into inline Style.":
    "スタイルのレコードを検証し、インラインStyleに変換します。",
  "Creates the visible summary for a disclosure element.":
    "detailsの開閉に使う見出しsummaryを作ります。",
  "Creates a typed data table.": "型付きのデータ表を作ります。",
  "Validated opaque name for a custom HTML element.":
    "検証済みのカスタムHTML要素名です。直接中身を変更できない型です。",
  "Groups body rows in a typed data table.":
    "表の本体の行をtbodyでまとめます。",
  "Creates a data cell with typed span props.":
    "列や行にまたがる指定を持つ、表のデータセルtdを作ります。",
  "Creates a controlled textarea with typed input and change snapshots.":
    "値をアプリケーション側で管理するtextareaを作ります。入力・変更時の値には型が付きます。",
  "Groups footer rows in a typed data table.":
    "表の末尾の行をtfootでまとめます。",
  "Groups header rows in a typed data table.":
    "表の先頭の行をtheadでまとめます。",
  "Creates a header cell with typed span props.":
    "列や行にまたがる指定を持つ、表の見出しセルthを作ります。",
  "Creates the document title element.":
    "文書のタイトルを表すtitle要素を作ります。",
  "Creates a row in a typed data table.": "表の行trを作ります。",
  "Creates an unordered list.": "順序なしリストのul要素を作ります。",
  "Reports a URL rejected for its scheme, credentials, or control characters.":
    "URLの方式、認証情報、制御文字が原因で拒否されたことを表します。",
  "Creates a video media element.": "動画を再生するvideo要素を作ります。",
  "Validated opaque URL for security-sensitive HTML attributes.":
    "安全性が必要なHTML属性に使う、検証済みのURLです。",
  "Immutable wheel snapshot containing deltas, mode, coordinates, and modifiers.":
    "移動量、単位、座標、修飾キーを保持する、ホイール操作時点の不変の値です。",
  "Creates a pure namespaced SVG scene or explicitly bridges it to Html.":
    "SVG専用の名前空間を使う描画を純粋な値として作ります。Htmlへ組み込むときは明示的に変換します。",
  "Type from the standard SVG scene surface.": "型付きSVGの描画に使う型です。",
  "Mounts a typed state-update-view application into a DOM target.":
    "状態・更新関数・表示関数を持つ型付きアプリケーションを、指定したDOMの場所に配置します。",
  "Runs typed browser DOM behavior through the standard DOM surface.":
    "DOMのサービスを使い、ブラウザの要素を操作します。",
  "Type from the standard DOM capability surface.":
    "DOMを操作するサービスに使う型です。",
  "Captures a live pointer on an explicit DOM target through Effect.":
    "指定したDOM要素で、操作中のポインターを捕捉するEffectです。",
  "Snapshots an element bounding rectangle through the DOM capability.":
    "DOMのサービスを使い、要素の外接矩形を取得します。",
  "Observes element geometry with an explicit disposable resource lifecycle.":
    "要素の位置や大きさを監視します。監視を終了するリソースの解放操作が必要です。",
  "Releases a captured pointer on an explicit DOM target through Effect.":
    "指定したDOM要素のポインター捕捉を解放するEffectです。",
  "Creates or transforms reactive values through the standard Signal surface.":
    "変化する値を表すSignalを作る、または変換します。",
  "Creates a mutable Signal inside Effect.":
    "Effectの中で、更新できるSignalを作ります。",
  "Derives a Signal by transforming each current value.":
    "現在値を変換して、元の値の変化に追従するSignalを作ります。",
  "Type from the standard Signal surface.":
    "変化する値を表すSignalの操作に使う型です。",
  "Type from the standard Clock capability surface.":
    "時刻の取得や待機を行うClockサービスの型です。",
  "Reads the current monotonic clock instant through Clock Effect.":
    "ClockのEffectで、後戻りしない時計の現在時点を読みます。",
  "Waits for a Duration through the Clock Effect capability.":
    "ClockのEffectで、指定したDurationの間待ちます。",
  "Adds two Duration values with range validation.":
    "二つのDurationを加算し、結果の範囲を検証します。",
  "Constructs or computes with standard time values.":
    "標準の時刻や時間の長さを作る、または計算します。",
  "Type from the standard time surface.": "時刻や時間の長さを表す型です。",
  "Converts exact hours into Duration.": "時間の整数値をDurationに変換します。",
  "Converts exact milliseconds into Duration.":
    "ミリ秒の整数値をDurationに変換します。",
  "Converts exact minutes into Duration.": "分の整数値をDurationに変換します。",
  "Validates an Int as a nanosecond Duration.":
    "Intをナノ秒のDurationとして検証します。",
  "Converts exact seconds into Duration.": "秒の整数値をDurationに変換します。",
  "Returns a Duration's integer nanosecond count.":
    "Durationのナノ秒数をIntで返します。",
  "Returns the exact zero-nanosecond Duration.":
    "0ナノ秒のDurationを返します。",
  "Validates and appends one HTTP header field.":
    "HTTPヘッダー一項目を検証し、末尾に追加します。",
  "Performs HTTP client operations through the standard HttpClient capability.":
    "HttpClientサービスを使い、HTTPリクエストを送ります。",
  "Type from the standard HTTP client capability surface.":
    "HTTPクライアントのサービスに使う型です。",
  "The standard CONNECT Method value.":
    "CONNECTメソッドを表す標準のMethod値です。",
  "Validates an uppercase custom HTTP method token.":
    "大文字の独自HTTPメソッド名を検証します。",
  "The standard DELETE Method value.":
    "DELETEメソッドを表す標準のMethod値です。",
  "The empty immutable Headers value.": "空の不変のHeaders値です。",
  "Reads the message carried by an HTTP client failure.":
    "HTTPクライアントの失敗が持つメッセージを読みます。",
  "The standard GET Method value.": "GETメソッドを表す標準のMethod値です。",
  "Immutable ordered HTTP header collection.":
    "順序を保つ不変のHTTPヘッダーの集まりです。",
  "The standard HEAD Method value.": "HEADメソッドを表す標準のMethod値です。",
  "Positive response body byte limit.":
    "レスポンス本文の最大バイト数を表す正の値です。",
  "Typed validation failure while building an HTTP value.":
    "HTTPの値を作る際の検証の失敗を表します。",
  "Typed HTTP transport, protocol, or body failure.":
    "HTTPの通信、プロトコル、本文の読み取りの失敗を表します。",
  "Normalized absolute HTTP or HTTPS URL.":
    "正規化済みの絶対HTTPまたはHTTPS URLです。",
  "Opaque validated HTTP request method.":
    "検証済みのHTTPリクエストメソッドです。",
  "Reads the text of a validated HTTP Method.":
    "検証済みのMethodの文字列を読みます。",
  "The standard OPTIONS Method value.":
    "OPTIONSメソッドを表す標準のMethod値です。",
  "Parses and normalizes an absolute HTTP or HTTPS URL.":
    "絶対HTTPまたはHTTPS URLを解析し、正規化します。",
  "The standard PATCH Method value.": "PATCHメソッドを表す標準のMethod値です。",
  "The standard POST Method value.": "POSTメソッドを表す標準のMethod値です。",
  "The standard PUT Method value.": "PUTメソッドを表す標準のMethod値です。",
  "Removes every case-insensitive occurrence of a header.":
    "大文字・小文字を区別せず、指定したヘッダーをすべて削除します。",
  "Renders a normalized HttpUrl.": "正規化済みのHttpUrlを文字列にします。",
  "Builds an immutable request head from Method and HttpUrl.":
    "MethodとHttpUrlから、不変のリクエストの先頭部分を作ります。",
  "Immutable HTTP request head without a body.":
    "本文を含まない、不変のHTTPリクエストの先頭部分です。",
  "Copies the immutable Bytes body of a small response.":
    "小さいレスポンスの、不変のBytes本文をコピーして返します。",
  "Copies the immutable headers of a small response.":
    "小さいレスポンスの、不変のヘッダーをコピーして返します。",
  "Reads the validated status of a small response.":
    "小さいレスポンスの、検証済みステータスを読みます。",
  "Small HTTP response with an immutable Bytes body.":
    "不変のBytes本文を持つ、小さいHTTPレスポンスです。",
  "Sends an explicit Bytes body through HttpClient.":
    "HttpClientを使い、指定したBytesを本文として送ります。",
  "Sends an explicit empty body through HttpClient.":
    "HttpClientを使い、空の本文を明示して送ります。",
  "Replaces a header field at its first ordered position.":
    "ヘッダーを、元の最初の位置で置き換えます。",
  "Reads the Int code of a validated HTTP Status.":
    "検証済みStatusの数値をIntで読みます。",
  "Validates an HTTP status code from 100 through 999.":
    "100から999のHTTPステータスコードを検証します。",
  "Opaque validated HTTP response status.":
    "検証済みのHTTPレスポンスのステータスです。",
  "The standard TRACE Method value.": "TRACEメソッドを表す標準のMethod値です。",
  "Returns the exact non-negative magnitude of a BigInt.":
    "BigIntの符号を除いた、正確な絶対値を返します。",
  "Describes a BigInt that cannot be narrowed to Int exactly.":
    "BigIntを正確にIntへ変換できなかった理由を表します。",
  "Reports a zero BigInt divisor.": "BigIntの除数が0であることを表します。",
  "Describes checked BigInt division or remainder by zero.":
    "BigIntの除算や余りの計算で、除数が0の場合の失敗を表します。",
  "Reports a BigInt outside the safe Int range.":
    "BigIntがIntで表せる範囲外であることを表します。",
  "Describes an invalid BigInt text or radix without a magnitude limit.":
    "BigIntの文字列や基数が不正であることを表します。値の大きさには上限を設けません。",
  "Describes a checked negative BigInt exponent.":
    "BigIntの指数に負の整数を指定した失敗を表します。",
  "Opaque arbitrary-precision signed integer value.":
    "任意精度の符号付き整数です。",
  "Divides exactly represented BigInts or returns a typed zero-divisor failure.":
    "BigInt同士を除算します。除数が0なら型付きの失敗を返します。",
  "Raises a BigInt by a non-negative Int using exact exponentiation.":
    "BigIntを、0以上のIntで指定した回数だけ正確に累乗します。",
  "Computes truncating BigInt remainder or returns a typed zero-divisor failure.":
    "BigInt同士の余りを計算します。除数が0なら型付きの失敗を返します。商は0に向けて切り捨てます。",
  "Reports an empty BigInt input.":
    "BigIntの入力文字列が空であることを表します。",
  "Formats a BigInt in radix 2 through 36.":
    "BigIntを、2から36で指定した基数の文字列にします。",
  "Formats a BigInt as canonical decimal text.":
    "BigIntを、標準の十進文字列にします。",
  "Converts Int to BigInt exactly.": "Intを正確にBigIntへ変換します。",
  "Reports the first invalid BigInt digit at a UTF-8 byte offset.":
    "BigInt文字列の最初の不正な数字の位置を、UTF-8のバイト位置で表します。",
  "Reports a BigInt radix outside the inclusive range 2 through 36.":
    "BigIntの基数が2から36の範囲外であることを表します。",
  "Reports a negative Int exponent.": "Intの指数が負であることを表します。",
  "Parses signed BigInt text in radix 2 through 36.":
    "2から36で指定した基数で、符号付きBigIntの文字列を解析します。",
  "Parses canonical decimal text into an exact BigInt.":
    "十進文字列を、正確なBigIntへ解析します。",
  "Returns minus one, zero, or one for a BigInt.":
    "BigIntの符号を-1、0、1のいずれかで返します。",
  "Narrows BigInt to Int with a typed range failure.":
    "BigIntをIntへ変換します。範囲外なら型付きの失敗を返します。",
  "Describes an Int that cannot be represented as a Byte.":
    "IntがByteで表せない値であることを表します。",
  "Describes an invalid half-open Bytes slice range.":
    "Bytesの切り出し範囲が不正であることを表します。範囲は開始位置を含み、終了位置を含みません。",
  "Immutable sequence of bytes with explicit copy boundaries.":
    "不変のバイト列です。データのコピーを行う境界は操作ごとに明示されています。",
  "Validates an Int and converts it to an opaque Byte.":
    "Intを検証してByteへ変換します。",
  "Opaque unsigned 8-bit value in the inclusive range 0 through 255.":
    "0から255の符号なし8ビット値です。",
  "Creates an independent copy of Bytes.":
    "元の値と独立したBytesのコピーを作ります。",
  "Creates an empty immutable Bytes value.": "空の不変のBytesを作ります。",
  "Copies an Array of Byte values into immutable Bytes.":
    "ByteのArrayをコピーして、不変のBytesを作ります。",
  "Validates Int values in order and copies them into Bytes.":
    "Intを順に検証し、コピーしてBytesを作ります。",
  "Creates Bytes containing exactly one Byte.":
    "Byte一個だけを含むBytesを作ります。",
  "Returns a validated half-open slice without exposing mutation.":
    "開始を含み終了を含まない範囲を検証し、切り出したBytesを返します。変更可能な内部データは公開しません。",
  "Copies Bytes into an Array of Byte values.":
    "BytesをコピーしてByteのArrayを作ります。",
  "Copies Bytes into an Array of Int values.":
    "BytesをコピーしてIntのArrayを作ります。",
  "Converts a Byte to its Int value.": "Byteの値をIntへ変換します。",
  "Describes a strict RFC 4648 Base64 decoding failure.":
    "RFC 4648のBase64を厳密に復号した際の失敗を表します。",
  "Strictly decodes canonical padded RFC 4648 Base64 text.":
    "RFC 4648の、末尾の埋め合わせを持つ標準のBase64文字列を厳密に復号します。",
  "Strictly decodes canonical unpadded URL-safe Base64 text.":
    "末尾の埋め合わせを持たない、URL向けBase64文字列を厳密に復号します。",
  "Encodes Bytes as canonical padded RFC 4648 Base64 text.":
    "Bytesを、RFC 4648の末尾の埋め合わせを持つBase64文字列にします。",
  "Encodes Bytes as canonical unpadded URL-safe Base64 text.":
    "Bytesを、末尾の埋め合わせを持たないURL向けBase64文字列にします。",
  "Reports the UTF-8 byte offset of the first invalid Base64 digit.":
    "最初の不正なBase64文字の位置を、UTF-8のバイト位置で表します。",
  "Reports an invalid Base64 input length measured in UTF-8 bytes.":
    "Base64入力の長さが不正であることを表します。長さの単位はUTF-8バイトです。",
  "Reports the UTF-8 byte offset of invalid Base64 padding.":
    "Base64の埋め合わせが不正な位置を、UTF-8のバイト位置で表します。",
  "Reports non-zero unused bits in the final Base64 sextet.":
    "Base64の最後の6ビットのうち、使われていない部分が0でないことを表します。",
  "Decodes ASCII hexadecimal text or reports the first typed failure.":
    "ASCIIの十六進文字列を復号します。不正な場合は最初の失敗を返します。",
  "Encodes Bytes as canonical lowercase hexadecimal text.":
    "Bytesを、標準の小文字の十六進文字列にします。",
  "Describes an invalid hexadecimal text input with a UTF-8 byte position.":
    "十六進文字列の不正な入力を、UTF-8のバイト位置とともに表します。",
  "Reports the UTF-8 byte offset of the first non-hexadecimal digit.":
    "最初の十六進数字でない文字の位置を、UTF-8のバイト位置で表します。",
  "Reports an odd hexadecimal input length measured in UTF-8 bytes.":
    "十六進文字列の長さが奇数であることを表します。長さの単位はUTF-8バイトです。",
  "Stops reduction immediately with the final result.":
    "最終結果を返し、要素をまとめる処理を直ちに止めます。",
  "Continues reduction with the new accumulator.":
    "新しい蓄積値を使い、要素をまとめる処理を続けます。",
  "Pure reduction control: Next continues and Done stops.":
    "純粋な集計の継続を表します。Nextは継続、Doneは終了です。",
  "Folds an Iterable in source order until Done; does not pull the remaining elements.":
    "Iterableを順に処理し、Doneで止めます。終了後の要素は取り出しません。",
  "Validates a positive precision and constructs a DecimalContext.":
    "精度が正であることを検証し、DecimalContextを作ります。",
  "Describes a checked decimal division failure.":
    "Decimalの除算で発生する検証済みの失敗を表します。",
  "Describes an invalid explicit decimal arithmetic context.":
    "Decimalの計算方法を指定する値が不正であることを表します。",
  "Explicit positive precision and rounding policy for decimal operations.":
    "Decimalの計算で使う、有効桁数と丸め方の明示的な指定です。有効桁数は正です。",
  "Describes a failed explicit Int or Float conversion.":
    "IntやFloatへの明示的な変換の失敗を表します。",
  "Reports a zero Decimal divisor.": "Decimalの除数が0であることを表します。",
  "Reports a Decimal with a nonzero fractional part.":
    "Decimalに0でない小数部分があり、整数へ変換できないことを表します。",
  "Reports a Decimal outside the finite binary64 range.":
    "Decimalが有限の64ビットFloatの範囲外であることを表します。",
  "Reports an integral Decimal outside the safe Int range.":
    "整数であるDecimalが、Intで表せる範囲外であることを表します。",
  "Describes invalid decimal text at a UTF-8 byte offset.":
    "Decimal文字列が不正な位置を、UTF-8のバイト位置で表します。",
  "Opaque finite arbitrary-precision decimal value.":
    "有限の任意精度の十進数です。",
  "Divides Decimals only when the exact result has a finite decimal expansion.":
    "正確な結果が有限桁の十進数になる場合にだけ、Decimal同士を除算します。",
  "Divides Decimals using explicit significant-digit precision and rounding.":
    "有効桁数と丸め方を指定して、Decimal同士を除算します。",
  "Reports NaN or infinity at the explicit Float conversion boundary.":
    "Floatからの変換にNaNまたは無限大が渡されたことを表します。",
  "Converts the exact finite binary64 value under an explicit DecimalContext.":
    "DecimalContextを指定し、有限の64ビットFloatの正確な値を変換します。",
  "Converts Int to Decimal exactly.": "Intを正確にDecimalへ変換します。",
  "Reports the first byte where decimal parsing becomes invalid.":
    "Decimalの解析で最初に不正となるバイト位置を表します。",
  "Reports a decimal precision that is zero or negative.":
    "Decimalの有効桁数が0または負であることを表します。",
  "Reports an exact quotient with a non-terminating decimal expansion.":
    "正確な商が、有限桁の十進数にならないことを表します。",
  "Parses exact Decimal text without binary floating point.":
    "二進の浮動小数点数を経由せず、十進文字列を正確なDecimalへ解析します。",
  "Returns the significant-digit precision of a context.":
    "DecimalContextの有効桁数を返します。",
  "Rounds a Decimal to an explicit decimal scale and rounding mode.":
    "小数点以下の桁数と丸め方を指定し、Decimalを丸めます。",
  "Returns the rounding mode of a context.":
    "DecimalContextの丸め方を返します。",
  "Rounds Decimal explicitly to the nearest finite binary64 value.":
    "Decimalを明示的に丸め、最も近い有限の64ビットFloatへ変換します。",
  "Converts an integral in-range Decimal to Int without rounding.":
    "範囲内の整数であるDecimalを、丸めずにIntへ変換します。",
  "Builds or transforms a cold standard Effect value.":
    "実行するまで処理が始まらないEffectを作る、または変換します。",
  "Moves typed failure into Either while preserving defects and cancellation.":
    "型付きの失敗をEitherの値へ移します。欠陥やキャンセルは捕捉しません。",
  "Defers construction of a cold Effect until each execution.":
    "Effectを実行するたびに、その処理を作る関数を呼び出します。",
  "Creates an Effect that fails with a typed error.":
    "指定した型のエラーで失敗するEffectを作ります。",
  "Type from the standard Effect control surface.":
    "Effectの実行や継続を制御する型です。",
  "Runs cold actions sequentially in Iterable order, stopping successfully at Break without pulling the next element.":
    "Iterableの順にEffectを実行します。Breakなら正常に終了し、次の要素は取り出しません。",
  "Lifts Either into Effect without performing an external operation.":
    "EitherをEffectへ変換します。この操作自体は外部への入出力を行いません。",
  "Lifts Maybe into Effect with an explicit missing-value error.":
    "MaybeをEffectへ変換します。値がない場合に使うエラーを明示的に渡します。",
  "Normal-success control for sequential Effect traversal: Continue or Break.":
    "Effectで要素を順に処理する際の正常な継続を表します。Continueは継続、Breakは終了です。",
  "Transforms an Effect failure while preserving its success value.":
    "Effectの失敗値を変換します。成功値は変えません。",
  "Projects an outer environment into the environment required by an Effect.":
    "外側の環境から、そのEffectに必要な環境を取り出す関数を指定します。",
  "Runs an Effect with a fixed environment.":
    "固定した環境を渡してEffectを実行します。",
  "Recovers a typed Effect failure without catching defects or cancellation.":
    "型付きの失敗を別の処理で回復します。欠陥やキャンセルは捕捉しません。",
  "Builds a zero-delay Schedule with a bounded number of reruns.":
    "指定した回数まで、待たずに再実行するScheduleを作ります。",
  "Repeats successes according to a Schedule and Clock.":
    "ScheduleとClockに従い、成功した処理を繰り返します。",
  "Retries typed failures according to a Schedule and Clock.":
    "ScheduleとClockに従い、型付きの失敗が起きた処理を再試行します。",
  "Selects one value from the execution environment.":
    "実行環境から一つの値を取り出します。",
  "Builds a fixed-delay Schedule with bounded reruns.":
    "一定の待ち時間を挟み、指定した回数まで再実行するScheduleを作ります。",
  "Creates an Effect that succeeds with a value.":
    "指定した値で成功するEffectを作ります。",
  "Applies a Clock-backed timeout with an explicit typed failure on expiry.":
    "Clockを使って制限時間を設けます。時間切れでは明示的に渡した型付きエラーで失敗します。",
  "Applies a Clock-backed timeout and returns Maybe on expiry.":
    "Clockを使って制限時間を設け、時間切れではNothingを返します。",
  "Builds a Schedule that continues while a predicate is true.":
    "条件がTrueの間、再実行を続けるScheduleを作ります。",
  "Transforms the selected Left or Right branch once.":
    "選ばれたLeftまたはRightの中身を一度だけ変換します。",
  "Eliminates Either by calling only the selected branch function.":
    "LeftかRightに対応する関数だけを呼び出し、Eitherの中身を結果に変換します。",
  "Transforms only the Left payload.": "Leftの中身だけを変換します。",
  "Transforms only the Right payload using the standard Functor instance.":
    "標準Functorの実装を使い、Rightの中身だけを変換します。",
  "Traverses wrapped values with identity, preserving source shape and order.":
    "包まれた各値をそのまま使って処理をまとめ、元の構造と順序を保ちます。",
  "Exchanges Left and Right without changing the payload.":
    "中身を変えず、LeftとRightを入れ替えます。",
  "Uses the source Traversable and the module's Applicative without a separate traversal implementation.":
    "元のTraversableと、このモジュールのApplicativeを使って要素の処理をまとめます。",
  "Decodes every JSON array element in source order.":
    "JSON配列の全要素を順に読み取ります。",
  "Reports the root-to-leaf path and kind of a decode failure.":
    "読み取りに失敗した種類と、根元からその値までの位置を返します。",
  "Pure Json decoder that reports a typed path-aware DecodeError.":
    "Jsonを値に変換する純粋な関数です。失敗では位置を持つDecodeErrorを返します。",
  "Parses and decodes text through its selected JsonDecode dictionary.":
    "文字列を解析し、その型のJsonDecodeの実装で値に変換します。",
  "Pure function that encodes a value as Json.":
    "値をJsonへ変換する純粋な関数です。",
  "Encodes a value through its selected JsonEncode dictionary.":
    "その型のJsonEncodeの実装で、値をJsonへ変換します。",
  "Decodes a required object field and prepends its path segment on failure.":
    "必須のオブジェクトフィールドを読み取ります。失敗には、そのフィールドの位置を加えます。",
  "Selects a following decoder from a successful decoder result.":
    "読み取りに成功した値から、次に使う読み取り関数を選びます。",
  "Decodes one array position and prepends its index on failure.":
    "配列の一つの位置を読み取ります。失敗には、その添字を加えます。",
  "Reports invalid syntax or a duplicate object field.":
    "構文が不正、またはオブジェクトのフィールド名が重複していることを表します。",
  "Distinguishes JSON syntax failure from value decode failure.":
    "JSON構文の失敗と、値への変換の失敗を区別します。",
  "Exact JSON value with Decimal numbers and ordered object fields.":
    "数値には正確なDecimalを使い、オブジェクトのフィールド順を保つJSONの値です。",
  "Transforms a successful decoder result.":
    "読み取りに成功した値を変換します。",
  "Tries decoders in order and returns the first successful result.":
    "読み取り関数を順に試し、最初に成功した結果を返します。",
  "Decodes an object field as Maybe and accepts only a missing field as Nothing.":
    "フィールドをMaybeとして読みます。フィールドがない場合だけNothingとし、不正な値は成功扱いにしません。",
  "Parses RFC 8259 JSON with exact Decimal numbers and duplicate-field rejection.":
    "RFC 8259のJSONを解析します。数値は正確なDecimalで扱い、フィールド名の重複を拒否します。",
  "Decodes a declared homogeneous object field set and rejects unknown fields.":
    "指定した同じ型のフィールドの集合を読み取ります。不明なフィールドを拒否します。",
  "Writes compact deterministic JSON with minimal escaping.":
    "必要最小限のエスケープを施し、一定の形式の短いJSON文字列を作ります。",
  "Selects the fallback Maybe for Nothing, preserving an existing Just.":
    "Nothingなら代わりのMaybeを返し、Justなら元の値を保ちます。",
  "Selects the fallback for Nothing, or the Just payload. Arguments are strict.":
    "Nothingなら代わりの値、Justならその中身を返します。関数なので、引数は呼び出し前に計算されます。",
  "Creates a fresh Effect-local mutable Ref.":
    "Effectの実行ごとに新しいRefを作ります。Refの中身は明示的な操作で更新できます。",
  "Returns a result and updates a Ref in one atomic callback.":
    "一つの関数で結果と新しい値を求め、Refの値の更新と結果の取得をまとめて行います。",
  "Type from the standard mutable Ref surface.":
    "明示的に更新できるRefの操作に使う型です。",
  "Replaces the current Ref value atomically.":
    "Refの現在値を、一つの操作で置き換えます。",
  "Updates a Ref with one pure atomic callback.":
    "純粋な関数を一度呼び、その結果でRefを更新します。読み取りと更新は一つの操作です。",
  "Compiles a portable pattern or returns a typed error without throwing.":
    "移植可能な正規表現をコンパイルします。不正なら例外を投げず、型付きのエラーを返します。",
  "Compiles a portable pattern using explicit RegexOptions.":
    "RegexOptionsを明示して、移植可能な正規表現をコンパイルします。",
  "Returns the portable Regex defaults with every option disabled.":
    "すべての選択肢が無効の、標準RegexOptionsを返します。",
  "Reports a repeated named-capture identifier.":
    "同じ名前のキャプチャーが繰り返し指定されていることを表します。",
  "Escapes text so it denotes a literal portable Regex fragment.":
    "文字列を、正規表現の記号ではなく文字そのものとして扱えるようにエスケープします。",
  "Returns non-overlapping matches and advances one scalar after an empty match.":
    "重複しない一致をすべて返します。空の一致の後はUnicodeスカラー値一個分進みます。",
  "Reports an invalid or incomplete regular-expression escape.":
    "正規表現のエスケープが不正、または未完成であることを表します。",
  "Reports a malformed or misplaced quantifier.":
    "繰り返しの指定が不正、または位置が誤っていることを表します。",
  "Reports an invalid character-class range.":
    "文字クラスの範囲指定が不正であることを表します。",
  "Reports whether the leftmost-first engine finds a match.":
    "左側から最初の一致を選ぶ規則で、文字列に一致があるか返します。",
  "Captured source text together with its UTF-8 byte span.":
    "キャプチャーした文字列と、その開始を含み終了を含まないUTF-8バイト範囲です。",
  "Classifies a portable regular-expression compile failure.":
    "正規表現のコンパイル失敗の種類を表します。",
  "Reports a compile failure at a UTF-8 byte offset in the pattern.":
    "正規表現のコンパイル失敗を、パターン内のUTF-8バイト位置とともに表します。",
  "A leftmost match with ordered and named captures.":
    "最も左の一致です。順序付き・名前付きのキャプチャーを持ちます。",
  "Controls case folding, multiline anchors, and dot-newline matching.":
    "大文字・小文字の扱い、複数行の先頭・末尾、ドットと改行の一致を指定します。",
  "Half-open UTF-8 byte span in the searched text.":
    "対象文字列の、開始を含み終了を含まないUTF-8バイト範囲です。",
  "Opaque compiled portable regular expression with pinned Unicode semantics.":
    "Unicodeの扱いを固定した、コンパイル済みの移植可能な正規表現です。",
  "Replaces every match with literal text; capture markers are not expanded.":
    "すべての一致を指定した文字列に置き換えます。キャプチャー参照の記号は展開しません。",
  "Replaces matches by invoking a callback in source order.":
    "一致ごとに関数を呼び、その結果で順に置き換えます。",
  "Splits text at non-overlapping portable Regex matches.":
    "重複しない正規表現の一致を区切りとして、文字列を分割します。",
  "Reports a pattern that ends before its syntax is complete.":
    "正規表現の構文が完成する前に、パターンが終わっていることを表します。",
  "Reports an unexpected pattern character.":
    "パターンに想定外の文字があることを表します。",
  "Reports syntax intentionally excluded from the portable Regex contract.":
    "移植可能な正規表現では意図的に使えない構文を表します。",
  "Decodes UTF-8 and replaces invalid sequences.":
    "UTF-8を復号します。不正な並びは代替文字に置き換えます。",
  "Strictly decodes UTF-8 or reports the first invalid byte offset.":
    "UTF-8を厳密に復号します。不正なら最初のバイト位置を返します。",
  "Encodes String as UTF-8 Bytes.": "StringをUTF-8のBytesへ変換します。",
  "Reports the byte offset of the first invalid UTF-8 sequence.":
    "最初の不正なUTF-8の並びの、バイト位置を表します。",
  "Explicitly converts Left to one error and Right to Valid.":
    "Leftを一個のエラーを持つInvalid、RightをValidへ明示的に変換します。",
  "Constructs an invalid result from a NonEmptyList, preserving error order.":
    "NonEmptyListからInvalidを作り、エラーの順序を保ちます。",
  "Constructs an invalid result containing one error.":
    "エラー一個を含むInvalidを作ります。",
  "Explicitly converts Invalid to Left containing all errors, or Valid to Right.":
    "Invalidを全エラーを持つLeft、ValidをRightへ明示的に変換します。",
  "Independent validation with a Valid value or a non-empty, source-ordered collection of errors.":
    "独立した検証の結果です。成功はValidの値、失敗は元の順序を保つ一個以上のエラーで表します。",
  "Constructs a successful Validation value.": "検証に成功した値を作ります。",
}

export function japaneseReferenceDescription(description: string): string {
  const translated = japaneseDescriptions[description]
  if (translated === undefined) {
    throw new Error(`Missing Japanese API explanation: ${description}`)
  }
  return translated
}
