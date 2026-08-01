このsampleを選ぶ理由: 明示的なSignal + `dom.run`を使うsingle-file Showcaseとして、controlled form、validation、derived view、inline edit、empty state、keyboard / pointer eventをまとめて試したいときに選びます。

一つのfeature-owned `MutableSignal<Model>`から`Signal<Html<Task<Unit>>>`を作り、form、editable table、
文書link / image、keyboard / pointer操作を同じWeb UIへ統合します。親へ巨大なAction unionを公開せず、
各eventはfeature内の`dispatch`で`Task<Unit>`へ変換されます。

sourceはdomain / updateの後に`cx`と役割名を持つclass valueをまとめ、intro、form、filter navigation、
workspace card、tableを画面上の意味単位へ分けています。同じinput classは再利用し、note / priority cellは
動的なclass文字列を渡さず、それぞれのcomponentが固定したvisual contractを所有します。

- intro画像とsource linkは固定HTTPS URLを`parseWebUrl`で検証してから使います。画像は意味のある`alt`、幅・高さを持ち、取得中もlayoutを安定させます。
- `label.htmlFor`とcontrolの`id`、native button、`role: "status"`で基本的なaccessibilityを保ちます。
- `onInput` / `onChange`はhost Eventを保持せず、immutable snapshotからTaskを作ります。
- form submitは同期的にdefaultを防いでからTodoを追加します。
- table内のtitle inputでTodoを編集し、Delete、All / Urgentで削除・絞り込みできます。
- filter buttonへfocusしたあと、左右矢印でもAll / Urgentを切り替えられます。
- rowの`onPointerDown`はmouse / touch / penを同じ`PointerEvent`として扱うため、iOSのtouch操作も
  browser objectをstateへ持ち込みません。
- Deleteは`stopClickPropagation`を指定し、nested controlのclickをrow側へ漏らしません。

日本語IMEの変換中はrerenderを保留し、確定した文字列だけを一度Taskへ変換します。Playgroundでは
title / noteの変換入力、inline edit、keyboard filter、touch操作を続けて試せます。

pure reducerだけの最小構成は`interactive-app`、複数Signalとcustom実行境界を段階的に読む場合は`feature-composition`が前提です。Explorer込みのmodule分割へ進む場合は`project-flow-app`を選びます。
