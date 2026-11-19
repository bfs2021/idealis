# Quint生成物のIR v1/v2回帰とベンチマーク

基点はPR #62をマージしたmain d38fdf5f1。保存済みのQuint 0.33.1 / Apalache 0.63.0の6生成物を使い、生成TLA+と既存TLCオラクルは変更しない。Quint自体の最新版への追従は今回の対象にしない。

- 正常/違反の12設定を、省略v1・明示v1・明示v2とRust native/Wasmで検査する。IRのversionを実際に確認し、同じ経路のcompile/checkと直接check-tlaを照合する。v1とv2のworkは同一と仮定しない。
- 判定と、TLCに値がある場合の状態数・生成数を独立したTLCオラクルに照合する。初期違反の欠測はnullのまま残す。拒否・予算切れ・空探索を一致へ読み替えない。
- Chromiumでも同じversion行列を実行する。生成元・stdout・TLA+・設定のdigestとバイト列を確認し、ベンチ用のcrate内fixtureも原本と照合する。
- Criterionで6モデルの解析、compile、prepared check、parse+compile+checkを分ける。compile/checkはv1/v2で比較し、各測定前にTLCの正常判定・状態数・生成数を確認する。JVM起動・Quint生成・JSONL輸送はこの測定に含めない。
- sample 20、warm-up 500 ms、measurement 1 s。CIは時間の閾値を設けずbenchのsmoke検査を行う。単一ホストの中央値と不確かさを記録し、一般的な高速化や未知故障発見の証拠にはしない。
- raw出力・入力とsourceのdigest・環境・コマンドを新しいrunへ保存し、上書きを拒否する。既存の0044とPR #62のrunは変更しない。
- 条件の変更とRedを保存する。check、browser、mutation、quint-test、compilerのfmt/clippy/all-targets、評価と再生、Jevの同期を実行する。Quintの再生成とTLCの再照合は利用できる固定toolchainで別runに保存する。

固定corpusによる回帰であり、新しい無作為試験や独立した確認seedの実験として集計しない。実行時間・IR bytes・workを分け、v2で遅くなる場合も記録する。
