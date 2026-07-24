`[]`と`[head, ...tail]`を並べると、空Arrayと空でないArrayを網羅できます。Listではopening bracketの前にbacktickを付けて、`` `[] ``と`` `[head, ...tail] ``を使います。`head`は要素型、`tail`は元と同じCollection型です。

Listの`tail`は元のListを共有するため定数時間です。Arrayの`tail`は残りのArrayを作るため、同じ再帰形でも計算量が異なります。
