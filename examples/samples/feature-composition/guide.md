各feature constructorはprivateな`MutableSignal`を一度だけ生成し、外へは`Signal<Html<Effect<{}, Never, Unit>>>`だけを返します。親は子の`CounterState`や`CounterAction`を知らず、Effectになったeventをrootの実行境界へ渡すだけです。

`Hide / show`では`switchMap`が表示branchを切り替えます。非表示中もconstructorを呼び直さないためstateは保持されます。`Swap order`でnode順を変えてもstateは`first` / `second`のSignal bindingに所属し、HTMLの`key`やcomponent呼び出し順をfeature identityには使いません。

Playgroundは単一file編集なので同じ境界を一file内で見せています。実際のmodule privacy、Todo Form / Listのshared state、root所有のapp-wide stateは`project-schema-1/feature-module-composition`、動的branchは`project-schema-1/feature-module-lifetime`で固定しています。
