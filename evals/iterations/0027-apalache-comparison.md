# 0027: Apalacheとrtlcの方式・保証・費用を比較する

[比較資料](../../docs/apalache-comparison.md)に機能表、保証の違い、小規模pilotの全6条件と制約を記録した。[計画](../plans/0027-apalache-comparison.md)を先に固定し、Apalache v0.63.0を公式checksumで検査してから実行した。

[pilot](../runs/0027-apalache-pilot/result.json)は同じsource/configで54測定と18 warmupを実行した。TLA+解析を含むfresh processの中央値はrtlc経路61.90-73.86 ms、Apalache1119.62-1873.60 ms。これはJVM・SMT準備を含む小規模検査の費用であり、kernelや大規模状態空間の速度比ではない。hostを専有しておらず、全sampleの範囲も保存した。

Apalacheのboundedな反例なし、rtlcのcomplete、depth予算切れを区別した。故障ringはk=4では見つからず、k=8で両者が違反を返した。関数の故障版は初期状態で違反した。全反例の初期状態・遷移・最終違反を独立したvalidatorで確認し、元のtraceとともに保存した。実SUTへの反例再現や未知故障発見は今回行っていない。

正常ringの初期成立と保存条件もApalacheで別に検査した。固定16状態の帰納的不変条件が成立することを確認し、性能sampleには混ぜていない。

採用判断は、rtlcの置き換えをせず、ITFによる接続と保証範囲を区別するbackend契約を次の候補にする。Apalache自体のbrowser移植やSMT backendの実装を採用した結果ではない。大規模domain・常駐server・長い反例・Wasm/並列の比較は残る。[検証記録](../runs/0027-apalache-verification/result.json)にchecksum、保存結果の再検査、テストログを置く。
