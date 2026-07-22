# 13. pure HTML treeとDOM renderer

## 13.1 目的とmodule境界

SeseragiのWeb UIは、props recordを受け取る純粋関数でimmutableなHTML treeを構築します。実DOM nodeの生成・更新、
event listener、focus、browser resourceはEffect境界へ置きます。component class、hook、hidden lifecycle、JSX専用構文を
言語へ追加しません。

- `std/web/html`: pureなHtml tree、props、event message、SSR。すべてのtargetで利用できる。
- `std/web/dom`: browser DOM rendererとDom service。DOM capabilityを持つtargetだけが提供する。

通常のfunctionがcomponentです。

```seseragi
fn card<Msg, C> title: String -> children: C -> html.Html<Msg>
where html.IntoChildren<C, Msg> =
  html.section {
    className: "card",
    children: [
      html.h2 { children: title },
      html.div { children }
    ]
  }
```

cardを呼ぶだけではDOM、global state、subscriptionへ触れません。同じ引数から観測可能に同じHtml treeを作ります。

## 13.2 Htmlとchildren

```seseragi
opaque type Html<Msg>

trait IntoChildren<C, Msg> {
  fn intoChildren value: C -> Array<Html<Msg>>
}

fn text<Msg> value: String -> Html<Msg>
fn fragment<Msg, C> children: C -> Html<Msg>
where IntoChildren<C, Msg>
```

Html<Msg>は将来DOM eventからMsgを生成しうるpure treeです。node identity、DOM handle、subscriptionを公開しません。
同じHtml valueを複数回renderしてもDOM nodeを共有しません。HtmlはEq、Ord、Hash、Show instanceを持たず、Debugは
event functionとsecret attribute valueを展開しません。

IntoChildrenのstandard instanceは次だけです。

- `Unit`: childなし。
- `String`: text node一件。
- `Html<Msg>`: node一件。
- `Array<Html<Msg>>`: source順。
- `List<Html<Msg>>`: source順。

String instanceは型parameterMsgについてparametricなので、eventを持たないtreeは周囲の期待Html<Msg>へ推論できます。
任意値をshowしてtextへ暗黙変換しません。数値などは `text $ show value` と明示します。Array内でStringとHtmlを
arbitrary unionにせず、混在する場合はStringを `text` で包みます。

## 13.3 props record

共通propsはoptional structural record fieldを使います。`children`だけはrequiredで、String、単一Html、Array、List、
UnitをIntoChildrenで正規化します。

```seseragi
alias ElementProps<Msg, C> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  onClick?: Msg,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onMouseDown?: MouseEvent -> EventAction<Msg>,
  onMouseUp?: MouseEvent -> EventAction<Msg>,
  onKeyDown?: KeyboardEvent -> EventAction<Msg>,
  children: C
}

alias ButtonProps<Msg, C> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  disabled?: Bool,
  buttonType?: String,
  onClick?: Msg,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onMouseDown?: MouseEvent -> EventAction<Msg>,
  onMouseUp?: MouseEvent -> EventAction<Msg>,
  onKeyDown?: KeyboardEvent -> EventAction<Msg>,
  children: C
}

alias InputProps<Msg> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  value?: String,
  checked?: Bool,
  name?: String,
  disabled?: Bool,
  required?: Bool,
  placeholder?: String,
  inputType?: String,
  onInput?: InputEvent -> Msg,
  onChange?: ChangeEvent -> Msg,
  onMouseDown?: MouseEvent -> EventAction<Msg>,
  onKeyDown?: KeyboardEvent -> EventAction<Msg>
}

alias TextareaProps<Msg> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  value?: String,
  name?: String,
  disabled?: Bool,
  required?: Bool,
  placeholder?: String,
  onInput?: InputEvent -> Msg,
  onChange?: ChangeEvent -> Msg
}

alias FormProps<Msg, C> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  onClick?: Msg,
  onSubmit?: Msg,
  children: C
}

alias LabelProps<Msg, C> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  htmlFor?: String,
  onClick?: Msg,
  children: C
}

alias AnchorProps<Msg, C> = {
  id?: String,
  className?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  href: WebUrl,
  target?: LinkTarget,
  rel?: String,
  onClick?: Msg,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onMouseDown?: MouseEvent -> EventAction<Msg>,
  onMouseUp?: MouseEvent -> EventAction<Msg>,
  children: C
}

type LinkTarget deriving Eq, Show =
  | SameContext
  | NewContext
```

省略fieldをNothingで埋めたrecordへ書き換えず、presenceを保ちます。`id: Nothing` はid省略ではなく、field型自体が
Maybeの場合のpresent値です。tag functionは受け取ったrecordを変更せず、normal evaluationと同じくfield式を
source順に一度だけ評価します。

standard tag functionは少なくとも次を提供します。

```seseragi
fn div<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn span<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn p<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn main<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn section<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn h1<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn h2<Msg, C> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn button<Msg, C> props: ButtonProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn form<Msg, C> props: FormProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn label<Msg, C> props: LabelProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
fn input<Msg> props: InputProps<Msg> -> Html<Msg>
fn textarea<Msg> props: TextareaProps<Msg> -> Html<Msg>
fn a<Msg, C> props: AnchorProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
```

ほかのHTML tagも同じ規則のtag固有propsを持ちます。void elementへchildren fieldを与えられません。通常のrecord
width subtypingは維持するため、既存recordが追加fieldを持っていてもtag functionへ渡せますが、rendererが読むのは
parameter型に宣言されたfieldだけです。fresh record literalにtag固有props外のfieldがあればSES-L0101 Warningを出し、
`clasName`のようなtypoを黙認しません。custom attributeはattributes field、custom elementは13.5のvalidated Tagを
使います。

## 13.4 event message

event propはEffectを実行せず、immutableなevent snapshotからMsgを作ります。

フォーム入力の公開snapshotは次です。どちらもruntimeだけが構築でき、user codeはfieldを読み取れますがhostの
`Event`、`target`、prototype、mutable stateは保持しません。

```seseragi
opaque struct InputEvent {
  value: String
}

opaque struct ChangeEvent {
  value: String,
  checked: Bool
}
```

`onInput`はtext inputとtextareaの現在valueを一度だけ読み、`InputEvent`をmapperへ渡します。`onChange`は同じ時点の
valueとcheckedを一度ずつ読み、`ChangeEvent`をmapperへ渡します。browser固有の`InputEvent` classは要求しないため、
iOS Safariを含む通常のbubbleする`input` / `change` eventを同じcontractで処理します。

`onSubmit: message`を持つformは、messageをqueueへ入れる前に同期的に`preventDefault`します。handlerがないformの
native submitは変更しません。SSRは`onInput`、`onChange`、`onSubmit`をattributeへ出力しません。

```seseragi
type EventAction<Msg> =
  | IgnoreEvent
  | Dispatch Msg
  | DispatchPreventDefault Msg
  | DispatchStopPropagation Msg
  | DispatchPreventDefaultAndStop Msg

struct MouseEvent deriving Eq, Show {
  button: Int,
  clientX: Float,
  clientY: Float,
  altKey: Bool,
  controlKey: Bool,
  metaKey: Bool,
  shiftKey: Bool
}

struct KeyboardEvent deriving Eq, Show {
  key: String,
  code: String,
  repeat: Bool,
  altKey: Bool,
  controlKey: Bool,
  metaKey: Bool,
  shiftKey: Bool
}
```

`onClick: message` はclick時にDispatch messageを作ります。preventClickDefault / stopClickPropagationはclickの
default actionとbubbleをmessage enqueue前に制御します。onKeyDownのmapperはEventActionを直接返します。event
snapshotはhost DOM Eventを保持せず、currentTarget、prototype、mutable fieldを公開しません。

rendererはEventActionを同期的に決定してpreventDefault / stopPropagationを適用し、その後DispatchされたMsgを13.9の
queueへ渡します。mapperがthrowする概念はなく、pure function defectは通常runtime defectです。
preventClickDefault / stopClickPropagationだけを指定してonClickがabsentなら動作せず、SES-L0101 Warningを出します。

## 13.5 safe tag、attribute、style、URL

```seseragi
type HtmlBuildError deriving Eq, Show =
  | InvalidTagName String
  | InvalidAttributeName String
  | ReservedAttributeName String
  | InvalidStyleProperty String
  | UnsafeWebUrlScheme String

opaque type Tag
opaque type Attribute
opaque type Style
opaque type WebUrl

fn customTag name: String -> Either<HtmlBuildError, Tag>
fn attribute name: String -> value: String
  -> Either<HtmlBuildError, Attribute>
fn style entries: Array<(String, String)> -> Either<HtmlBuildError, Style>
fn parseWebUrl value: String -> Either<HtmlBuildError, WebUrl>
fn custom<Msg, C> tag: Tag -> props: ElementProps<Msg, C> -> Html<Msg>
where IntoChildren<C, Msg>
```

TagはASCII lowercase letterで始まり、lowercase letter、digit、`-`だけを持ちます。custom element名は少なくとも
一つ`-`を含みます。attribute nameはHTML ASCII nameですが、`on`で始まるname、`style`、`class`、`id`、rendererが
使う内部nameをReservedAttributeNameとして拒否します。event listenerをString attributeとして注入できません。

Style propertyはlowercase CSS propertyまたは`--`で始まるcustom propertyです。entry順を保ち、重複propertyは最後の
値を採用して最初の位置を保ちます。StyleはCSS sanitizerではありません。security-sensitive URLはWebUrl propを使い、
parseWebUrlはrelative URLと`http`、`https`、`mailto`、`tel`だけを受理し、control character、userinfo、
`javascript`、`data`、`file` schemeを拒否します。

raw HTML Stringをtreeへ挿入する標準operationは提供しません。sanitizer packageは独自opaque TrustedHtmlと明示nodeを
提供できますが、Stringからのunchecked constructorをstd/web/htmlへ置きません。

## 13.6 pure tree semantics

Html treeはnamespace、tag、normalized props、ordered childrenを持ちます。textはStringを保持し、tree構築時にはescape
しません。SSRまたはDOM backendがsinkに合わせ一度だけescape / textContent化します。component functionの境界は
runtime treeに残らず、component local stateやmount hookを暗黙生成しません。

keyはparent直下のsibling identity hintで、HTML attributeではありません。同じsibling listのpresent keyは一意で
なければなりません。pure treeとSSRはkeyを保持・無視できますが、DOM mount / updateはtree全体を変更前に検査し、
重複をDuplicateSiblingKeyとして拒否します。keyがないnodeのidentityは同じparent内のunkeyed相対位置です。keyを
global ID、CSS selector、component identityとして使いません。

## 13.7 SSR

```seseragi
fn renderToString<Msg> tree: Html<Msg> -> String
fn renderDocument<Msg> tree: Html<Msg> -> String
```

renderToStringはfragmentをHTML textへ、renderDocumentはASCII lowercase `<!doctype html>`を先頭へ一度加えます。
textでは`&`、`<`、`>`、attributeではさらにquoteをescapeします。Unicode scalarはUTF-8で保持し、同じtreeから同じ
bytesを生成します。

attribute順はtag固有propsの宣言順、続いてattributes Array順です。absent prop、event prop、keyは出力しません。
Bool attributeはTrueならnameだけ、False / absentなら省略します。classNameは`class`、Styleはproperty順のcanonical
Stringです。void elementはend tagを出さず、非void elementはchildrenが空でも開始・終了tagを出します。
buttonType absentは安全な`"button"`、inputType absentは`"text"`として`type`属性へ出します。
NewContext linkは `target="_blank"` とし、relにnoopenerがなければ自動で追加します。

SSRはeventを実行せずDOM serviceを要求しません。render結果を再parseしても同じHTML tree semanticsを持ちますが、
browserが行うtable補正などHTML parser固有normalizationが必要な構造はhydration時に13.11の規則で検査します。

## 13.8 Dom serviceとtarget

```seseragi
type DomError deriving Eq, Show =
  | InvalidSelector String
  | DomTargetNotFound String
  | DomTargetAlreadyMounted
  | DuplicateSiblingKey String
  | HydrationMismatch { path: Array<Int>, expected: String, actual: String }
  | ManagedDomMutated { path: Array<Int> }
  | DomEventQueueOverflow Int
  | DomTargetRemoved
  | DomOperationFailed String

type DomRuntimeError<E> =
  | DomFailure DomError
  | DispatchFailure E

type HydrationMode deriving Eq, Show =
  | FreshMount
  | HydrateStrict
  | HydrateOrReplace

type CleanupMode deriving Eq, Show =
  | ClearRenderedDom
  | PreserveRenderedDom

struct DomOptions deriving Eq, Show {
  eventCapacity: BufferCapacity,
  hydration: HydrationMode,
  cleanup: CleanupMode
}

opaque type DomTarget
opaque type DomMount<E>

fn defaultOptions -> DomOptions
fn query selector: String
  -> Effect<{ dom: Dom }, DomError, DomTarget>
fn mount<R, E, Msg>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Msg -> Effect<R, E, Unit>)
  -> content: Signal<Html<Msg>>
  -> Effect<R & { dom: Dom }, DomError, DomMount<E>>
fn awaitMount<E> mount: DomMount<E>
  -> Task<DomRuntimeError<E>, Unit>
fn unmount<E> mount: DomMount<E> -> Task<Never, Unit>
fn run<R, E, Msg>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Msg -> Effect<R, E, Unit>)
  -> content: Signal<Html<Msg>>
  -> Effect<R & { dom: Dom }, DomRuntimeError<E>, Unit>
fn app<State, Msg>
  config: {
    target: String,
    initial: State,
    update: Msg -> State -> State,
    view: State -> Html<Msg>
  }
  -> Effect<{ dom: Dom }, String, Unit>
```

Dom serviceのcanonical requirement名は`dom`で、`with Dom`は`with dom: Dom`へ展開します。queryは現在documentの
CSS selectorを一度評価し、最初のElementだけをtargetにします。0件はDomTargetNotFound、不正selectorは
InvalidSelectorです。DomTargetはhost Element identityを保持するopaque capabilityで、foreign objectとしてfield access
できません。

defaultOptionsはeventCapacity 1024、FreshMount、ClearRenderedDomです。mountはinitial Signal snapshotとsubscription
登録をatomicに行い、initial render完了後にresource登録済みDomMountを返します。同じtargetへ同時に二mountできません。
runはmount後にawaitMountし、終了時にunmountするconvenienceです。
initial treeのkey・prop・void-element invariantは既存DOMを変更する前に検査し、validation failureでpartial mountを
残しません。

appはpure reducerで完結する通常のapplication向けconvenienceです。内部でMutableSignalを一つ作り、viewをmapし、
targetのquery、defaultOptionsによるrun、messageごとのSignal更新を所有します。query / runtime failureは実行可能な
compact mainがそのまま推論できるString failureへ正規化します。effectful dispatch、custom options、mount後の値や
終了理由が必要なprogramはquery / runまたはmount / awaitMount / unmountを直接使います。appはcompiler構文ではなく
`std/web/dom`の通常関数であり、StateやMsgごとのcompiler hardcodeを持ちません。

## 13.9 event dispatchとresource lifetime

mountはmanaged subtreeのevent listener、Signal subscription、dispatch Fiber、event queueを現在Effect scopeへ登録します。
success、failure、cancellationのすべてで新規event受付を止め、queueとdispatch Fiberをcancelし、listenerとsubscriptionを
解除してから終了します。unmountはidempotentです。CleanupModeに従いmanaged childを削除または最終DOMを残しますが、
どちらもlistenerを残しません。

Dispatch messageはhost観測順のbounded FIFOへ入り、一件ずつdispatch Effectを完了してから次へ進みます。並列event
handlerを暗黙起動しません。満杯時はmessageを捨てずDomEventQueueOverflowでmountを失敗させます。dispatchがEで
失敗するとDispatchFailureでmountを終了し、未処理messageを破棄してresource cleanupします。

awaitMountはtargetがdocumentから外れた場合DomTargetRemovedで失敗します。明示unmount後はUnit successです。mountを
awaitせず外側scopeが閉じてもfinalizerが同じcleanupを行います。browser page自体のforced terminationだけは保証外です。

## 13.10 Signal renderとreconciliation

content Signalはsubscription時のsnapshotをinitial treeにし、以後glitch-free transactionが公開する安定treeごとに
一回reconcileします。同じtransaction中の中間値をrenderしません。render中に次transactionが完了した場合は順序を
保って最新treeまで進め、古いtreeへ戻しません。

reconciliationは次で固定します。

- namespaceとtagが同じnodeは再利用し、異なればsubtreeをreplaceする。
- keyed siblingはkeyで対応し、unkeyed siblingはunkeyed同士の相対indexで対応する。
- keyed nodeの移動はDOM identityとfocusを可能な限り保持する。
- textはtextContent、attributeは対応attribute、input value / checkedはDOM propertyを更新する。
- event mapper変更はlistener resourceを置換するが、nodeを置換しない。
- 削除nodeのlistenerとchild resourceを親から子の順に新規event受付停止、子から親の順にreleaseする。

renderer以外がmanaged subtreeを変更することはcontract外です。次reconcileで期待identityと一致しなければ
ManagedDomMutatedとしてmountを終了し、未知nodeを「たぶん同じ」と採用しません。focus、selection、scrollのbrowser
stateはnode再利用時に保持し、node replace時には保証しません。controlled inputのvalue更新で同じStringならpropertyを
書き直さずselectionを保ちます。

## 13.11 hydration

FreshMountはtargetの既存childを削除してinitial treeをrenderします。HydrateStrictは既存DOMをinitial Html treeと
照合し、tag、namespace、text、typed propが一致するnodeを再利用します。eventとkeyはSSR出力に存在しないため、tree側
からlistenerと将来identityを登録します。browserが隣接text nodeをmergeしている場合は同じ連結textなら境界をsplitして
再利用できます。

mismatch時、HydrateStrictは最初のpathをHydrationMismatchとして返して既存DOMを変更しません。
HydrateOrReplaceは一致したancestorを保ち、最小の不一致subtreeをinitial treeで置換します。どちらも不一致を黙って
成功扱いせず、replace modeはdiagnosticへpathを一件記録します。hydration完了前にevent listenerを有効化せず、途中failure
で半分だけinteractiveなtreeを残しません。

## 13.12 targetとinterop

std/web/htmlの意味はTypeScript DOM型へ依存しません。TypeScript backendはHtmlをplain host objectとして公開せず、
7.12のopaque ABI wrapperを使います。Dom service adapterだけがElement、Event、Nodeへアクセスします。SSR target、test
target、browser targetは同じHtml treeとescape規則を共有します。

test adapterはin-memory DOM、synthetic event、focus、listener leak、Signal transaction回数を観測可能にします。実browser
conformanceは少なくともChromium系一つだけに意味を委譲せず、HTML parsing / event順が標準contractと一致する複数engine
fixtureを持ちます。SVG、MathML、custom rendererは同じpure tree原則を使う別moduleで、HTML tagへnamespaceを暗黙混在
させません。

project fixtureのschema 1 DOM scenarioは、target adapterへ次を渡します。

```text
dom.document: test開始前のUTF-8 HTML document
dom.events: mount安定後に順番に送るsynthetic event
  selector: target document内の一意なCSS selector
  type: click | input | change | keydown | mousedown | mouseup
  value: input / change時のtarget String value、その他は省略
  keyboard / mouse: 対応snapshot field、対象eventだけで指定
  afterHtml: dispatchと対応reconcileが安定した後のdocument HTML
dom.expected.mountHtml: initial mount / hydration成功後のdocument HTML。成功を期待するscenarioだけ指定
dom.expected.finalHtml: root Effect終了と全cleanup完了後のdocument HTML
dom.expected.reconciliations: initial renderを除くreconcile回数
dom.expected.activeListeners: cleanup後のlistener数
dom.expected.activeSubscriptions: cleanup後のSignal subscription数
```

eventsは一件のdispatch Effectと、それが起こした全Signal transaction / reconcileが安定してから次へ進みます。
selectorが0件または複数件、event fieldの型違い、afterHtml不一致、program終了前の未送信event、expected resource数の
不一致はfixture failureです。programがevent処理中に終了した場合はそのeventを完了させてcleanupを待ち、後続eventを
送信しません。real browser、network、wall clockへfallbackせず、scenario外のhost mutationを生成しません。
