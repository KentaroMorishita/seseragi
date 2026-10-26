# Stringの内容を比較するprogram

標準Web buildでpureなpredicateをWASMへ生成し、Seseragi UIから元のtyped bridgeを通して呼ぶ。
ボタンで`ready\n`を比較すると`accepted`を表示する。bridgeは毎回instanceと返却callbackを所有し、
終了時にfinallyで解放する。

```sh
seseragi build . --target web --profile development --out-dir site
```

`src/predicate.ssrg`のString Eqは、Unicode/NUL、独立した文字列、保存methodと部分適用を使う。
CLI検証は通常buildの両profileとfresh Bun/Nodeで元の型宣言・bridgeをstrict検査し、
返却callbackの独立性、不正codec入力後の回復、解放後の拒否とmanaged root=0を確認する。
同じpredicateのprobeをnativeでも実行する。一般provider ABIや公開Ref等の追加はこの例に含めない。
