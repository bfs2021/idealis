# 0061: 限定symmetryの次の改善

基準は62eac486。Rust nativeを主指標にし、Wasmは移植性と挙動を検査する。各段階を順に実装・評価し、後退する案は保存して採用を見送る。既存runは編集しない。pushとmergeは行わない。

## 順序と仮説

0. 0060の通常ringの変動を診断する。凍結したbefore/final binaryを同じfilterでbefore/after/after/before、さらに逆順で測る。Criterionは20 samples、warmup 0.5秒、measurement 1秒。buildやtestと同時に測らない。持続する差があればprofileと原因の修正を先に行う。
1. node参照のないrecord payloadとnode部分集合を状態として扱う。各nodeのrecord全体と各集合の所属bitを同じ行に含める。recordのschema・型とglobal値をkeyへ含め、相関を失わない。node参照、任意のnested構造、複数群は対象外。
2. 単一node bindingの操作で、現在状態の同じ行に属するnodeを代表だけ展開する試作を行う。初期状態生成へ適用せず、個別node・順序依存式を拒否する既存gateと利用者の対称性前提を維持する。複数bindingは通常展開へ戻す。具体trace、deadlock、予算を検査する。generated/workが変わる場合は明記する。
3. symmetryでも既存の並列frontier展開を試す。workerの値をrootへ移送してから逐次にcanonical keyを作る。狭いfrontierは逐次へ戻す。小さい縮約グラフと大きい縮約グラフを比較する。
4. 式ごとのHashMap検索を避ける命令IDの直接実行を試す。scalarのread/literal、算術・比較を最小対象とし、短絡・overflow・workの挙動を維持する。既存評価器へのfallbackを保ち、通常counter/ringで有用な短縮がなければ採用しない。
5. safetyに限定した部分順序削減を試す。独立性・propertyへの可視性・cycle条件を保守的に判定する。縮約できない場合は完全探索へ戻す。誤検出・見逃しがあれば採用せず、検査した境界と次の条件を記録する。

## 正しさの対照

Redを先に実行し、logを保存する。recordはN2/3とInt(0..2)・Bool field、集合はN2/3と2集合・関数との組み合わせを固定する。安全版と到達後に破れるinvariantを比較し、TLCで縮約あり/なしの判定を確認する。縮約なしのNextによる具体反例の再生、native/Wasmの照合、hash衝突・型・schema・相関・拒否条件、work/state/depth予算を検査する。

各段階の候補はsourceとbinary hashで固定する。counter N5/K3、N7/K5と2 Bool関数N5を既存対照にし、record N5/K2と集合+Bool関数N5を追加する。並列は同じ候補の1/8 worker要求を比較する。追加の大きい入力や条件は実行前にこのplanへ追記する。起動込みとpreparedを区別し、計数が異なる縮約のworkやgeneratedを同一の仕事量とは扱わない。

## 採否と検証

TLCへの判定・利用可能な計数の一致、反例再生、既存回帰を前提にする。速度の採用は同じホストで順序を入れ替えた測定から判断し、10%以上の後退を明記する。record/集合は対応範囲の追加価値も評価する。実際の改善がない試作は撤去し、試作source・失敗log・結果を保存する。

終了前にcheck/browser、portable-build/test、safety-test、runtime-build/test、fmt/clippy、evaluate/replay-evaluationを実行する。共有TSテストの変更があればmutation、配布変更があればpackageも実行する。SUPPORT/Jevのliveとofflineを確認し、キーなしのskipと検査済みを区別する。各段階の採否、未解決の制限、追加測定の条件をiterationへ記録する。
