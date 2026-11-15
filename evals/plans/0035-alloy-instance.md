# 0035: 実装の関係スナップショットをAlloyモデルへ照合する

`ralloy`へ固定instanceの検査を追加する。探索で別の構造を見つける機能と分け、実装が返したsig/fieldをそのまま評価する。`Model::check_instance(command, instance, max_work)`と同じnative/Wasm requestを提供し、その上へNode不要のTS/Wasm adapterを置く。既存の汎用backend契約、CLI、Alloy構文の対応範囲は変更しない。

結果はmatched、mismatch、invalid、unknownを区別する。matchedは指定した構造が宣言・fact・commandの式を満たす場合。check commandはassertの真偽を返し、探索時の反例検索の否定とは区別する。性質の違反はmismatch、scope・宣言multiplicity・factの違反はinvalidとし、reasonを返す。malformedなinstance、未知の宣言・atom、重複atom/tupleはprotocol error。workが不足する場合はunknownで、成功にしない。1つの観測構造の照合であり、全実装入力や時間的振る舞いの証明とは扱わない。

atom名は実装のopaque IDを使える。sig間はdisjointで、builtin Intと衝突してはならない。fieldは全宣言分を明示し、欠落を空関係へ補完しない。空sigのfield制約は既存の意味論どおりvacuous。field range、one/lone/some、scope/exactly、Int、事実・性質を順に検査する。parser・入力復号の費用はworkの対象外で、wall clock制限は追加しない。

先にRustのAPIテストを失敗させる。DAG・2-cycle・self-loopの仮定違反、scope違反、multiplicity、空owner、opaqueなatom名、壊れた輸送データ、予算切れ、探索からの反例再照合を検査する。TSは実Wasm/nativeで契約を共有し、node:testとVitest、実Chromium、配布archiveの別consumerで確認する。公開型はcontracts.tsへ定義し、コアはNode/テストランナーへ依存させない。

評価はAlloy 6.2.0/SAT4Jへ構造を固定し、factsと宣言の可否、propertyの真偽を照合する。開発seed0x35a1の16件、確認seed0x35c3の32件を新しく予約する。固定ケースも含め、2-3 nodeのTSグラフ走査による独立した期待値を使う。未知入力・schema違反は成功へ変換しない。2-cycleを受理する既知の故障版とDAG判定の正常版を対照とし、未知バグ発見とは数えない。

planを先にコミットし、開発・確認の入力と結果、ソース・binary・公式JARのhash、全receiptを新規runへ保存する。以前の確認seedを新しい独立確認に再利用しない。必要なcheck/browser、Rust fmt/clippy/test、runtime-build/test、evaluate/replay、package、mutationを実行する。新しいRust APIとTS adapterを別のstack層へ分け、mergeやregistry publishは行わない。
