このsampleを選ぶ理由: `dom.app`版と同じTrail plannerを使い、Signal作成からDOM runtime接続までを自分で所有すると何が増えるか、sourceの末尾で比較したいときに選びます。

State / Action / pure `update` / `view`は`interactive-app`と同一です。初期HTMLと三つのAction後HTMLも同じで、違いは`// Runtime boundary`以降だけです。

森林のfixed Unsplash image、responsiveなroute action、progressのvisualも比較先と同一です。二つのsampleは同じapplicationを比較する一組なので、imageを別の題材へ変えず、runtime ownershipだけをsource末尾で読み比べられるようにしています。

明示的なruntime接続は、`signals.make initialState`で`MutableSignal`を作り、`signals.map view state`でHTMLのSignalを導出します。`handle state`がActionを受けて`signals.update (update action) state`を実行し、`dom.query "#app"`、`dom.defaultOptions ()`、`dom.run options target (handle state) content`を順に組み立てます。queryとrunのfailureもこの境界でportableな`String`へ変換します。

この版は単に長い旧APIではありません。effectful dispatch、custom options、mount lifecycle、複数のfeature-owned Signal、feature moduleの合成を自分で制御するときの入口です。一つのStateとpure reducerで完結するなら、比較先の`interactive-app`で`dom.app`へ同じmodel / viewを渡す方が意図を短く表せます。複数featureの実例は`feature-composition`へ進みます。
