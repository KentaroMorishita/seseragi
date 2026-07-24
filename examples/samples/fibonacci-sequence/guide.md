`loop`は「残り回数・現在値・次の値」を引数に持ち、再帰呼び出しをtail positionへ置きます。そのため入力に比例してhost call stackを増やしません。`for`は`0..=10`を先頭から走査し、計算したFibonacci数を同じ順序で出力します。
