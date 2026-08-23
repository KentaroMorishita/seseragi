# 13. pure HTML treeとDOM renderer

## 13.1 目的とmodule境界

SeseragiのWeb UIは、props recordを受け取る純粋関数でimmutableなHTML treeを構築します。実DOM nodeの生成・更新、
event listener、focus、browser resourceはEffect境界へ置きます。component class、hook、hidden lifecycle、JSX専用構文を
言語へ追加しません。

- `std/web/html`: pureなHtml tree、props、event Action、SSR。すべてのtargetで利用できる。
- `std/web/dom`: browser DOM rendererとDom service。DOM capabilityを持つtargetだけが提供する。

Web UIで発生した操作・意図を表す型parameterは`Action`と呼びます。このgeneric parameter名の変更は
`Html`や`dom.app`の型identity・runtime ABIを変更しません。standard surfaceに旧用語を含む公開symbolはないため、
user programのprivate ADTは互換aliasなしで`Action`へ移行できます。

通常のfunctionがcomponentです。

```seseragi
fn card<Action, C> title: String -> children: C -> html.Html<Action>
where html.IntoChildren<C, Action> =
  html.section {
    class: "card",
    children: [
      html.h2 { children: title },
      html.div { children }
    ]
  }
```

cardを呼ぶだけではDOM、global state、subscriptionへ触れません。同じ引数から観測可能に同じHtml treeを作ります。

## 13.2 Htmlとchildren

```seseragi
opaque type Html<Action>

trait IntoChildren<C, Action> {
  fn intoChildren value: C -> Array<Html<Action>>
}

fn text<Action> value: String -> Html<Action>
fn fragment<Action, C> children: C -> Html<Action>
where IntoChildren<C, Action>
```

Html<Action>は将来DOM eventからActionを生成しうるpure treeです。node identity、DOM handle、subscriptionを公開しません。
同じHtml valueを複数回renderしてもDOM nodeを共有しません。HtmlはEq、Ord、Hash、Show instanceを持たず、Debugは
event functionとsecret attribute valueを展開しません。

IntoChildrenのstandard instanceは次だけです。

- `Unit`: childなし。
- `String`: text node一件。
- `Html<Action>`: node一件。
- `Array<Html<Action>>`: source順。
- `List<Html<Action>>`: source順。

String instanceは型parameter Actionについてparametricなので、eventを持たないtreeは周囲の期待Html<Action>へ推論できます。
任意値をshowしてtextへ暗黙変換しません。数値などは `text $ show value` と明示します。Array内でStringとHtmlを
arbitrary unionにせず、混在する場合はStringを `text` で包みます。

## 13.3 props record

共通propsはoptional structural record fieldを使います。`children`だけはrequiredで、String、単一Html、Array、List、
UnitをIntoChildrenで正規化します。

HTMLのclass属性はSeseragiでも`class`をcanonical field名とし、rendererが同名のHTML属性へ出力します。
0.4.1では0.4.0の`className`を互換aliasとして残さないbreaking migrationを行ったため、旧sourceは
`className: value`を`class: value`へ置き換えます。

```seseragi
alias ElementProps<Action, C> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>,
  children: C
}

alias ButtonProps<Action, C> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  disabled?: Bool,
  buttonType?: String,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>,
  children: C
}

alias InputProps<Action> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  value?: String,
  checked?: Bool,
  name?: String,
  disabled?: Bool,
  required?: Bool,
  placeholder?: String,
  inputType?: String,
  onInput?: InputEvent -> Action,
  onChange?: ChangeEvent -> Action,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>
}

alias TextareaProps<Action> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  value?: String,
  name?: String,
  disabled?: Bool,
  required?: Bool,
  placeholder?: String,
  onInput?: InputEvent -> Action,
  onChange?: ChangeEvent -> Action,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>
}

alias FormProps<Action, C> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>,
  onSubmit?: Action,
  children: C
}

alias LabelProps<Action, C> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  htmlFor?: String,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>,
  children: C
}

alias AnchorProps<Action, C> = {
  id?: String,
  class?: String,
  title?: String,
  hidden?: Bool,
  key?: String,
  style?: Style,
  attributes?: Array<Attribute>,
  role?: String,
  tabIndex?: Int,
  lang?: String,
  dir?: String,
  draggable?: Bool,
  contentEditable?: Bool,
  href: WebUrl,
  target?: LinkTarget,
  rel?: String,
  onClick?: Action,
  preventClickDefault?: Bool,
  stopClickPropagation?: Bool,
  onFocus?: Action,
  onBlur?: Action,
  onKeyDown?: KeyboardEvent -> EventAction<Action>,
  onKeyUp?: KeyboardEvent -> EventAction<Action>,
  onMouseDown?: MouseEvent -> EventAction<Action>,
  onMouseUp?: MouseEvent -> EventAction<Action>,
  onPointerDown?: PointerEvent -> EventAction<Action>,
  onPointerUp?: PointerEvent -> EventAction<Action>,
  onDoubleClick?: MouseEvent -> EventAction<Action>,
  onContextMenu?: MouseEvent -> EventAction<Action>,
  onScroll?: ScrollEvent -> EventAction<Action>,
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
fn div<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn span<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn p<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn main<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn section<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn h1<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn h2<Action, C> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn button<Action, C> props: ButtonProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn form<Action, C> props: FormProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn label<Action, C> props: LabelProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
fn input<Action> props: InputProps<Action> -> Html<Action>
fn textarea<Action> props: TextareaProps<Action> -> Html<Action>
fn a<Action, C> props: AnchorProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
```

ほかのHTML tagも同じ規則のtag固有propsを持ちます。void elementへchildren fieldを与えられません。通常のrecord
width subtypingは維持するため、既存recordが追加fieldを持っていてもtag functionへ渡せますが、rendererが読むのは
parameter型に宣言されたfieldだけです。fresh record literalにtag固有props外のfieldがあればSES-L0101 Warningを出し、
`clasName`のようなtypoを黙認しません。custom attributeはattributes field、custom elementは13.5のvalidated Tagを
使います。fresh record literalでrequired propが欠けていればSES-T0702 Errorを出します。spreadを含むliteralは
spread元がrequired propを供給できるため、通常のrecord型検査で判定します。`preventClickDefault`または
`stopClickPropagation`だけを指定して`onClick`がなければSES-L0101 Warningを出します。parser recovery中は
これらのsemantic diagnosticを追加しません。

## 13.4 event Action

event propはEffectを実行せず、immutableなevent snapshotからActionを作ります。

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
valueとcheckedを一度ずつ読み、`ChangeEvent`をmapperへ渡します。公開snapshotはbrowser固有のevent objectや
`isComposing`を露出せず、iOS Safariを含む通常のbubbleする`input` / `change` eventを同じcontractで処理します。

### 13.4.1 IME composition

text inputとtextareaの`onInput`は日本語IMEなどのcompositionをruntime内で透過的に処理します。利用者が
composition eventを個別に配線する必要はありません。

- `compositionstart`から`compositionend`までは途中の`input`をActionへ変換しない。
- native `InputEvent.isComposing`がtrueなら、`compositionstart`を先に観測できないbrowserでもcomposition中として扱う。
- `compositionend`の前後に最終`input`が発生するbrowser差を同じsessionへまとめ、確定したvalueを一度だけActionへ変換する。
- composition中と確定待ちの間はcontrolled rerenderを保留し、active control、native composition、caret、selectionを保持する。
- 外部Signal更新と競合した場合は最新treeだけを保留し、native確定valueのActionを先にdispatchしてからrenderする。
- handler tableとDOM markerは確定まで同じsnapshotを使い、確定後のrenderで同時に更新する。
- composition中にsubmitされた場合は、未確定controlの最終Input Actionをsource順にqueueへ入れてからSubmit Actionをdispatchする。

通常の英数字入力、paste、delete、autofillはcomposition sessionを作らず、従来どおり各`input`を直ちにActionへ
変換します。公開`InputEvent { value }`は維持し、`isComposing`はruntime内部の制御情報だけに使います。

`onSubmit: action`を持つformは、Actionをqueueへ入れる前に同期的に`preventDefault`します。handlerがないformの
native submitは変更しません。SSRは`onInput`、`onChange`、`onSubmit`をattributeへ出力しません。

```seseragi
opaque struct KeyboardEvent {
  key: String,
  code: String,
  repeat: Bool,
  altKey: Bool,
  controlKey: Bool,
  metaKey: Bool,
  shiftKey: Bool
}

opaque struct MouseEvent {
  button: Int,
  clientX: Float,
  clientY: Float,
  altKey: Bool,
  controlKey: Bool,
  metaKey: Bool,
  shiftKey: Bool
}

opaque struct PointerEvent {
  pointerId: Int,
  pointerType: String,
  isPrimary: Bool,
  button: Int,
  clientX: Float,
  clientY: Float,
  pressure: Float,
  altKey: Bool,
  controlKey: Bool,
  metaKey: Bool,
  shiftKey: Bool
}

opaque struct ScrollEvent {
  scrollLeft: Float,
  scrollTop: Float
}

type EventAction<Action> =
  | IgnoreEvent
  | Dispatch Action
  | DispatchPreventDefault Action
  | DispatchStopPropagation Action
  | DispatchPreventDefaultAndStop Action
```

`onFocus: action`と`onBlur: action`は、bubbleする`focusin` / `focusout`をruntime内で正規化してActionをqueueへ
渡します。`onKeyDown`と`onKeyUp`のmapperは、native keyboard eventから一度だけ読み取った`KeyboardEvent`を受け取り、
`EventAction<Action>`を返します。`onMouseDown`、`onMouseUp`、`onDoubleClick`、`onContextMenu`は`MouseEvent`、
`onPointerDown`と`onPointerUp`は`PointerEvent`、`onScroll`はeventのcurrent targetから読んだ`ScrollEvent`を同じ契約で
mapperへ渡します。snapshotはhost DOM Eventを保持せず、currentTarget、prototype、mutable fieldを公開しません。

`pointerType`はbrowserのPointer Events contractを保った`"mouse"`、`"touch"`、`"pen"`を通常値とします。runtimeは
pointer eventをmouse eventへ合成し直さず、同じpointer snapshot contractをdesktop browserとiOS Safariで使います。
pointer eventを提供しないbrowserではpointer handlerは発火せず、click / mouse handlerのcontractは独立して維持します。
scroll eventはbubbleしないためDOM rootのcapture listenerで観測しますが、mapperへ渡すoffsetはmarkerを持つcurrent targetの値です。

event mapperはnative listener内で同期的に一度だけ評価します。結果が`IgnoreEvent`ならbrowser制御もAction dispatchも行いません。
ほかのconstructorでは、指定された`preventDefault`、`stopPropagation`の順に同期実行してからActionをqueueへ入れます。
`onClick: action`は互換性のため直接Actionを受け取り、`preventClickDefault`と`stopClickPropagation`の省略値はfalseです。
これらのclick制御fieldは`onClick`が存在するときだけ使います。`onSubmit`は従来どおり常にdefaultを同期的に防ぎます。
Actionとして`Task<Unit>`を使う場合も同じ契約です。SSRはevent handlerや制御fieldをattributeへ出力しません。

## 13.5 safe tag、attribute、style、URL

```seseragi
type HtmlBuildError deriving Eq, Show, Debug =
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
fn custom<Action, C> tag: Tag -> props: ElementProps<Action, C> -> Html<Action>
where IntoChildren<C, Action>
```

TagはASCII lowercase letterで始まり、lowercase letter、digit、`-`だけを持ちます。custom element名は少なくとも
一つ`-`を含みます。attribute nameはHTML ASCII nameですが、`on`で始まるname、`style`、`class`、`id`、rendererが
使う内部nameをReservedAttributeNameとして拒否します。event listenerをString attributeとして注入できません。
`data-*`と`aria-*`はlowercaseで空でないsuffixを要求し、ほかのcustom attributeと同じくAttributeへ検証してから
`attributes`へ渡します。typed propまたはrenderer内部attributeと同じnameは大小文字を区別せず衝突として拒否します。

Style propertyはlowercase CSS propertyまたは`--`で始まるcustom propertyです。entry順を保ち、重複propertyは最後の
値を採用して最初の位置を保ちます。StyleはCSS sanitizerではありません。security-sensitive URLはWebUrl propを使い、
parseWebUrlはrelative URLと`http`、`https`、`mailto`、`tel`だけを受理し、control character、userinfo、
`javascript`、`data`、`file` schemeを拒否します。

`a`と`link`の`href`、`img`と`source`の`src`、`video`と`audio`のoptionalな`src`はすべて`WebUrl`です。
これらのfieldへ`String`を直接渡せず、`parseWebUrl`の`Right`だけを使用できます。relative URLにはroot-relative、
path-relative、query、fragment、scheme-relative URLを含みます。authorityを持つURLはscheme-relativeであっても
usernameまたはpasswordを含められません。ASCII control characterとDELを含む値、構文としてURLに解釈できない値、
allowlist外のschemeは元の入力を持つ`UnsafeWebUrlScheme`になります。受理時は入力の綴りを保持し、SSR / DOM backendが
attribute sinkへ出す段階で通常のattribute escapingを一度だけ適用します。

raw HTML Stringをtreeへ挿入する標準operationは提供しません。sanitizer packageは独自opaque TrustedHtmlと明示nodeを
提供できますが、Stringからのunchecked constructorをstd/web/htmlへ置きません。

## 13.6 pure tree semantics

Html treeはnamespace、tag、normalized props、ordered childrenを持ちます。textはStringを保持し、tree構築時にはescape
しません。SSRまたはDOM backendがsinkに合わせ一度だけescape / textContent化します。component functionの境界は
runtime treeに残らず、component local stateやmount hookを暗黙生成しません。

keyはparent直下のstructural regionがidentityを対応付けるためのhintで、HTML attributeではありません。pure treeと
SSRはkeyを保持または無視できます。keyの一意性、keyed nodeの移動、keyがないnodeの対応規則はglobal Html treeの
semanticsではなく、reactive structural regionを定義する13.10の拡張surfaceが所有します。keyをglobal ID、CSS
selector、component identity、component stateの暗黙identityとして使いません。

### 13.6.1 stateful featureのmodule所有境界

stateful featureもcomponent専用構文を追加せず、通常のEffect functionをconstructorとして表現します。constructorは
module-privateな`MutableSignal<State>`を一度生成し、親へはread-onlyな
`Signal<Html<Effect<{}, Never, Unit>>>`を返せます。privateなState、Action、updateはそのmoduleに留まり、event Actionは
feature自身がSignal更新Effectへ変換します。root DOM runtimeはこのopaqueなEffectを実行するだけで、子のAction variantを
列挙するroot Actionやdispatch分岐を要求しません。

```seseragi
effect fn create label: String
  -> Signal<html.Html<Effect<{}, Never, Unit>>>
fails Never =
  do {
    state <- signals.make $ CounterState { count: 0 }
    succeed $ signals.map (view label state) state
  }
```

stateの所有者は次で固定します。

- local stateはfeature constructorが所有し、private Actionだけが更新する。
- shared stateは最も近い共通親featureが所有し、子へread-only Signalまたは許可したEffectだけを渡す。
- app-wide stateはroot featureが所有し、`main`やmodule import時のglobal singletonにしない。

feature identityはconstructorが返したSignal bindingに所属します。component functionの呼び出し順やHtml nodeの`key`から
stateを暗黙生成しません。条件表示はSignal graphでactiveな値を選びます。DOM上のreactive leaf / structural regionへの
bindingと、regionを切り替えた際のDOM identity規則は13.10の拡張surfaceが所有し、component call順からは導出しません。

この最小surfaceの子eventはempty requirementかつ`Never` failureのEffectなので、子固有のresource requirementやfailure型を
親へ展開しません。rootの`dom.run`終了時はcontent subscriptionとevent listenerを既存のDOM resource境界で解除します。
独自resourceを持つchildのmount / unmount lifetimeは13.8〜13.10の`DomMount` ownershipへ結び付け、nodeの`key`を
feature resource scopeとして流用しません。pure function componentと単一State + Action用の`dom.app`はこの構成と
併存します。

## 13.7 SSR

```seseragi
fn renderToString<Action> tree: Html<Action> -> String
fn renderDocument<Action> tree: Html<Action> -> String
```

renderToStringはfragmentをHTML textへ、renderDocumentはASCII lowercase `<!doctype html>`を先頭へ一度加えます。
textでは`&`、`<`、`>`、attributeではさらにquoteをescapeします。Unicode scalarはUTF-8で保持し、同じtreeから同じ
bytesを生成します。

attribute順はtag固有propsの宣言順、続いてattributes Array順です。absent prop、event prop、keyは出力しません。
Bool attributeはTrueならnameだけ、False / absentなら省略します。classは`class`、Styleはproperty順のcanonical
Stringです。void elementはend tagを出さず、非void elementはchildrenが空でも開始・終了tagを出します。
buttonType absentは安全な`"button"`、inputType absentは`"text"`として`type`属性へ出します。
NewContext linkは `target="_blank"` とし、relにnoopenerがなければ自動で追加します。

SSRはeventを実行せずDOM serviceを要求しません。render結果を再parseしても同じHTML tree semanticsを持ちますが、
browserが行うtable補正などHTML parser固有normalizationが必要な構造はhydration時に13.11の規則で検査します。

## 13.8 Dom serviceとtarget

```seseragi
type DomError deriving Eq, Show, Debug =
  | InvalidSelector String
  | DomTargetNotFound String
  | DomTargetAlreadyMounted
  | HydrationMismatch { path: Array<Int>, expected: String, actual: String }
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
  eventCapacity: Int,
  hydration: HydrationMode,
  cleanup: CleanupMode
}

opaque type DomTarget
opaque type DomMount<E>
opaque type DomContent<Action>
opaque type DomBinding<Action>

fn defaultOptions -> DomOptions
fn query selector: String
  -> Effect<{ dom: Dom }, DomError, DomTarget>
fn mount<R, E, Action>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Action -> Effect<R, E, Unit>)
  -> content: Signal<Html<Action>>
  -> Effect<R & { dom: Dom }, DomError, DomMount<E>>
fn awaitMount<E> mount: DomMount<E>
  -> Effect<{}, DomRuntimeError<E>, Unit>
fn unmount<E> mount: DomMount<E> -> Task<Unit>
fn run<R, E, Action>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Action -> Effect<R, E, Unit>)
  -> content: Signal<Html<Action>>
  -> Effect<R & { dom: Dom }, DomRuntimeError<E>, Unit>
fn content<Action>
  initial: Html<Action>
  -> bindings: Array<DomBinding<Action>>
  -> DomContent<Action>
fn initialHtml<Action> content: DomContent<Action> -> Html<Action>
fn bindText<Action>
  selector: String -> source: Signal<String> -> DomBinding<Action>
fn bindAttribute<Action>
  selector: String
  -> name: String
  -> source: Signal<Maybe<String>>
  -> DomBinding<Action>
fn bindValue<Action>
  selector: String -> source: Signal<String> -> DomBinding<Action>
fn bindChecked<Action>
  selector: String -> source: Signal<Bool> -> DomBinding<Action>
fn bindStyle<Action>
  selector: String
  -> name: String
  -> source: Signal<Maybe<String>>
  -> DomBinding<Action>
fn bindRegion<Action>
  selector: String
  -> source: Signal<DomContent<Action>>
  -> DomBinding<Action>
fn mountContent<R, E, Action>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Action -> Effect<R, E, Unit>)
  -> content: DomContent<Action>
  -> Effect<R & { dom: Dom }, DomError, DomMount<E>>
fn runContent<R, E, Action>
  options: DomOptions
  -> target: DomTarget
  -> dispatch: (Action -> Effect<R, E, Unit>)
  -> content: DomContent<Action>
  -> Effect<R & { dom: Dom }, DomRuntimeError<E>, Unit>
fn app<State, Action>
  config: {
    target: String,
    initial: State,
    update: Action -> State -> State,
    view: State -> Html<Action>
  }
  -> Effect<{ dom: Dom }, String, Unit>
```

`DomRuntimeError<E>`は`Show<E>`があるときShow、`Debug<E>`があるときDebugを条件付きで
合成します。`E = Never`では標準の到達不能evidenceを使うため、DOM runtime自身が返す
`DomFailure`は明示的なuserland instanceなしで表示できます。`DispatchFailure`の`Never`
payloadは型上到達不能です。`DomTarget`、`DomMount<E>`、`Html<Action>`、`Attribute`等のopaque handleには
Show / Debugを提供せず、表示を要求した箇所で`SES-T0201`になります。

Dom serviceのcanonical requirement名は`dom`で、`with Dom`は`with dom: Dom`へ展開します。queryは現在documentの
CSS selectorを一度評価し、最初のElementだけをtargetにします。0件はDomTargetNotFound、不正selectorは
InvalidSelectorです。DomTargetはhost Element identityを保持するopaque capabilityで、foreign objectとしてfield access
できません。

defaultOptionsはeventCapacity 1024、FreshMount、ClearRenderedDomです。mountはinitial Signal snapshotとsubscription
登録をatomicに行い、initial render完了後にresource登録済みDomMountを返します。同じtargetへ同時に二mountできません。
runはmount後にawaitMountし、終了時にunmountするconvenienceです。
initial treeのprop・void-element invariantは既存DOMを変更する前に検査し、validation failureでpartial mountを
残しません。mountが所有するlistener、Signal subscription、IME timer、後続のreactive bindingは選択したrender
algorithmにかかわらず同じ`DomMount`へ登録します。

appはpure reducerで完結する通常のapplication向けconvenienceです。内部でMutableSignalを一つ作り、viewをmapし、
targetのquery、defaultOptionsによるrun、ActionごとのSignal更新を所有します。query / runtime failureは実行可能な
compact mainがそのまま推論できるString failureへ正規化します。effectful dispatch、custom options、mount後の値や
終了理由が必要なprogramはquery / runまたはmount / awaitMount / unmountを直接使います。appはcompiler構文ではなく
`std/web/dom`の通常関数であり、StateやActionごとのcompiler hardcodeを持ちません。

## 13.9 event dispatchとresource lifetime

mountはmanaged subtreeのevent listener、Signal subscription、dispatch Fiber、event queueを現在Effect scopeへ登録します。
success、failure、cancellationのすべてで新規event受付を止め、queueとdispatch Fiberをcancelし、listenerとsubscriptionを
解除してから終了します。unmountはidempotentです。CleanupModeに従いmanaged childを削除または最終DOMを残しますが、
どちらもlistenerを残しません。

DispatchされたActionはhost観測順のbounded FIFOへ入り、一件ずつdispatch Effectを完了してから次へ進みます。並列event
handlerを暗黙起動しません。満杯時はActionを捨てずDomEventQueueOverflowでmountを失敗させます。dispatchがEで
失敗するとDispatchFailureでmountを終了し、未処理Actionを破棄してresource cleanupします。

awaitMountはtargetがdocumentから外れた場合DomTargetRemovedで失敗します。明示unmount後はUnit successです。mountを
awaitせず外側scopeが閉じてもfinalizerが同じcleanupを行います。browser page自体のforced terminationだけは保証外です。

## 13.10 Signal-driven DOM bindingと更新境界

`mount`の`content: Signal<Html<Action>>`はsubscription時のsnapshotをinitial treeとして使い、initial DOMとevent
bindingを接続する互換surfaceです。以後のstable publicationをcoarse content updateとして扱う実装は許されますが、
publicationごとにroot Html tree全体を再生成してwhole-tree reconciliationすることを言語semanticsとして要求しません。
同じtransaction中の中間値をDOMへ公開せず、処理中に次のpublicationが到着した場合は順序を保って最新値まで進めます。

canonical surfaceは、pure Html / SSRを維持したまま`DomContent<Action>`で明示的なinitial snapshotとbinding planを
組にします。`content initial bindings`はSignalをreadせず、`initial`をそのまま保持します。`initialHtml`は同じpure
`Html<Action>`を返すため、serverは必要なSignal snapshotをEffectで明示取得してinitial Htmlを組み立て、通常の
`renderToString` / `renderDocument`へ渡せます。client hydrationは同じserialized stateからDomContentを再構築し、
initial Html照合後にbindingを接続します。Html値の内部へSignal、subscription、host Nodeを格納しません。

更新単位は次の三種類です。

- static DOM: mount後に値更新を購読しないclosed subtree。
- reactive leaf binding: `bindText`、`bindAttribute`、`bindValue`、`bindChecked`、`bindStyle`が一つのsinkへ
  Signal値を反映するbinding。
- structural region: `bindRegion`が条件分岐やcollection等、指定Elementのchild構造を所有するregion。

各selectorは現在のDomContent scopeのroot Elementから相対評価し、exactly oneのdescendant Elementへ解決します。不正、
0件、複数件、binding種別とElementの不一致、invalid attribute / style nameは`DomOperationFailed`でmountContentを失敗
させ、途中で登録したbindingを解除します。selectorはscope外へ出ず、region内の同名selectorは親scopeとidentityを共有
しません。

bindTextは対象Elementのtext content、bindAttributeは指定attribute、bindValueはinput / textarea / selectのvalue
property、bindCheckedはinputのchecked property、bindStyleは一つのCSS propertyだけを所有します。MaybeのNothingは
attribute / style propertyの削除です。他のattribute、property、style、child、static siblingを書き換えません。現在DOMと
同値ならwriteしません。bindValueはIME composition中のhost valueを上書きせず、composition commit後のstable valueまで
適用を遅延します。leaf updateはElement identity、focus、selection、event listenerを置換しません。

bindRegionのSignal値は入れ子の`DomContent<Action>`です。region target Element自身は置換せず、そのchildrenだけを
current contentのinitial Htmlへ切り替え、入れ子bindingとevent handlerを同じscopeへ接続します。切替時は旧contentの
subscriptionとevent bindingを解除してから新contentを接続し、region外node identityとlistenerを維持します。initial
attachment時に既存childrenがinitial Htmlと一致する場合はnodeを再利用します。現surfaceはregion-local childrenを一つの
structural valueとして扱い、`key`によるcollection diffを行いません。将来keyed collection surfaceを追加する場合もkeyは
そのregion内のsibling identityだけを表し、global component / state identityにはなりません。

Signal subscriberは5.13のtransaction commit後のstable valueだけを受け取ります。同一transactionの中間値をDOMへ
公開しません。`Signal.distinct`が同値publicationを止めた場合はbinding callbackもDOM writeも発生せず、callbackが
到達した場合もsinkの同値比較で不要なwriteを省きます。「一transactionにつき全DOMで一回だけwrite」は要求せず、影響を
受けたsinkごとの更新順はbinding配列とSignal publication順を保ちます。

DomContentのlistener、subscription、binding、nested region resourceはすべて現在の`DomMount`へ登録します。
unmount、root cancellation、target removal、dispatch failure後は新規publicationを反映せず、独自のglobal lifecycle managerを
作りません。region Html内のevent handlerが変わる場合はdelegated listenerを増やさずregion-local handler tableだけを
入れ替え、旧handlerへ到達できないようにします。global virtual tree、component hook、component call順から作るhidden
stateは導入しません。

互換用coarse updateはmanaged childrenを置換してもよく、一般のDOM node identityを保証しません。ただしevent受付と
subscriptionの所有権を二重化せず、IME composition中の入力を破棄せず、対応するcontrolled controlを識別できる場合は
focusとselectionを復元します。更新algorithmの違いでunmount、cancellation、cleanup、typed failureの意味を変えては
なりません。renderer外からactiveなmanaged subtreeを書き換えることはcontract外ですが、その検出方法をglobal
whole-tree snapshot比較へ固定しません。

## 13.11 hydration

FreshMountはtargetの既存childを削除してinitial treeをrenderします。HydrateStrictは既存DOMをinitial Html treeと
照合し、tag、namespace、text、typed propが一致するnodeを再利用します。eventとkeyはSSR出力に存在しないため、tree側
からlistenerと将来binding metadataを登録します。browserが隣接text nodeをmergeしている場合は同じ連結textなら境界を
splitして再利用できます。この照合はinitial attachmentだけを対象とし、mount後の更新algorithmとは独立です。

mismatch時、HydrateStrictは最初のpathをHydrationMismatchとして返して既存DOMを変更しません。
HydrateOrReplaceは一致したancestorを保ち、最小の不一致subtreeをinitial treeで置換します。replace modeの
mismatchはtyped failureにせず、hostにhydration diagnostic channelがある場合は最初のpathを一件報告できます。
hydration完了前にevent listenerを有効化せず、途中failureで半分だけinteractiveなtreeやsubscriptionを残しません。
mountContentはDomContentのinitial Htmlについてこの照合を完了した後、leaf / regionのcurrent Signal snapshotと既存sinkを
比較してbindingを接続します。snapshotがinitial Htmlと同値ならnode identityを維持し、hydration開始後にSignalが進んで
いれば対象leaf / regionだけを最新stable valueへ更新します。接続したreactive leaf / structural regionは同じ`DomMount`が
所有します。

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
dom.expected.contentPublications: initial snapshotを除きDOM adapterが受け取ったstable content publication数
dom.expected.activeListeners: cleanup後のlistener数
dom.expected.activeSubscriptions: cleanup後のSignal subscription数
```

eventsは一件のdispatch Effectと、それが起こした全Signal transaction / content publicationが安定してから次へ進みます。
selectorが0件または複数件、event fieldの型違い、afterHtml不一致、program終了前の未送信event、expected resource数の
不一致はfixture failureです。programがevent処理中に終了した場合はそのeventを完了させてcleanupを待ち、後続eventを
送信しません。real browser、network、wall clockへfallbackせず、scenario外のhost mutationを生成しません。
