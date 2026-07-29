buttonの`onClick`はtypedな`Action`をdispatchします。`update`はActionと現在stateだけから次の`Counter`を返すpure reducerで、`view`もstateからpureなHtml treeを作ります。

`dom.app`は単一feature向けのconvenienceです。内部で`MutableSignal<Counter>`を所有し、eventごとのreducer適用を`Task<Unit>`のSignal更新として実行してからviewを再描画します。子featureがstateを直接所有する構成では、同じ境界を`Html<Task<Unit>>`として親へ渡し、rootの`dom.run`がTask actionを実行できます。
