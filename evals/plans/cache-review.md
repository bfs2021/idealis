# Cache review regressions

PR #70 のレビューで指摘された契約境界を、反例を追加してから修正する。

- positional な program/bounds を options の余分なプロパティで置換できない。
- 無効化時は cache を読み取らず、循環参照や不正な snapshot に触れない。
- snapshot は canonical な値、dense な配列、正確な own fields だけを受け付ける。
- receipt は実行した lease の owner/fence/runId に結び付ける。以前の実行の receipt を別 worker の成果にしない。
- JSON で区別できない -0 は lease 取得前に拒否し、+0 は受け付ける。
- digest が壊れた ready cell は CAS と fence を保ったまま evict できる。
- release の失敗は成功結果や先行した例外を上書きしない。cleanup error は呼び出し側へ返す。
- Rust の cache 用 portable 変換は1展開64 KiBの保守的な予算内に限定する。超過時と保存容量不足時は通常の streaming 探索を続け、不完全な展開を保存しない。

Red と Green の出力、評価・再生・証明を別ディレクトリへ保存する。既存の性能値は過去のソースに対する値として保持する。検査結果の比較では work のみを除外し、状態数・生成数・反例・bounded を維持する。小さい共有木の回帰を Red に使い、修正後は深さ40の共有木でも巨大なコピーを作らず bounded に到達することを確認する。
