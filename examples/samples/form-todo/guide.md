このsampleを選ぶ理由: 一つの画面で、Signal、controlled form、validation、
inline edit、filter、keyboard / pointer event、空状態まで通して体験したいときに選びます。
以前のTodo表を拡張するのではなく、launch loopという一つの仕事の流れを
mobile-firstのカードUIとして組み立てています。

## この画面で試すこと

- Plan name と Why it matters に入力して Add to launch loop を押す。
- Plan nameが空のときは送信buttonがdisabledになります。名前だけ入力して
  Why it mattersを空のまま送信すると、role: "alert" のvalidationを確認できます。
- Build / Share / Rest のtrack chooserを切り替える。
  これはselect相当の状態選択を、タッチしやすいbutton群で表現したものです。
- Keep this move in focus のcheckboxでpinned状態を持たせる。
- カードのタイトルをその場で編集し、Mark complete、Pin for focus、
  Removeを操作する。
- All / Pinned / Open / Done を切り替え、完了カードを
  Clear completed でまとめて取り除く。
- filter buttonで左右矢印、Home、Endを押してキーボード経路を試す。
  カードをpointerで触るとstatus live regionにも操作結果が表示されます。

## 構造

Modelがdraft、validation、filter、plans、lastInteractionを所有し、
PlanActionをupdateが純粋に次のModelへ畳み込みます。各DOMイベントは
InputEvent / ChangeEvent / KeyboardEvent / PointerEventの値を
feature内のadapterでsnapshotし、dispatchからTask<Unit>へ変換します。
そのためbrowser event objectをstateへ保存せず、フォームとカードが同じ
MutableSignal<Model>を共有できます。

表示はsignals.map (view state ...) stateで作り、mountは明示的な
signals.make + dom.runです。このsampleではdom.appを使いません。
dom.runのperformを自分で渡すことで、イベントをTask<Unit>として
実行する境界と、失敗をStringへ変換するmount処理をこのsingle-file
featureの中で読み取れるようにしています。純粋なreducerだけを学ぶなら
interactive-app、runtime接続だけを段階的に読むならsignal-run-routeが
先に向いています。

## UIの読み方

heroは固定HTTPSのUnsplash画像、source link、open / pinned / doneの
derived metricsを持ちます。formとboardはカードの配列として分け、
デスクトップでは2列、狭い画面では1列になります。テーブルを横に縮めず、
タイトル編集と状態操作を一枚のカードへまとめているのがこのsampleの
見せ場です。

各入力には対応するlabel.htmlForとidを持たせ、送信ボタンはvalidation
中にdisabledになります。role: "alert"は入力エラー、role: "status"は
最後の操作を示します。画像には意味のあるalt、幅、高さ、loading:
"eager"を指定し、読み込み中もレイアウトを安定させます。カード内の
buttonはstopClickPropagation: Trueでpointer処理と衝突しません。

## 前提と次のsample

interactive-appでpure viewとSignalの基本、signal-run-routeで
明示的なdom.run、feature-compositionでfeatureごとのlocal stateを
先に読むと、このsampleの構造を追いやすくなります。Explorerを含む複数
module構成へ進むときはproject-flow-appを選んでください。
