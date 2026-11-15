# 0029: rtlaのASTからrtlcのIRを生成する

rtlaとrtlcを独立したまま、橋渡しをrtla-compiler crateへ置く。既存TS frontendと同じ有限fragmentを目標に、解析済みModule/Configから型付きSafetyProgramを返す。名前解決、定数、operator/LETのlexical scope、action列挙、temporal/fairnessを変換する。未対応の意味論を補わず拒否する。

先にRustの挙動テストを追加してRedを保存する。TLCのTool.javaと既存TS compilerを読み、代入順序・UNCHANGED・有限量化・propertyのlevelを保つ。既存の固定入力からIRと結果をTS/native/Wasmで比較する。構文表現の差は明示し、TLC対照の原本・既存runを編集しない。新しいunsupported入力と空集合・資源切れ・違反も区別する。

modelt-coreへsource/configのcompile-tlaとcheck-tla requestを追加する。既存ABIのrequestは維持する。TS SDKと実ブラウザから同じWasmでRustの解析・変換・検査を呼ぶ。tla-tsを入れない単独consumerでも確認する。rtla parserとrtlc checkerの依存関係は増やさない。

plan、入力・結果・source digest、検証ログを新規runへ保存する。差分は実行時間の測定と分け、性能向上は主張しない。cargo fmt/clippy/test、portable test、check/browser、package、evaluate/replayを実行する。今回は全TLA+、INSTANCE、ユーザーモジュール、高階・再帰operator、symmetry、無限集合の列挙まで対応したとはしない。
