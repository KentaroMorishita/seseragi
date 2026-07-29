`signals.make`は更新可能な`MutableSignal`を作ります。Signalは時間とともに変わる現在値であり、`*total`はEffectの中でその時点の値を読みます。

`<$>`はpure関数をSignalへmapし、`<*>`はSignal内の関数を次のSignal値へ適用します。このApplicative合成は入力の変化を保ったderived Signalを作り、Monadとして前の値から次のSignalを選ぶ操作ではありません。`transaction`は複数更新の途中状態をobserverへ見せません。
