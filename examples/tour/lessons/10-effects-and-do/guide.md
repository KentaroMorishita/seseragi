`Effect<R, E, A>`は、service環境`R`を要求し、`E`で失敗するか`A`で成功する遅延計算です。値として組み立てただけでは処理を始めません。

`with Console`は必要なservice、`fails ConsoleError`は回復可能な失敗を明示します。`do`はEffectを上から順に合成し、`<-`は成功値を待って名前へbindします。`succeed`はpureな値を失敗しないEffectへ持ち上げます。
