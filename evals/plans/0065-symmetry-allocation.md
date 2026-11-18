# 0065: SYMMETRYのnode行の割り当て

[#67](https://github.com/mizchi/idealis/issues/67)。基準はa42670fb。準備済みAPIの次に、nodeごとのVecとcurrent stateのOption列コピーを減らす。縮約条件・joint rowの相関・完全key比較・具体反例・論理workは維持する。

## 仮説

NodeRowsのVec<Vec<Scalar>>を、group数×column数の平坦なVecへ変える。canonical keyはrow indexを同じScalar順でsortし、各sliceを同じ順で並べる。代表選択はsliceの完全比較を使う。currentとnextをiteratorで読み、currentのSome列を作らない。

columnが0のglobal scalarだけの状態では、元の実装と同じく先頭のnodeを代表にする。partial state、subset、flat recordのschemaや型違い、typed/未対応形の拒否と、予算切れのphaseを変えない。arena.entriesのコピーとshapeのcacheは混ぜず、最初の変更だけを評価する。

## Redと対照

既存のjoint-row相関、完全keyの衝突、concrete trace、worker remapとTLC対照を使う。追加でcolumn 0の代表、Refのcolumn数が複数の場合、global variableの位置を跨ぐ場合、エラーと小予算を検査する。見た目のallocation回数だけを性能改善と数えない。

## 計測と採否

beforeのsource/native/Wasmとsymmetry Criterion binaryを固定する。既存21本のsymmetry/reduction fixtureを同じcompiled inputでABBA、20 samples、warmup 0.5秒、measurement 1秒で比較する。初期の対象をrecord-12のsymmetry/actionsの1/8 workerに固定し、有用なら全21本と通常14本へ進む。

約5%以上のnative短縮または明確な割り当て・RSS削減を採用の根拠とする。10%以上の後退は固定binaryのABBAで再確認し、再現する汎用変更は戻す。source/IR境界を含むrecord/set/jointと通常caseも1 warmup + 3 samplesで別に測り、state/generated/work/verdictの一致を要求する。native 1/8要求とWasmを区別する。logical workは実際のsort回数に合わせて減らさない。

rawは追記のみ。build/testと通常計測を重ねず、source/binary/input/順序/全sampleを保存する。小さい差だけならshape cacheや一般の対称性へ広げず、他の実モデルで効果を示せる候補を優先する。

終了前にcheck/browser、safety/portable、analysis、fmt/clippy、旧版差分とTLC対照、package、SUPPORT/Jevを確認する。準備済みAPIの不変な共有と、探索ごとのsymmetry代表が独立する契約も保つ。結果をiteration/#67/#63へ戻して次の採否を決める。
