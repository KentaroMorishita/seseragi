このsampleを選ぶ理由: `dom.app`が扱う一つのpure reducerを越えて、複数のfeature-owned Signal、effectful Action、custom実行境界を明示的な`dom.run`で制御したいときに選びます。

各feature constructorはprivateな`MutableSignal`を一度だけ生成し、外へは`Signal<Html<Task<Unit>>>`だけを返します。親は子の`CounterState`や`CounterAction`を知らず、Effectになったeventをrootの実行境界へ渡すだけです。

`Hide / show`では`switchMap`が表示branchを切り替えます。非表示中もconstructorを呼び直さないためstateは保持されます。`Swap order`でnode順を変えてもstateは`first` / `second`のSignal bindingに所属し、HTMLの`key`やcomponent呼び出し順をfeature identityには使いません。

Playgroundは単一file編集なので同じ境界を一file内で見せています。実際のmodule privacy、Todo Form / Listのshared state、root所有のapp-wide stateは`project-schema-1/feature-module-composition`、動的branchは`project-schema-1/feature-module-lifetime`で固定しています。

長いutility列は同じsourceの`cx`と役割名を持つclass valueへ分けています。Signalの所有権を説明するcomponentと見た目の定義を混ぜず、上からclassの意図、component、mount経路を追えます。

一つのState / Action / pure reducerだけで足りる場合は`interactive-app`の`dom.app`が短い選択です。form、validation、keyboard / pointer eventまで一つの画面で試す場合は`form-todo`へ、同じSignal ownershipをmodule境界で追う場合は`project-flow-app`へ進みます。
