# 空行を除くprogram WASM

Seseragiのcalculation moduleでstd/text.lines・List.filter・std/text.joinを使い、Unicodeの改行を認識して空行を除き、残る行の内容と順序を保ったままLFで返す。空/全空行の結果は空文字、それ以外の結果には末尾LFを付ける。trimやUnicode正規化は行わない。

標準Web buildのボタンは元のtyped bridgeから返却callbackを呼び、finallyでcallbackとinstanceを閉じる。通常の開発経路で次のようにbuildする。

```sh
seseragi build . --target web --profile development --out-dir site
```

同じclean関数はnativeのfile処理にも使う。CLI検証では通常buildの両profile・fresh Bun/Node、元のbridge/型宣言のstrict検査、Unicode/空行、codec拒否後の回復、callback独立性・解放後の拒否とroot0を確認する。Array<String>は内部値として使い、公開する境界はString入出力と返却callbackに限る。公開Array codec、一般provider ABIやstreamingはこの例へ含めない。
